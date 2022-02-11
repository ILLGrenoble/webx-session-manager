use nix::unistd::User;

use crate::{
    authentication::{Authenticator, Credentials},
    common::{Account, ApplicationError, Session, ScreenResolution},
};

use super::XorgService;

pub struct SessionService {
    authenticator: Authenticator,
    xorg_service: XorgService,
}

impl SessionService {
    pub fn new(authenticator: Authenticator, xorg_service: XorgService) -> Self {
        Self {
            authenticator,
            xorg_service,
        }
    }

    /// create a new session for the user
    pub fn create_session(&self, credentials: &Credentials, resolution: ScreenResolution) -> Result<Session, ApplicationError> {
        return match self.authenticator.authenticate(credentials) {
            Ok(_) => {
                debug!("Successfully authenticated user: {}", &credentials.username());
                if let Ok(Some(user)) = User::from_name(credentials.username()) {
                    debug!("Found user: {}", &credentials.username());
                    if let Some(account) = Account::from_user(user) {

                        // if the user already has a x session running then exit early...
                        if let Some(session) = self.xorg_service.get_session_for_user(account.uid()) {
                            debug!("User {} already has a session {}", &credentials.username(), session.id());
                            return Ok(session);
                        }

                        // create the necessary configuration files
                        if let Err(error) = self.xorg_service.create_user_files(&account) {
                            return Err(ApplicationError::session(format!("Error occurred setting up the configuration for a session {}", error)));
                        }

                        // finally, let's launch the x server...
                        return self.xorg_service.execute(&account, resolution);
                    }
                    return Err(ApplicationError::session(format!("User {} is invalid. check they have a home directory?", credentials.username())));
                }
                Err(ApplicationError::session(format!("Could not find user {}", credentials.username())))
            }
            Err(error) => {
                Err(ApplicationError::session(format!("Error authenticating user {}", error)))
            }
        }
    }

    /// get all sessions
    pub fn get_all(&self) -> Option<Vec<Session>> {
        if let Some(sessions) = self.xorg_service.get_all_sessions() {
            debug!("found sessions: {:?}", sessions.len());
            return Some(sessions);
        }

        None
    }

    // clean up zombie session
    pub fn clean_up(&self) {
        if self.xorg_service.clean_up() > 0 {
            info!("Cleaned up {} zombie sessions", self.xorg_service.clean_up());
        }
    }
}
