use std::{fmt, future::Future, marker::Sync, collections::HashMap};
use url::Url;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use hyper::{header, body::Buf, Body, Client, Method, Request, Response, Server, StatusCode};
use tokio_postgres::types::ToSql;
pub use radix::{core::GenericError, postgres::GenericClient};


/// this useful function aggregates and deserializes the payload of a request
async fn get_payload<T: DeserializeOwned>(req: Request<Body>) -> Result<T, GenericError>{
	let whole_body = hyper::body::aggregate(req).await?;
	let req_payload: T =  serde_json::from_reader(whole_body.reader())?;
	Ok(req_payload)
}


/// this useful function builds a HTTP response and adds headers from a serializable payload
pub fn build_response_json_cors<T: Serialize>(resp_payload: &T) -> Result<Response<Body>, GenericError> {
	let json = serde_json::to_string(&resp_payload)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, "application/json")
		.header("Access-Control-Allow-Origin", "*")
		.body(Body::from(json))?;
	Ok(response)
}


/// The switch_psql_handler is intended to help you NOT have to make a different HTTP endpoint
/// and associated hander method for every type of struct you want to pass over
/// For instance, if data_type_key="data_type" and pk_key="name", you could call the http endpoint
/// GET http://foo.bar.org/search?data_type=cities&name=richmond
/// Under the hood, the switcher method will use match on the provided data_type 
/// (which ="cities" in this example) and return a future of a Box of a list of cities,
/// Where the struct for one city must implement Serialize
/// voila!
/// NOTE: I do not fully understand why the 'a is needed for the client and nothing else
pub async fn switch_psql_handler<
    'a, GC: GenericClient+Sync, PK: ToSql+Sync+std::str::FromStr, T: Serialize
    >(req: Request<Body>, data_type_key: &'static str, pk_key: &'static str, client: &'a GC,
        switcher: fn(&str, &PK, &GC) -> std::pin::Pin<Box<dyn Future<Output=Result<T, GenericError>>>>
    ) -> Result<Response<Body>, GenericError>
{
    let data_type: String = get_query_param(&req, data_type_key)?;
    let pk: PK = get_query_param(&req, pk_key)?;
    let payload = switcher(&data_type, &pk, client).await?;
    build_response_json_cors(&payload)
}


/// CommonHeaders is intended to capture the headers you probably care the most about.
#[derive(Debug)]
pub struct CommonHeaders {
    pub user_agent: Option<String>,
    pub x_api_key: Option<String>,
}

/// This struct gets passed when the user provides simple text as a payload
#[derive(Deserialize)]
pub struct PhrasePayload {
    pub phrase: String
}

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


pub fn get_common_headers(req: &Request<Body>) -> CommonHeaders {
    let user_agent = get_header(req, "user-agent");
    let x_api_key = get_header(req, "X-Api-Key");
    CommonHeaders{user_agent: user_agent, x_api_key: x_api_key}
}


// Gather any query parameters (i.e. path?key1=val1&key2=val2 etc.) into a HashMap
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

/// This nifty function takes the request sent to HYPER, looks for a URL query value, and attempts to parse it to the desired type
/// It most often will be use for strings or i32 etc.
pub fn get_query_param<T: std::str::FromStr>(req: &Request<Body>, key: &str) -> Result<T, ErrHTTP> {
    let hm = get_query(req);
    let key_string = key.to_string();
    let s = match hm.get(&key_string) {
        Some(val) => val,
        None => return Err(ErrHTTP{message: format!("Specified URL parameter '{}' not found!", key)})
    };
    let val = match T::from_str(&s) {
        Ok(x) => x,
        Err(_) => return Err(ErrHTTP{message: format!("URL parameter value '{}' could not be converted to the specified type", &s)})
    };
    Ok(val)
}



/// this method returns CORS preflight headers
/// If you want to allow CORS, Google Chrome looks for headers on BOTH the request and the preflight
/// Consider routing like this:
/// (&Method::OPTIONS, _) => server_utils::preflight(req).await,
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
