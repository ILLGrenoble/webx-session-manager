use pam_client::{Context, Flag};
use pam_client::conv_mock::Conversation;
use pam_client::env_list::EnvList;

use crate::common::{ApplicationError};
use crate::authentication::Credentials;

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
        let conversation = Conversation::with_credentials(
            credentials.username(),
            credentials.password(),
        );
        let mut context = Context::new(
            self.service.as_str(),
            None,
            conversation,
        )?;

        context.authenticate(Flag::NONE)?;
        context.acct_mgmt(Flag::NONE)?;
        let session = context.open_session(Flag::NONE)?;
        Ok(session.envlist())
    }
}