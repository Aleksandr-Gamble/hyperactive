//! This module contains errors.
//! 

/// The GenericError encompasses almost every possible error type that could be passed.
/// Asynchronous functions that return Result<T, GenericError> can call other functions and use the "?" operator to return the Err() variant as needed.
 
pub type GenericError = Box<dyn std::error::Error + Send + Sync>;
