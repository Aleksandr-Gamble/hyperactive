//! The server module makes sending responses slighly more ergonomic.


// standard library
use std::fmt;
// crates.io



/// This error captures a missing api key or incorrect api_key
#[derive(Debug)]
pub enum ApiKeyError {
    /// Return this variant when you expected an environment variable to be set for the API key,
    /// but it was not 
    MissingEnv(String),
    /// Reuturn this variant when the provided api key was rejected 
    Rejected(String),
}


/// This error captures several things that can go wrong when responding to a request 
#[derive(Debug)]
pub enum HypErr {
    ApiKey(ApiKeyError),
    Arg(ArgError),
    SerdeJSON(serde_json::Error),
    Hyper(hyper::Error),
    HyperHTTP(hyper::http::Error),
}

impl std::error::Error for HypErr {}

impl fmt::Display for HypErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


impl From<ArgError> for HypErr {
    fn from(err: ArgError) -> Self {
        HypErr::Arg(err)
    }
}

impl From<ApiKeyError> for HypErr {
    fn from(err: ApiKeyError) -> Self {
        HypErr::ApiKey(err)
    }
}

impl From<MalformedArg> for HypErr {
    fn from(err: MalformedArg) -> Self {
        let argerr = ArgError::from(err);
        HypErr::from(argerr)
    }
}


impl From<MissingArg> for HypErr  {
    fn from(err: MissingArg) -> Self {
        let argerr = ArgError::from(err);
        HypErr::from(argerr)
    }
}


impl From<serde_json::Error> for HypErr {
    fn from(err: serde_json::Error) -> Self {
        HypErr::SerdeJSON(err)
    }
}

impl From<hyper::Error> for HypErr {
    fn from(err: hyper::Error) -> Self {
        HypErr::Hyper(err)
    }
}

impl From<hyper::http::Error> for HypErr {
    fn from(err: hyper::http::Error) -> Self {
        HypErr::HyperHTTP(err)
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
    pub fn new(key: &str, value: &str, dtype: &str) -> Self {
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

