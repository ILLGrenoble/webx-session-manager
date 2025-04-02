use nix::unistd::User;
use uuid::Uuid;

use crate::{
    authentication::{Authenticator, Credentials},
    common::{Account, ApplicationError, Session, ScreenResolution},
};

use super::XorgService;

/// The `SessionService` struct provides functionality for managing user sessions,
/// including creating, retrieving, and terminating sessions.
pub struct SessionService {
    authenticator: Authenticator,
    xorg_service: XorgService,
}

impl SessionService {
    /// Creates a new `SessionService` instance.
    ///
    /// # Arguments
    /// * `authenticator` - The authenticator for user authentication.
    /// * `xorg_service` - The Xorg service for managing Xorg sessions.
    ///
    /// # Returns
    /// A new `SessionService` instance.
    pub fn new(authenticator: Authenticator, 
               xorg_service: XorgService
    ) -> Self {
        Self {
            authenticator,
            xorg_service,
        }
    }

    /// Creates a new session for a user.
    ///
    /// # Arguments
    /// * `credentials` - The user's credentials.
    /// * `resolution` - The screen resolution for the session.
    ///
    /// # Returns
    /// A `Result` containing the created `Session` or an `ApplicationError`.
    pub fn create_session(&self, credentials: &Credentials, resolution: ScreenResolution) -> Result<Session, ApplicationError> {
        return match self.authenticator.authenticate(credentials) {
            Ok(environment) => {
                debug!("Successfully authenticated user: {}", &credentials.username());
                if let Ok(Some(user)) = User::from_name(credentials.username()) {
                    debug!("Found user: {}", &credentials.username());
                    if let Some(account) = Account::from_user(user) {

                        // if the user already has an x session running then exit early...
                        if let Some(session) = self.xorg_service.get_session_for_user(account.uid()) {
                            debug!("User {} already has a session {}", &credentials.username(), session.id());
                            return Ok(session);
                        }

                        let webx_user = User::from_name("webx").unwrap().unwrap();
                        // create the necessary configuration files
                        if let Err(error) = self.xorg_service.create_user_files(&account, &webx_user) {
                            return Err(ApplicationError::session(format!("Error occurred setting up the configuration for a session {}", error)));
                        }

                        // finally, let's launch the x server...
                        return self.xorg_service.execute(&account, &webx_user, resolution, environment);
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

    /// Retrieves all active sessions.
    ///
    /// # Returns
    /// An `Option` containing a vector of `Session` instances, or `None` if no sessions are found.
    pub fn get_all(&self) -> Option<Vec<Session>> {
        if let Some(sessions) = self.xorg_service.get_all_sessions() {
            debug!("Found sessions: {:?}", sessions.len());
            return Some(sessions);
        }

        None
    }

    /// Terminates a session by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the session to terminate.
    ///
    /// # Returns
    /// A `Result` indicating success or an `ApplicationError`.
    pub fn kill_by_id(&self, id: Uuid) -> Result<(), ApplicationError> {
        if let Some(session ) = self.xorg_service.get_by_id(&id) {
            // kill the processes
            // the session will be automatically removed by the clean up procedure
            session.window_manager().kill()?;
            session.xorg().kill()?;
            return Ok(());
        }
        Err(ApplicationError::session(format!("Session {} not found", id)))
    }

    /// Terminates all active sessions.
    ///
    /// # Returns
    /// A `Result` indicating success or an `ApplicationError`.
    pub fn kill_all(&self) -> Result<(), ApplicationError> {
        if let Some(sessions) = self.xorg_service.get_all_sessions() {
            for session in sessions {
                session.window_manager().kill()?;
                session.xorg().kill()?;
            } 
        }
        Ok(())
    }

    /// Cleans up zombie sessions by removing sessions whose processes are no longer running.
    pub fn clean_up(&self) {
        if self.xorg_service.clean_up() > 0 {
            info!("Cleaned up {} zombie sessions", self.xorg_service.clean_up());
        }
    }
}
