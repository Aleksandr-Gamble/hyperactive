//! The server module makes sending responses slighly more ergonomic.


// standard library
use std::{fmt, collections::HashMap};
// crates.io
use url::Url;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use hyper::{header, body::Buf, Body, Request, Response, StatusCode};
// this crate
pub use crate::err::GenericError;


/// Aggregate the body of a request in a buffer and deserialize it.
pub async fn get_payload<T: DeserializeOwned>(req: Request<Body>) -> Result<T, GenericError>{
	let whole_body = hyper::body::aggregate(req).await?;
	let req_payload: T =  serde_json::from_reader(whole_body.reader())?;
	Ok(req_payload)
}


/// Send a simple 200 status code response with a message as a string.
pub fn build_response_200_message(message: &str) -> Result<Response<Body>, GenericError> {
    let response = Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(message.to_string()))?;
    Ok(response)
}

/// Build a response out of any serializeable struct, adding the "application/json" header.
pub fn build_response_json<T: Serialize>(resp_payload: &T) -> Result<Response<Body>, GenericError> {
	let json = serde_json::to_string(&resp_payload)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json))?;
	Ok(response)
}




/// Build a response out of any serializeable struct, adding the "application/json" and CORS "*" headers
pub fn build_response_json_cors<T: Serialize>(resp_payload: &T) -> Result<Response<Body>, GenericError> {
	let json = serde_json::to_string(&resp_payload)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .body(Body::from(json))?;
	Ok(response)
}



/// Look for the specified in a given request, returning Some(value) if it is present
pub fn get_header(req: &Request<Body>, header: &str) -> Option<String> {
    // Get a specific header from a request
    let header_lower = header.to_lowercase();
    for (k, v) in req.headers() {
        let key = k.as_str();
        if key != &header_lower {
            continue
        }
        match String::from_utf8(v.as_bytes().to_owned()) {
            Ok(val) => {
                if val.len() > 0 {
                    return Some(val)
                } 
            },
            Err(_) => return None
        }
    }
    // if you reach this point, you never found the header you were looking for
    return None
}

/// Return the CommonHeaders from a request
pub fn get_common_headers(req: &Request<Body>) -> CommonHeaders {
    let user_agent = get_header(req, "user-agent");
    let x_api_key = get_header(req, "X-Api-Key");
    let host = get_header(req, "Host");
    let accept = get_header(req, "Accept");
    CommonHeaders{user_agent, x_api_key, host, accept}
}


/// CommonHeaders is intended to capture the most frequently used request headers
#[derive(Debug, Serialize, Deserialize)]
pub struct CommonHeaders {
    pub user_agent: Option<String>,
    pub x_api_key: Option<String>,
    pub host: Option<String>,
    pub accept: Option<String>
}


/// Gather any query parameters (i.e. path?key1=val1&key2=val2 etc.) into a HashMap
pub fn get_query(req: &Request<Body>) -> HashMap<String, String> {
    let mut hm = HashMap::<String, String>::new();
    let mut url_str = req.uri().to_string();
    if url_str.starts_with("/") {
        url_str = format!("http://whatever.com{}", &url_str); // odd that it takes only the path?
    }
    let url = match Url::parse(&url_str) {
        Ok(val) => val,
        Err(_) => {// I don't think this should ever happen?
            println!("get_query failed to parse url {}", &url_str);
            return hm
        }
    };
    let pairs = url.query_pairs();
    for (k, v) in pairs {
        let key = k.to_string();
        let val = v.to_string();
        hm.insert(key, val);
    }
    hm
}


/// Look for the value contained in a query parameter and convert it to a struct implementing std::str::FromStr
/// # Examples:
/// ```
/// let user_id: i32 = get_query_param(&req, "user_id").await?;
/// ```
pub fn get_query_param<T: std::str::FromStr>(req: &Request<Body>, key: &str) -> Result<T, ErrHTTP> {
    let opt: Option<T> = get_query_opt_param(req, key)?;
    let val: T =  opt.ok_or(ErrHTTP{message: format!("Specified URL parameter '{}' not found!", key)})?;
    Ok(val)
}


/// Look for the value contained in a query parameter and convert it to an Opt<struct> implementing std::str::FromStr
/// # Examples:
/// ```
/// let page_no: Option<i32> = get_query_opt_param(&req, "page_no").await?;
/// ```
pub fn get_query_opt_param<T: std::str::FromStr>(req: &Request<Body>, key: &str) -> Result<Option<T>, ErrHTTP> {
    let hm = get_query(req);
    let key_string = key.to_string();
    let s = match hm.get(&key_string) {
        Some(val) => val,
        None => return Ok(None)
    };
    let val = match T::from_str(&s) {
        Ok(x) => x,
        Err(_) => return Err(ErrHTTP{message: format!("URL parameter value '{}' could not be converted to the specified type", &s)})
    };
    Ok(Some(val))
}


/// Apply CORS preflight headers.  
/// If you want to allow CORS, Google Chrome looks for headers on BOTH the request and the preflight.  
/// # Examples:
/// ```
/// // Consider routing like this withing a server block
/// match (req.method(), req.uri().path()) {
///     (&Method::OPTIONS, _) => preflight(req).await,
///     _ => build_response_200_message("success").await,
/// }
/// ```
pub async fn preflight_cors(req: Request<Body>) -> Result<Response<Body>, GenericError> {
    let _whole_body = hyper::body::aggregate(req).await?;
    let response = Response::builder()
        .status(StatusCode::OK)
        //.header(header::CONTENT_TYPE, "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Headers", "*")
        .header("Access-Control-Allow-Methods", "POST, GET, OPTIONS")
        .body(Body::default())?;
    Ok(response)
}


/// This error represents something went wrong processing an http request.  

#[derive(Debug)]
pub struct ErrHTTP {
    // A very generic error.
    pub message: String,
}

impl std::error::Error for ErrHTTP {}

impl fmt::Display for ErrHTTP {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ErrHTTP: {}", self.message)
    }
}
