use crate::token::Token;
use std::fmt;
#[derive(Debug, Clone)]
pub enum Error {
    RuntimeError(RuntimeError),
    ReturnError(ReturnError),
}

/// Runtime error type
#[derive(Debug, Clone)]
pub struct ReturnError {
    pub value: Option<crate::token::Value>,
}
impl ReturnError {
    pub fn new(value: Option<crate::token::Value>) -> Self {
        Self { value }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

impl RuntimeError {
    pub fn new(token: Token, message: String) -> Self {
        Self { token, message }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[line {}] Runtime Error: {}", self.token.line, self.message)
    }
}

pub type Result<T> = std::result::Result<T, Error>;