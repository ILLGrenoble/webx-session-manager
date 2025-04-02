use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::Session;

/// The `SessionDto` struct represents a data transfer object for a user session.
/// It contains details about the session, such as the user, session ID, and process IDs.
#[derive(Serialize, Deserialize, Clone)]
pub struct SessionDto {
    id: String,
    username: String,
    uid: u32,
    display_id: String,
    xorg_process_id: u32,
    window_manager_process_id: u32,
    xauthority_file_path: String,
    width: u32,
    height: u32
}

#[allow(dead_code)]
impl SessionDto {
    /// Creates a new `SessionDto` instance.
    ///
    /// # Arguments
    /// * `id` - The session ID.
    /// * `username` - The username of the session owner.
    /// * `uid` - The user ID of the session owner.
    /// * `display_id` - The X11 display ID.
    /// * `xorg_process_id` - The process ID of the Xorg server.
    /// * `window_manager_process_id` - The process ID of the window manager.
    /// * `xauthority_file_path` - The path to the Xauthority file.
    /// * `width` - The screen width.
    /// * `height` - The screen height.
    ///
    /// # Returns
    /// A new `SessionDto` instance.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        username: String,
        uid: u32,
        display_id: String,
        xorg_process_id: u32,
        window_manager_process_id: u32,
        xauthority_file_path: String,
        width: u32,
        height: u32
    ) -> Self {
        Self {
            id,
            username,
            uid,
            display_id,
            xorg_process_id,
            window_manager_process_id,
            xauthority_file_path,
            width,
            height
        }
    }

    /// Returns the session ID.
    pub fn id(&self) -> &str {
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

    /// Returns the process ID of the Xorg server.
    pub fn xorg_process_id(&self) -> u32 {
        self.xorg_process_id
    }

    /// Returns the process ID of the window manager.
    pub fn window_manager_process_id(&self) -> u32 {
        self.window_manager_process_id
    }

    /// Returns the path to the Xauthority file.
    pub fn xauthority_file_path(&self) -> &str {
        &self.xauthority_file_path
    }

    /// Returns the screen resolution as a string in the format "widthxheight".
    pub fn resolution(&self) -> String {
        format!("{}x{}", self.width, self.height)
    }
}

impl fmt::Display for SessionDto {
    /// Formats the `SessionDto` for display.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionDto")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("uid", &self.uid)
            .field("display_id", &self.display_id)
            .field("xorg_process_id", &self.xorg_process_id)
            .field("window_manager_process_id", &self.window_manager_process_id)
            .field("xauthority_file_path", &self.xauthority_file_path)
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl From<&Session> for SessionDto {
    /// Converts a `Session` into a `SessionDto`.
    ///
    /// # Arguments
    /// * `session` - The `Session` to convert.
    ///
    /// # Returns
    /// A `SessionDto` instance.
    fn from(session: &Session) -> Self {
        let username = session.username();
        let uid = session.uid();
        let display_id = session.display_id();
        let xorg_process_id = session.xorg().pid();
        let window_manager_process_id = session.window_manager().pid();
        let xauthority_file_path = session.xauthority_file_path();
        let id  = session.id().simple();
        let (width, height) = session.resolution().split();
        SessionDto::new(
            id.to_string(),
            username.into(),
            uid,
            display_id.into(),
            xorg_process_id,
            window_manager_process_id,
            xauthority_file_path.into(),
            width,
            height
        )
    }
}
