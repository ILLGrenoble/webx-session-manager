use std::{fmt, io};
use std::error::Error;

#[derive(Debug)]
pub enum ApplicationError {
    AuthenticationFailed(String),
    Environment(String),
    Command(String),
}

impl Error for ApplicationError {}

impl fmt::Display for ApplicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApplicationError::AuthenticationFailed(message) => write!(formatter, "{}", message),
            ApplicationError::Environment(message) => write!(formatter, "{}", message),
            ApplicationError::Command(message) => write!(formatter, "{}", message)
        }
    }
}

impl From<pam_client::Error> for ApplicationError {
    fn from(error: pam_client::Error) -> Self {
        ApplicationError::AuthenticationFailed(error.to_string())
    }
}

impl From<io::Error> for ApplicationError {
    fn from(error: io::Error) -> Self {
        ApplicationError::Command(error.to_string())
    }
}