use std::{fmt, io};

use config::ConfigError;

/// The `ApplicationError` struct represents an error that can occur in the WebX Session Manager.
#[derive(Clone, Debug)]
pub struct ApplicationError {
    message: String,
    kind: ApplicationErrorKind,
}

/// The `ApplicationErrorKind` enum categorizes the types of errors that can occur.
#[derive(Clone, Copy, Debug)]
pub enum ApplicationErrorKind {
    Configuration,
    Authentication,
    Environment,
    Transport,
    Session,
}

impl ApplicationError {
    /// Creates a new `ApplicationError` with the specified message and kind.
    ///
    /// # Arguments
    /// * `message` - The error message.
    /// * `kind` - The kind of error.
    ///
    /// # Returns
    /// A new `ApplicationError` instance.
    fn new(message: impl AsRef<str>, kind: ApplicationErrorKind) -> Self {
        Self {
            message: message.as_ref().to_string(),
            kind,
        }
    }

    /// Creates an authentication error.
    ///
    /// # Arguments
    /// * `explanation` - The explanation for the error.
    ///
    /// # Returns
    /// An `ApplicationError` instance.
    pub fn authentication(explanation: impl AsRef<str>) -> Self {
        Self::new(
            explanation,
            ApplicationErrorKind::Authentication,
        )
    }

    /// Creates an environment error.
    ///
    /// # Arguments
    /// * `explanation` - The explanation for the error.
    ///
    /// # Returns
    /// An `ApplicationError` instance.
    pub fn environment(explanation: impl AsRef<str>) -> Self {
        Self::new(
            explanation,
            ApplicationErrorKind::Environment,
        )
    }

    /// Creates a session error.
    ///
    /// # Arguments
    /// * `explanation` - The explanation for the error.
    ///
    /// # Returns
    /// An `ApplicationError` instance.
    pub fn session(explanation: impl AsRef<str>) -> Self {
        Self::new(
            explanation,
            ApplicationErrorKind::Session,
        )
    }

    /// Creates a transport error.
    ///
    /// # Arguments
    /// * `explanation` - The explanation for the error.
    ///
    /// # Returns
    /// An `ApplicationError` instance.
    pub fn transport(explanation: impl AsRef<str>) -> Self {
        Self::new(
            explanation,
            ApplicationErrorKind::Transport,
        )
    }

    /// Creates a configuration error.
    ///
    /// # Arguments
    /// * `explanation` - The explanation for the error.
    ///
    /// # Returns
    /// An `ApplicationError` instance.
    pub fn configuration(explanation: impl AsRef<str>) -> Self {
        Self::new(
            explanation,
            ApplicationErrorKind::Configuration,
        )
    }

    /// Returns the error message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ApplicationError {
    /// Formats the `ApplicationError` for display.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}; {}", self.kind, self.message)
    }
}

impl fmt::Display for ApplicationErrorKind {
    /// Formats the `ApplicationErrorKind` for display.
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