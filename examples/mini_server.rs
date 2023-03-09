use serde::Serialize;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyperactive::{err::GenericError, server};


static INDEX: &[u8] = b"Hello Rust -> Tokio -> Hyper -> Hyperactive !";
static NOTFOUND: &[u8] = b"Not Found";

#[derive(Serialize)]
struct User {
    id: i32,
    name: String,
}

async fn request_router(req: Request<Body>, _ip_address: String) -> Result<Response<Body>, GenericError> {
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
async fn main() -> Result<(), GenericError> {
    
    let new_service = make_service_fn(move |conn: &AddrStream| {
        let remote_addr = conn.remote_addr();
        let ip_address = remote_addr.ip().to_string();
        async {
            Ok::<_, GenericError>(service_fn(move |req| {
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

