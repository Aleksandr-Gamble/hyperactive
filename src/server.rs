//! The server module makes sending responses slighly more ergonomic.


// standard library
use std::{fmt, collections::HashMap};
// crates.io
use url::Url;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use hyper::{header, body::Buf, Body, Request, Response, StatusCode};

const MSG_NOT_FOUND: &'static str = "ITEM NOT FOUND";
const APPLICATION_JSON: &'static str = "application/json";


/// Aggregate the body of a request in a buffer and deserialize it.
pub async fn get_payload<T: DeserializeOwned>(req: Request<Body>) -> Result<T, ServerError> {
	let whole_body = hyper::body::aggregate(req).await?;
	let req_payload: T =  serde_json::from_reader(whole_body.reader())?;
	Ok(req_payload)
}


/// Send a simple 200 status code response with a message as a string.
pub fn build_response_200_message(message: &str) -> Result<Response<Body>, ServerError> {
    let response = Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(message.to_string()))?;
    Ok(response)
}

/// Build a response out of any serializeable struct, adding the "application/json" header.
pub fn build_response_json<T: Serialize>(resp_payload: &T) -> Result<Response<Body>, ServerError> {
	let json = serde_json::to_string(&resp_payload)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, APPLICATION_JSON)
        .body(Body::from(json))?;
	Ok(response)
}


/// build a response out of any serializable struct, returning 404 if None was provided 
pub fn build_response_json_404<T: Serialize>(opt_payload: &Option<T>) -> Result<Response<Body>, ServerError> {
    match opt_payload {
        Some(resp_payload) => build_response_json(&resp_payload),
        None => {
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header(header::CONTENT_TYPE, APPLICATION_JSON)
                .body(Body::from(MSG_NOT_FOUND.to_string()))?;
            Ok(response)
        }
    }
}


/// Build a response out of any serializeable struct, adding the "application/json" and CORS "*" headers
pub fn build_response_json_cors<T: Serialize>(resp_payload: &T) -> Result<Response<Body>, ServerError> {
	let json = serde_json::to_string(&resp_payload)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, APPLICATION_JSON)
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
pub fn get_query_param<T: std::str::FromStr>(req: &Request<Body>, key: &str) -> Result<T, ArgError> {
    let opt: Option<T> = get_query_opt_param(req, key)?;
    let val: T =  opt.ok_or(MissingArg{missing_key: key.to_string()})?;
    Ok(val)
}


/// Look for the value contained in a query parameter and convert it to an Opt<struct> implementing std::str::FromStr
/// # Examples:
/// ```
/// let page_no: Option<i32> = get_query_opt_param(&req, "page_no").await?;
/// ```
pub fn get_query_opt_param<T: std::str::FromStr>(req: &Request<Body>, key: &str) -> Result<Option<T>, MalformedArg> {
    let hm = get_query(req);
    let key_string = key.to_string();
    let s = match hm.get(&key_string) {
        Some(val) => val,
        None => return Ok(None)
    };
    let val = match T::from_str(&s) {
        Ok(x) => x,
        Err(_) => return Err(MalformedArg::new(key, &s, &std::any::type_name::<T>())),
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
pub async fn preflight_cors(req: Request<Body>) -> Result<Response<Body>, ServerError> {
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


/// This error captures several things that can go wrong when responding to a request 
#[derive(Debug)]
pub enum ServerError {
    Arg(ArgError),
    SerdeJSON(serde_json::Error),
    Hyper(hyper::Error),
    HyperHTTP(hyper::http::Error),
}

impl std::error::Error for ServerError {}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


impl From<ArgError> for ServerError {
    fn from(err: ArgError) -> Self {
        ServerError::Arg(err)
    }
}



impl From<serde_json::Error> for ServerError {
    fn from(err: serde_json::Error) -> Self {
        ServerError::SerdeJSON(err)
    }
}

impl From<hyper::Error> for ServerError {
    fn from(err: hyper::Error) -> Self {
        ServerError::Hyper(err)
    }
}

impl From<hyper::http::Error> for ServerError {
    fn from(err: hyper::http::Error) -> Self {
        ServerError::HyperHTTP(err)
    }
}


/// This error captures when the item you want to return cannot be found in a database  
#[derive(Debug)]
pub struct PK404 {}

impl std::error::Error for PK404 {}

impl fmt::Display for PK404 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PK404 error: Primary Key not found")
    }
}


/// The MissingArg error indicates that a required url argument (i.e. "&key=val" etc.) was not
/// provided 
#[derive(Debug)]
pub struct MissingArg {
    /// This field captures the key that was missing
    pub missing_key: String,
}       
     
impl std::error::Error for MissingArg {}

impl fmt::Display for MissingArg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Required argument '{}' not found", self.missing_key)
    }  
}




/// The MalformedArg error indicates that a required url argument (i.e. "&key=val" etc.) could not
/// be deserialized/converted to the desired type 
#[derive(Debug)]
pub struct MalformedArg {
    /// This is the key that was provided 
    pub key: String,
    /// the provided value for the key 
    pub value: String,
    /// this string indicates the type of value that was desired 
    pub dtype: String,
}       


impl MalformedArg {
    fn new(key: &str, value: &str, dtype: &str) -> Self {
        let key = key.to_string();
        let value = value.to_string();
        let dtype = dtype.to_string();
        MalformedArg{key, value, dtype}
    }
}


impl std::error::Error for MalformedArg {}

impl fmt::Display for MalformedArg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not convert value '{}' for key '{}' to {} type", self.value, self.key, self.dtype)
    }  
}


/// The ArgError error captures both MissingArg and MalformedArg variants 
#[derive(Debug)]
pub enum ArgError {
    Missing(MissingArg),
    Malformed(MalformedArg),
}


impl From<MissingArg> for ArgError {
    fn from(err: MissingArg) -> Self {
        ArgError::Missing(err)
    }
}

impl From<MalformedArg> for ArgError {
    fn from(err: MalformedArg) -> Self {
        ArgError::Malformed(err)
    }
}


impl std::error::Error for ArgError {} 

impl fmt::Display for ArgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArgError::Missing(err) => write!(f, "{}", &err),
            ArgError::Malformed(err) => write!(f, "{}", &err),
        }
    }
}




/// convert any error that can be displayed with Debug to a BAD_REQUEST response 
pub fn bad_request_resp<T: std::fmt::Debug>(err: &T) -> Result<Response<Body>, ServerError> {
    let response = Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from(format!("BAD_REQUEST: {:?}", &err)))?;
    Ok(response)
}


/// If you use nginx in a docker container, you have to explicitly set headers showing the
/// real IP address where requests are coming from. This is typically done via nginx.conf.
/// 
/// In the author's experience, adding 
/// proxy_set_header X-Forwarded-For $remote_addr only gave the docker IP, i.e. "172.69.59.58"
/// Whereas using this approach
/// proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
/// gave the (real_ip, docker_ip): i.e. "104.218.65.97, 172.69.59.58"
/// 
/// This function takes such a string and returns the Real-IP you probably 'want' if possible.
pub fn nginx_real_ip_only(ip_addresses: &str) -> Option<String> {
    let sp = ip_addresses.split(", ")
        .filter(|ip| !ip.starts_with("172."))
        .map(|ip| ip.to_string())
        .collect::<Vec<String>>();
    match sp.get(0) {
        Some(val) => Some(val.to_owned()),
        None => None
    }
}

/// this constant is used for an unknown ipv4 address but some downstream function expects a string
pub const UNKNOWN_IP: &'static str = "?.?.?.?";


/// This is a conveneint way for getting the ip address for an NGINX instance running in Docker
/// using the X-Forwarded-For header. See also the  nginx_real_ip_only method
pub fn nginx_get_ip(req: &Request<Body>) -> String {
    let ip_addresses = get_header(&req, "X-Forwarded-For").unwrap_or(UNKNOWN_IP.to_string());
    nginx_real_ip_only(&ip_addresses).unwrap_or(UNKNOWN_IP.to_string())
}
