use pam_client::env_list::EnvList;
use pam_client::{Context, Flag};
use pam_client::conv_mock::Conversation;

use crate::authentication::Credentials;
use crate::common::{ApplicationError};

/// The `Authenticator` struct provides functionality for authenticating users using PAM (Pluggable Authentication Modules).
pub struct Authenticator {
    service: String,
}

impl Authenticator {
    /// Creates a new `Authenticator` instance.
    ///
    /// # Arguments
    /// * `service` - The PAM service to use for authentication.
    ///
    /// # Returns
    /// A new `Authenticator` instance.
    pub fn new(service: String) -> Self {
        Self {
            service
        }
    }

    /// Authenticates a user using their credentials.
    ///
    /// # Arguments
    /// * `credentials` - The user's credentials (username and password).
    ///
    /// # Returns
    /// A `Result` containing an `EnvList` of environment variables if authentication succeeds,
    /// or an `ApplicationError` if authentication fails.
    pub fn authenticate(&self, credentials: &Credentials) -> Result<EnvList, ApplicationError> {
        let service = &self.service;
        debug!("Authenticating user {} for service {}", credentials.username(), service);
        let conversation =
            Conversation::with_credentials(credentials.username(), credentials.password());
        let mut context = Context::new(service, None, conversation)?;

        context.authenticate(Flag::NONE)?;
        let session = context.open_session(Flag::NONE)?;
        Ok(session.envlist())
    }
}
