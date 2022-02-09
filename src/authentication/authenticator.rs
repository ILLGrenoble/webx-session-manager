use pam_client::{Context, Flag};
use pam_client::conv_mock::Conversation;

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

    pub fn authenticate(&self, credentials: &Credentials) -> Result<(), ApplicationError> {
        let service = self.settings.service();
        debug!("Authenticating user {} for service {}", credentials.username(), service);
        let conversation =
            Conversation::with_credentials(credentials.username(), credentials.password());
        let mut context = Context::new(service, None, conversation)?;

        context.authenticate(Flag::NONE)?;
        let _ = context.open_session(Flag::NONE)?;
        Ok(())

    }
}
