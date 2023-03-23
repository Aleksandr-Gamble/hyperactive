//! This module contains errors.
//! 


use std::{error::Error, fmt};
use serde::{Serialize, Deserialize};


/// The GenericError encompasses almost every possible error type that could be passed.
/// Asynchronous functions that return Result<T, GenericError> can call other functions and use the "?" operator to return the Err() variant as needed.
 
pub type GenericError = Box<dyn std::error::Error + Send + Sync>;


/// The SimpleError is easy to instantiate when a more specific error is not needed
#[derive(Debug, Serialize, Deserialize)]
pub struct SimpleError {
    // A very generic error. This is a bit of an antipattern,
    // but it is easier than creating a new error types for a hundred misc things
    pub message: String,
}

impl Error for SimpleError {}

impl fmt::Display for SimpleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SimpleError: {}", self.message)
    }
}

impl SimpleError {
    pub fn from_str(message: &str) -> Self {
        SimpleError{
            message: message.to_string()
        }
    }
}
