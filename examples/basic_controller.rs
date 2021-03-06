extern crate saphir;

use saphir::*;

struct TestControllerContext {
    pub resource: String,
}

impl TestControllerContext {
    pub fn new(res: &str) -> Self {
        TestControllerContext {
            resource: res.to_string(),
        }
    }

    pub fn function_to_receive_any_get_http_call(&self, _req: &SyncRequest, res: &mut SyncResponse) {
        res.status(StatusCode::OK).body(format!("this is working nicely!\r\n the context string is : {}", self.resource));
    }
}

fn main() {
    let server_builder = Server::builder();

    let server = server_builder
        .configure_router(|router| {
            let basic_test_cont = BasicController::new("/test", TestControllerContext::new("this is a private resource"));

            basic_test_cont.add(Method::GET, "/", TestControllerContext::function_to_receive_any_get_http_call);

            basic_test_cont.add(Method::POST, "/", |_, _, _| { println!("this was a post request") });

            basic_test_cont.add_with_guards(Method::PUT, "/with-body", BodyGuard.into(), |_, _, _| { println!("this is only reachable if the request has a body") });

            router.add(basic_test_cont)
        })
        .configure_listener(|listener_config| {
            listener_config.set_uri("http://0.0.0.0:12345")
        })
        .build();

    if let Err(e) = server.run() {
        println!("{:?}", e);
        assert!(false);
    }
}