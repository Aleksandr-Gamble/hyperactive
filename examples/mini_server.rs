//! This shows an example http server constructed using the Rust -> Tokio -> Hyper -> Hyperactive stack.
//! Note the general pattern is a request router that matches the (method, path) or a request to a function that 
//! returns hyper::Response<hyper::Body>.  
//! 
//! To run this server, use the following command:
//! ```cargo run --example mini_server```
//! 
//! You can then test the output of making the below http calls in a client of your choice, perhaps curl or Postman or a similar application:  
//! GET http://127.0.0.1:8080/
//! GET http://127.0.0.1:8080/users?user_id=5
use serde::Serialize;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyperactive::{err::HypErr, server};


static INDEX: &[u8] = b"Hello from the Rust -> Tokio -> Hyper -> Hyperactive stack!";
static NOTFOUND: &[u8] = b"Not Found";

#[derive(Serialize)]
struct User {
    id: i32,
    name: String,
}


async fn request_router(req: Request<Body>, _ip_address: String) -> Result<Response<Body>, HypErr> {
    /* Notice a pattern in the signature for this function:
    All the arguments consume them, but then the routing consumes a reference to the consumed arguments */
    let _hdrs = server::get_common_headers(&req);
    match (req.method(), req.uri().path()) {
        (&Method::OPTIONS, _) => server::preflight_cors(req).await,
        (&Method::GET,  "/") | (&Method::GET, "/index.html") => Ok(Response::new(INDEX.into())),
        (_, "/users") => {
            // look for the argument "?user_id=123" etc.
            let user_id: i32 = server::get_query_param(&req, "user_id")?;
            let user = User{id: user_id, name: "Some Body".to_string()};
            server::build_response_json(&user)
        },
        _ => { // Return 404 not found response.
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(NOTFOUND.into())
                .unwrap())
        }
    }
}



#[tokio::main]
async fn main() -> Result<(), HypErr> {
    
    let new_service = make_service_fn(move |conn: &AddrStream| {
        let remote_addr = conn.remote_addr();
        let ip_address = remote_addr.ip().to_string();
        async {
            Ok::<_, HypErr>(service_fn(move |req| {
                // Clone again to ensure everything you need outlives this closure.
                request_router(req, ip_address.to_owned())
            }))
        }
    });

    let bind_to = format!("0.0.0.0:8080").parse().unwrap();
    let server = Server::bind(&bind_to).serve(new_service);
    println!("Listening on http://{}", &bind_to);
    server.await?;
    Ok(())
}

