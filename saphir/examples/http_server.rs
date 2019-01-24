extern crate saphir;

use saphir::*;

struct TestMiddleware {}

struct QueryParams(Vec<(String, String)>);

impl Middleware for TestMiddleware {
    fn resolve(&self, req: &mut BinaryRequest, _res: &mut ResponseBuilder) -> RequestContinuation {
        println!("I'm a middleware");
        println!("{:?}", req);

        let params = if let Some(_query_param_str) = req.uri().query() {
            vec![("param1".to_string(), "value1".to_string()), ("param2".to_string(), "value2".to_string())]
        } else {
            vec![]
        };

        req.extensions_mut().insert(QueryParams(params));

        RequestContinuation::Continue
    }
}

struct TestControllerContext {
    pub resource: String,
}

impl TestControllerContext {
    pub fn new(res: &str) -> Self {
        TestControllerContext {
            resource: res.to_string(),
        }
    }

    pub fn function_to_receive_any_get_http_call(&self, _req: &BinaryRequest, res: &mut ResponseBuilder) {
        res.status(StatusCode::OK).body(format!("this is working nicely!\r\n the context string is : {}", self.resource));
    }
}

fn main() {
    let server_builder = Server::builder();

    let server = server_builder
        .configure_middlewares(|stack| {
            stack.apply(TestMiddleware {}, vec!("/"), None)
        })
        .configure_router(|router| {
            let basic_test_cont = BasicController::new("^/test", TestControllerContext::new("this is a private resource"));

            basic_test_cont.add(Method::GET, reg!("^/$"), TestControllerContext::function_to_receive_any_get_http_call);

            basic_test_cont.add(Method::POST, reg!("^/$"), |_, _, _| { println!("this was a post request") });

            basic_test_cont.add(Method::GET, reg!("^/panic$"), |_, _, _| { panic!("lol") });

            basic_test_cont.add(Method::GET, reg!("^/timeout"), |_, _, _| { std::thread::sleep(std::time::Duration::from_millis(15000)) });

            basic_test_cont.add(Method::GET, reg!("^/query"), |_, req, _| {
                if let Some(query_params) = req.extensions().get::<QueryParams>() {
                    for param in &query_params.0 {
                        println!("{:?}", param);
                    }
                }
            });

            basic_test_cont.add_with_guards(Method::PUT, "^/patate", BodyGuard.into(), |_, _, _| { println!("this is only reachable if the request has a body") });

            let basic_test_cont2 = BasicController::new("^/test2$", TestControllerContext::new("this is a second private resource"));
            basic_test_cont2.add(Method::GET, reg!("^/$"), |_, _, _| { println!("this was a get request handled by the second controller") });

            // This will add the controller and so the following method+route will be valid
            // GET  /test/
            // POST /test/
            // GET  /test/query
            // PUT  /test/patate

            // This will add the controller at the specified route and so the following method+route will be valid
            // GET  /api/test2/

            router.add(basic_test_cont)
                .route("^/api", basic_test_cont2)
        })
        .configure_listener(|listener_config| {
            listener_config.set_uri("http://0.0.0.0:12345")
                .set_request_timeout_ms(10000) // 10 sec
                .set_panic_handler(|panic| {
                    println!("HA HA! : {:?}", panic);
                })
        })
        .build();

    if let Err(e) = server.run() {
        println!("{:?}", e);
        assert!(false);
    }
}