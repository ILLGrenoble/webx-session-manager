use std::{fmt, io};

use config::ConfigError;

#[derive(Clone, Debug)]
pub struct ApplicationError {
    message: String,
    kind: ApplicationErrorKind,
}


#[derive(Clone, Copy, Debug)]
pub enum ApplicationErrorKind {
    Configuration,
    Authentication,
    Environment,
    Transport,
    Session,
}

impl ApplicationError {
    fn new(message: impl AsRef<str>, kind: ApplicationErrorKind) -> Self {
        Self {
            message: message.as_ref().to_string(),
            kind,
        }
    }

    pub fn authentication(explanation: impl AsRef<str>) -> Self {
        Self::new(
            explanation,
            ApplicationErrorKind::Authentication,
        )
    }

    pub fn environment(explanation: impl AsRef<str>) -> Self {
        Self::new(
            explanation,
            ApplicationErrorKind::Environment,
        )
    }

    pub fn session(explanation: impl AsRef<str>) -> Self {
        Self::new(
            explanation,
            ApplicationErrorKind::Session,
        )
    }

    pub fn transport(explanation: impl AsRef<str>) -> Self {
        Self::new(
            explanation,
            ApplicationErrorKind::Transport,
        )
    }

    pub fn configuration(explanation: impl AsRef<str>) -> Self {
        Self::new(
            explanation,
            ApplicationErrorKind::Configuration,
        )
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}


impl fmt::Display for ApplicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}; {}", self.kind, self.message)
    }
}


impl fmt::Display for ApplicationErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            ApplicationErrorKind::Authentication => "invalid credentials or bad account",
            ApplicationErrorKind::Environment => "invalid environment",
            ApplicationErrorKind::Transport => "transport",
            ApplicationErrorKind::Session => "issue launching session",
            ApplicationErrorKind::Configuration => "configuration"
        };
        write!(f, "{}", string)
    }
}


impl From<pam_client::Error> for ApplicationError {
    fn from(error: pam_client::Error) -> Self {
        ApplicationError::authentication(format!("{}", error))
    }
}

impl From<io::Error> for ApplicationError {
    fn from(error: io::Error) -> Self {
        let message = format!("{}", error);
        ApplicationError::transport(message)
    }
}

impl From<zmq::Error> for ApplicationError {
    fn from(error: zmq::Error) -> Self {
        let message = format!("zeromq: {}", error);
        ApplicationError::transport(message)
    }
}


impl From<ConfigError> for ApplicationError {
    fn from(error: ConfigError) -> Self {
        let message = format!("{}", error);
        ApplicationError::configuration(message)
    }
}