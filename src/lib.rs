#![deny(unused_extern_crates)]
#![deny(missing_docs)]
#![deny(warnings)]

//! # Saphir
//!
//! Saphir is a progressive http server framework based on Hyper-rs that aims to reduce the time spent on playing with futures and
//! limiting the amount of copied code amongst request matching.
//!
//! Saphir provide what's needed to easily start with your own server with middleware, controllers and request routing.
//!
//! Futures version will comes with more macro and a nightly experiment is currently being tested to reproduces decorator in rust.

#[macro_use]
mod utils;
/// Modules for the error handling into saphir
pub mod error;
/// Modules for the middlewares
pub mod middleware;
/// Modules for the controllers
pub mod controller;
/// Modules for responses
pub mod response;
/// Modules for request
pub mod request;
/// Modules for the router
pub mod router;
/// Modules for the http server
pub mod server;

pub use regex;
pub use hyper;

///
pub mod header {
    pub use http::header::*;
}

pub use http::StatusCode;
pub use http::Version;
pub use http::Method;
pub use http::Uri;
pub use crate::header::*;
pub use crate::utils::*;
pub use crate::request::*;
pub use crate::response::*;
pub use crate::middleware::Middleware;
pub use crate::middleware::MiddlewareStack;
pub use crate::controller::Controller;
pub use crate::controller::BasicController;
pub use crate::controller::ControllerDispatch;
pub use crate::controller::RequestGuard;
pub use crate::controller::RequestGuardCollection;
pub use crate::controller::BodyGuard;
pub use crate::router::Router;
pub use crate::server::{Server, ServerSpawn};
pub use crate::error::ServerError;