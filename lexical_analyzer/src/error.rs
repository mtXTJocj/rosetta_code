use error::Error;
use std::boxed::Box;
use std::{error, fmt, result};

pub type Result<T> = result::Result<T, CompileError>;

#[derive(Debug)]
pub enum ErrorKind {
    ReadError,
    LexicalAnalyzerError,
    ParseError,
}

#[derive(Debug)]
pub struct CompileError {
    kind: ErrorKind,
    repr: Box<dyn Error + Send + Sync>,
}

impl CompileError {
    pub fn new<E>(kind: ErrorKind, error: E) -> Self
    where
        E: Into<Box<dyn Error + Send + Sync>>,
    {
        CompileError {
            kind,
            repr: error.into(),
        }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}, {}", self.kind, self.repr)
    }
}

impl error::Error for CompileError {
    fn description(&self) -> &str {
        "compiler error"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}
