use regex::Regex;
use std::sync::Arc;

use crate::{BinaryRequest, ResponseBuilder};
use crate::utils::ToRegex;
use crate::utils::RequestContinuation;
use crate::utils::RequestContinuation::*;
use tokio::prelude::*;
use crate::request::Request;
use crate::response::AsyncResponder;

///
pub struct Builder {
    stack: Vec<(MiddlewareRule, Box<Middleware>)>,
}

impl Builder {
    /// Creates a new MiddlewareStack Builder
    pub fn new() -> Self {
        Builder {
            stack: Vec::new()
        }
    }

    /// Method to apply a new middleware onto the stack where the `include_path` vec are all path affected by the middleware,
    /// and `exclude_path` are exclusion amongst the included paths.
    pub fn apply<M: 'static + Middleware>(mut self, m: M, include_path: Vec<&str>, exclude_path: Option<Vec<&str>>) -> Self {
        let rule = MiddlewareRule::new(include_path, exclude_path);
        let boxed_m = Box::new(m);

        self.stack.push((rule, boxed_m));

        self
    }

    /// Build the middleware stack
    pub fn build(self) -> MiddlewareStack {
        let Builder {
            stack,
        } = self;

        MiddlewareStack {
            middlewares: Arc::new(stack),
        }
    }
}

/// Struct representing the layering of middlewares in the server
pub struct MiddlewareStack {
    middlewares: Arc<Vec<(MiddlewareRule, Box<Middleware>)>>
}

impl MiddlewareStack {
    ///
    pub fn new() -> Self {
        MiddlewareStack {
            middlewares: Arc::new(Vec::new())
        }
    }

    ///
    pub fn resolve(&self, req: &mut BinaryRequest, res: &mut ResponseBuilder) -> RequestContinuation {
        let path = req.uri().path().to_owned();

        for &(ref rule, ref middleware) in self.middlewares.iter() {
            if rule.validate_path(&path) {
                if let Stop = middleware.resolve(req, res) {
                    return Stop;
                }
            }
        }

        Continue
    }
}

impl Clone for MiddlewareStack {
    fn clone(&self) -> Self {
        MiddlewareStack {
            middlewares: self.middlewares.clone(),
        }
    }
}

/// The trait a struct need to `impl` to be considered as a middleware
pub trait Middleware: Send + Sync {
    /// This method will be invoked if the request is targeting an included path, (as defined when "applying" the middleware to the stack)
    /// and doesn't match any exclusion. Returning `RequestContinuation::Continue` will allow the request to continue through the stack, and
    /// returning `RequestContinuation::Stop` will cease the request processing, returning as response the modified `res` param.
    fn resolve(&self, req: &mut BinaryRequest, res: &mut ResponseBuilder) -> RequestContinuation;
}

struct MiddlewareRule {
    included_path: Vec<Regex>,
    excluded_path: Option<Vec<Regex>>,
}

impl MiddlewareRule {
    pub fn new<R: ToRegex>(include_path: Vec<R>, exclude_path: Option<Vec<R>>) -> Self {
        let mut included_path = Vec::new();
        for include in include_path.iter() {
            included_path.push(reg!(include));
        }

        let mut excluded_path: Option<Vec<Regex>> = Option::None;

        if let Some(excludes) = exclude_path {
            let mut excludes_vec = Vec::new();
            for exclude in excludes.iter() {
                excludes_vec.push(reg!(exclude));
            }

            excluded_path = Some(excludes_vec);
        }

        MiddlewareRule {
            included_path,
            excluded_path,
        }
    }

    pub fn validate_path(&self, path: &str) -> bool {
        let path_clone = path.clone();
        if self.included_path.iter().enumerate().find(
            move |&(_index, r)| {
                r.is_match(path_clone)
            }
        ).is_some() {
            if let Some(ref excluded_path) = self.excluded_path {
                return excluded_path.iter().enumerate().find(
                    move |&(_index, re)| {
                        re.is_match(path_clone)
                    }
                ).is_none();
            } else {
                return true;
            }
        }

        false
    }
}

///
#[derive(Clone)]
pub struct MiddlewareCollection {
    middlewares: Arc<Vec<(MiddlewareRule, Box<Resolver>)>>,
}

impl MiddlewareCollection {
    ///
    pub fn new() -> Self {
        MiddlewareCollection {
            middlewares: Arc::new(Vec::new())
        }
    }

    ///
    pub fn resolve(&self, request: Request) -> impl Future<Item=Continuation, Error=()> {
        MiddlewaresResolverFuture {
            request: Some(request),
            middlewares: self.middlewares.clone(),
            current: None,
            next: 0
        }
    }
}

struct MiddlewaresResolverFuture {
    request: Option<Request>,
    middlewares: Arc<Vec<(MiddlewareRule, Box<Resolver>)>>,
    current: Option<ContinuationFuture>,
    next: usize,
}

impl Future for MiddlewaresResolverFuture {
    type Item = Continuation;
    type Error = ();

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        if let Some(fut) = self.current.as_mut().take() {
            match fut.poll()? {
                Async::Ready(Continuation::Continue(request)) => {
                    self.request = Some(request);
                }
                Async::Ready(Continuation::Stop(req, responder)) => {
                    return Ok(Async::Ready(Continuation::Stop(req, responder)));
                }
                _ => return Ok(Async::NotReady),
            }
        }

        loop {
            let next = &self.middlewares[self.next];

            {
                self.next += 1;
            }

            if next.0.validate_path(self.request.as_ref().expect("A MiddlewaresResolverFuture without request should not exist, this is fatal").uri().path()) {
                self.current = Some(next.1.resolve(self.request.take().expect("A MiddlewaresResolverFuture without request should not exist, this is fatal")));
                return Ok(Async::NotReady);
            }

            if self.next > self.middlewares.len() {
                return Ok(Async::Ready(Continuation::Continue(self.request.take().expect("A MiddlewaresResolverFuture without request should not exist, this is fatal"))));
            }
        }
    }
}

///
pub struct MiddlewareCollectionBuilder {
    stack: Vec<(MiddlewareRule, Box<Resolver>)>,
}

impl MiddlewareCollectionBuilder {
    /// Creates a new MiddlewareStack Builder
    pub fn new() -> Self {
        MiddlewareCollectionBuilder {
            stack: Vec::new()
        }
    }

    /// Method to apply a new middleware onto the stack where the `include_path` vec are all path affected by the middleware,
    /// and `exclude_path` are exclusion amongst the included paths.
    pub fn apply<M: 'static + Resolver>(mut self, m: M, include_path: Vec<&str>, exclude_path: Option<Vec<&str>>) -> Self {
        let rule = MiddlewareRule::new(include_path, exclude_path);
        let boxed_m = Box::new(m);

        self.stack.push((rule, boxed_m));

        self
    }

    /// Build the middleware stack
    pub fn build(self) -> MiddlewareCollection {
        let MiddlewareCollectionBuilder {
            stack,
        } = self;

        MiddlewareCollection {
            middlewares: Arc::new(stack),
        }
    }
}

///
pub enum Continuation {
    ///
    Stop(Request, Box<AsyncResponder>),
    ///
    Continue(Request),
}

///
pub struct ContinuationFuture {
    ///
    inner: Box<Future<Item=Continuation, Error=()> + Send>
}

impl Future for ContinuationFuture {
    type Item = Continuation;
    type Error = ();

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        self.inner.poll()
    }
}

///
pub trait Resolver: Send + Sync {
    ///
    fn resolve(&self, request: Request) -> ContinuationFuture;
}

impl<F, U> Resolver for F where F: Send + Sync + Fn(Request) -> U, U: 'static + Send + Future<Item=Continuation, Error=()> {
    fn resolve(&self, request: Request) -> ContinuationFuture {
        ContinuationFuture {
            inner: Box::new((*self)(request))
        }
    }
}