use std::{error, fmt};

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.message)
    }
}

impl error::Error for Error {}
