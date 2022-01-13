use pam_client::conv_mock::Conversation;
use pam_client::env_list::EnvList;
use pam_client::{Context, Flag};

use crate::authentication::Credentials;
use crate::common::{ApplicationError, AuthenticationSettings};

pub struct Authenticator {
    settings: AuthenticationSettings,
}

impl Authenticator {
    pub fn new(settings: AuthenticationSettings) -> Self {
        Self {
            settings
        }
    }
  
    pub fn authenticate(&self, credentials: &Credentials) -> Result<EnvList, ApplicationError> {
        let conversation =
            Conversation::with_credentials(credentials.username(), credentials.password());
        let mut context = Context::new(self.settings.service(), None, conversation)?;

        context.authenticate(Flag::NONE)?;
        context.acct_mgmt(Flag::NONE)?;
        let session = context.open_session(Flag::NONE)?;
        Ok(session.envlist())
    }
}
