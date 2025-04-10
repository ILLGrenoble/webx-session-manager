use std::fmt;

use uuid::Uuid;

use crate::common::ProcessHandle;

use super::ScreenResolution;

/// The `Session` struct represents a user session managed by the WebX Session Manager.
/// It contains details about the session, such as the user, session ID, the Xorg process and the Window Manager process.
#[derive(Clone)]
pub struct Session {
    id: Uuid,
    username: String,
    uid: u32,
    display_id: String,
    xauthority_file_path: String,
    xorg: ProcessHandle,
    window_manager: ProcessHandle,
    resolution: ScreenResolution,
}

#[allow(dead_code)]
impl Session {
    /// Creates a new `Session` instance.
    ///
    /// # Arguments
    /// * `id` - The unique identifier for the session.
    /// * `username` - The username of the session owner.
    /// * `uid` - The user ID of the session owner.
    /// * `display_id` - The X11 display ID.
    /// * `xauthority_file_path` - The path to the Xauthority file.
    /// * `xorg` - The process handle for the Xorg server.
    /// * `window_manager` - The process handle for the window manager.
    /// * `resolution` - The screen resolution for the session.
    ///
    /// # Returns
    /// A new `Session` instance.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        username: String,
        uid: u32,
        display_id: String,
        xauthority_file_path: String,
        xorg: ProcessHandle,
        window_manager: ProcessHandle,
        resolution: ScreenResolution,
    ) -> Self {
        Self {
            id,
            username,
            uid,
            display_id,
            xauthority_file_path,
            xorg,
            window_manager,
            resolution,
        }
    }

    /// Returns the unique identifier for the session.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Returns the username of the session owner.
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns the user ID of the session owner.
    pub fn uid(&self) -> u32 {
        self.uid
    }

    /// Returns the X11 display ID.
    pub fn display_id(&self) -> &str {
        &self.display_id
    }

    /// Returns the path to the Xauthority file.
    pub fn xauthority_file_path(&self) -> &str {
        &self.xauthority_file_path
    }

    /// Returns the process handle for the Xorg server.
    pub fn xorg(&self) -> &ProcessHandle {
        &self.xorg
    }

    /// Returns the process handle for the window manager.
    pub fn window_manager(&self) -> &ProcessHandle {
        &self.window_manager
    }

    /// Returns the screen resolution for the session.
    pub fn resolution(&self) -> &ScreenResolution {
        &self.resolution
    }
}

impl fmt::Display for Session {
    /// Formats the `Session` for display.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Session")
            .field("username", &self.username)
            .field("uid", &self.uid)
            .field("display_id", &self.display_id)
            .field("xauthority_file_path", &self.xauthority_file_path)
            .field("resolution", &format!("{}", &self.resolution))
            .field("xorg pid", &self.xorg.pid())
            .field("window_manager pid", &self.window_manager.pid())
            .finish()
    }
}
