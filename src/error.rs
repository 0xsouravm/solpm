//! # Error Handling Module
//!
//! This module defines the error types used throughout the Solana Program Manager.
//! It provides a comprehensive error system that covers:
//!
//! - File I/O errors
//! - Network and HTTP request failures
//! - Configuration and validation errors
//! - Authentication and authorization issues
//! - IDL parsing and validation errors
//! - Registry communication errors
//!
//! All errors implement standard Rust error traits and provide meaningful
//! error messages to help users diagnose and resolve issues.

use std::fmt;

/// Error types for Solana Package Manager operations.
/// 
/// This enum represents all possible error conditions that can occur
/// during solpm operations, from file I/O to network requests
/// to configuration issues.
#[derive(Debug)]
pub enum SolanaPmError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Http(reqwest::Error),
    ConfigNotFound(String),
    ProgramNotFound(String),
    InvalidIdl(String),
    UploadFailed(String),
    InvalidPath(String),
    DataMissing(String),
}

/// Implements Display for SolanaPmError to provide human-readable error messages.
/// 
/// Each error variant is formatted with appropriate context and suggestions
/// for the user to understand and resolve the issue.
impl fmt::Display for SolanaPmError {
    /// Formats the error for display to the user.
    /// 
    /// # Arguments
    /// 
    /// * `f` - The formatter to write to
    /// 
    /// # Returns
    /// 
    /// Returns a formatting result with user-friendly error messages.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SolanaPmError::Io(err) => write!(f, "IO error: {}", err),
            SolanaPmError::Json(err) => write!(f, "JSON parsing error: {}", err),
            SolanaPmError::Http(err) => write!(f, "HTTP request error: {}", err),
            SolanaPmError::ConfigNotFound(msg) => write!(f, "Configuration error: {}", msg),
            SolanaPmError::ProgramNotFound(name) => write!(f, "Program '{}' not found in registry", name),
            SolanaPmError::InvalidIdl(msg) => write!(f, "Invalid IDL: {}", msg),
            SolanaPmError::UploadFailed(msg) => write!(f, "Upload failed: {}", msg),
            SolanaPmError::InvalidPath(msg) => write!(f, "Invalid path: {}", msg),
            SolanaPmError::DataMissing(msg) => write!(f, "Data missing: {}", msg),
        }
    }
}

/// Implements the standard Error trait for SolanaPmError.
/// 
/// This allows SolanaPmError to be used with the standard error handling
/// infrastructure and error propagation mechanisms.
impl std::error::Error for SolanaPmError {}

/// Converts std::io::Error to SolanaPmError.
/// 
/// Enables automatic conversion using the `?` operator for file operations.
impl From<std::io::Error> for SolanaPmError {
    /// Converts a standard I/O error into a SolanaPmError.
    /// 
    /// # Arguments
    /// 
    /// * `err` - The I/O error to convert
    /// 
    /// # Returns
    /// 
    /// Returns a SolanaPmError::Io variant wrapping the original error.
    fn from(err: std::io::Error) -> Self {
        SolanaPmError::Io(err)
    }
}

/// Converts serde_json::Error to SolanaPmError.
/// 
/// Enables automatic conversion using the `?` operator for JSON operations.
impl From<serde_json::Error> for SolanaPmError {
    /// Converts a JSON parsing error into a SolanaPmError.
    /// 
    /// # Arguments
    /// 
    /// * `err` - The JSON error to convert
    /// 
    /// # Returns
    /// 
    /// Returns a SolanaPmError::Json variant wrapping the original error.
    fn from(err: serde_json::Error) -> Self {
        SolanaPmError::Json(err)
    }
}

/// Converts reqwest::Error to SolanaPmError.
/// 
/// Enables automatic conversion using the `?` operator for HTTP operations.
impl From<reqwest::Error> for SolanaPmError {
    /// Converts an HTTP request error into a SolanaPmError.
    /// 
    /// # Arguments
    /// 
    /// * `err` - The reqwest error to convert
    /// 
    /// # Returns
    /// 
    /// Returns a SolanaPmError::Http variant wrapping the original error.
    fn from(err: reqwest::Error) -> Self {
        SolanaPmError::Http(err)
    }
}

/// A type alias for Result with SolanaPmError as the error type.
/// 
/// This simplifies function signatures throughout the codebase by providing
/// a consistent Result type that uses SolanaPmError for all error cases.
/// 
/// # Examples
/// 
/// ```rust
/// fn my_function() -> Result<String> {
///     // Returns Result<String, SolanaPmError>
///     Ok("success".to_string())
/// }
/// ```
pub type Result<T> = std::result::Result<T, SolanaPmError>;