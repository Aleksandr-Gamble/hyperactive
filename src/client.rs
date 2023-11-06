//! The client module makes making http requests slighlty more ergonomic. 


// standard library
use std::{env};
// crates.io
use serde::{self, Serialize, de::DeserializeOwned};
use serde_json;
use hyper::body; // brings the to_bytes() method into scope:
use hyper::{Request, Body, Method, Client};
// this crate 
use crate::err::HypErr;

// return the value of the environment variable X_API_KEY
fn get_api_key(optkey: Option<&str>) -> String {
    match optkey {
        Some(key) => key.to_string(),
        None => match env::var("X_API_KEY") {
            Ok(key) => key,
            Err(_) => String::new(),
        }
    }
}

/// Let T be any struct implementing serde::de::DeserializeOwned.  
/// You can make an API call to get that struct using this get function.  
/// An optional X-Api-Key can be provided using optkey.  
/// If optkey is none, it will look for the environment variable X_API_KEY.  
pub async fn get<T: DeserializeOwned>(url: &str, optkey: Option<&str>) -> Result<T, HypErr> {
    let x_api_key = get_api_key(optkey);
    let request = Request::builder()
        .method(Method::GET)
        .uri(url)
        .header("accept", "application/json")
        .header("X-Api-Key", x_api_key)
        .body(Body::empty())?;
    let client = Client::new();
    let resp = client.request(request).await?;
    let bytes = body::to_bytes(resp.into_body()).await?;
    let foo = serde_json::from_slice::<T>(&bytes)?;
    Ok(foo)
}



/// Let U be any struct implementing serde::Serialize.  
/// Let T be any struct implementing serde::de::DeserializeOwned.  
/// This function makes it ergonomic to send U and get T back.  
/// To set the X-Api-Key header, pass a Some() variant of a string slice to the optkey argument.  
/// If optkey is None, the request will use the environment variable X_API_KEY to set the X-Api-Key header,
/// defaulting to "" if the X_API_KEY is not defined. 
pub async fn post<U: Serialize, T: DeserializeOwned>(url: &str, payload: &U, optkey: Option<&str>) -> Result<T, HypErr> {
    let body_string = serde_json::to_string(payload)?;
    let x_api_key = get_api_key(optkey);
    let request = Request::builder()
        .method(Method::POST)
        .uri(url)
        .header("accept", "application/json")
        .header("X-Api-Key", x_api_key)
        // IF YOU DON'T INCLUDE THIS HEADER, ONLY THE FIRST PROPERTY OF THE STRUCT GETS RETURNED???
        .header("Content-type", "application/json; charset=UTF-8")
        .body(Body::from(body_string))?;
    let client = Client::new();
    let resp = client.request(request).await?;
    let bytes = body::to_bytes(resp.into_body()).await?;
    //println!("   DEV_98Mi9 GOT BYTES: {}", std::str::from_utf8(&bytes).unwrap() );
    let foo = serde_json::from_slice::<T>(&bytes)?;
    Ok(foo)
}

/// Let U be any struct implementing serde::Serialize.  
/// This function makes it ergonomic to send U, expecting no struct back.  
/// To set the X-Api-Key header, pass a Some() variant of a string slice to the optkey argument.  
/// If optkey is None, the request will use the environment variable X_API_KEY to set the X-Api-Key header,
/// defaulting to "" if the X_API_KEY is not defined. 
pub async fn post_noback<U: Serialize>(url: &str, payload: &U, optkey: Option<&str>) -> Result<(), HypErr> {
    let body_string = serde_json::to_string(payload)?;
    let x_api_key = get_api_key(optkey);
    let request = Request::builder()
        .method(Method::POST)
        .uri(url)
        .header("accept", "application/json")
        .header("X-Api-Key", x_api_key)
        // IF YOU DON'T INCLUDE THIS HEADER, ONLY THE FIRST PROPERTY OF THE STRUCT GETS RETURNED???
        .header("Content-type", "application/json; charset=UTF-8")
        .body(Body::from(body_string))?;
    let client = Client::new();
    let _resp = client.request(request).await?;
    Ok(())
}


/// Let T be any struct implementing serde::de::DeserializeOwned.  
/// you can make an API call to put to make a PUT request returning the specified struct.  
/// To set the X-Api-Key header, pass a Some() variant of a string slice to the optkey argument.  
/// If optkey is None, the request will use the environment variable X_API_KEY to set the X-Api-Key header,
/// defaulting to "" if the X_API_KEY is not defined. 
pub async fn put<T: DeserializeOwned>(url: &str, optkey: Option<&str>) -> Result<T, HypErr> {
    let x_api_key = get_api_key(optkey);
    let request = Request::builder()
        .method(Method::PUT)
        .uri(url)
        .header("accept", "application/json")
        .header("X-Api-Key", x_api_key)
        .body(Body::empty())?;
    let client = Client::new();
    let resp = client.request(request).await?;
    let bytes = body::to_bytes(resp.into_body()).await?;
    let foo = serde_json::from_slice::<T>(&bytes)?;
    Ok(foo)
}

