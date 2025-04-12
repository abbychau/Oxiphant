use std::io;
use thiserror::Error;

use crate::ast::Location;

/// Result type for the compiler
pub type Result<T> = std::result::Result<T, CompilerError>;

/// Compiler error types
#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("Lexical error at {0}: {1}")]
    LexicalError(Location, String),

    #[error("Syntax error at {0}: {1}")]
    SyntaxError(Location, String),

    #[error("Type error at {0}: {1}")]
    TypeError(Location, String),

    #[error("Code generation error: {message}")]
    CodeGenError { message: String },

    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
}

/// Create a lexical error
pub fn lexical_error(location: &Location, message: impl Into<String>) -> CompilerError {
    CompilerError::LexicalError(location.clone(), message.into())
}

/// Create a syntax error
pub fn syntax_error(location: &Location, message: impl Into<String>) -> CompilerError {
    CompilerError::SyntaxError(location.clone(), message.into())
}

/// Create a type error
pub fn type_error(location: &Location, message: impl Into<String>) -> CompilerError {
    CompilerError::TypeError(location.clone(), message.into())
}
