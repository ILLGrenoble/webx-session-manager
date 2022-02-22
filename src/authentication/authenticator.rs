use pam_client::env_list::EnvList;
use pam_client::{Context, Flag};
use pam_client::conv_mock::Conversation;

use crate::authentication::Credentials;
use crate::common::{ApplicationError};

pub struct Authenticator {
    service: String,
}

impl Authenticator {
    pub fn new(service: String) -> Self {
        Self {
            service
        }
    }

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
