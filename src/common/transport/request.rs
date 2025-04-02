use serde::{Deserialize, Serialize};

/// The `Request` enum represents the possible requests that can be sent to the WebX Session Manager server.
/// Each variant corresponds to a specific type of request.
#[derive(Serialize, Deserialize)]
#[serde(tag = "request", content = "content")]
pub enum Request {
    /// A request to log in a user and create a new session.
    ///
    /// # Fields
    /// * `username` - The username of the user.
    /// * `password` - The password of the user.
    /// * `width` - The screen width for the session.
    /// * `height` - The screen height for the session.
    #[serde(rename = "login")]
    Login { username: String, password: String, width: u32, height: u32 },

    /// A request to list all active sessions.
    #[serde(rename = "who")]
    Who,

    /// A request to log out a user and terminate the session.
    ///
    /// # Fields
    /// * `id` - The session ID to terminate.
    #[serde(rename = "logout")]
    Logout { id: String },
}




