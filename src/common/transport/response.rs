use serde::{Deserialize, Serialize};

use crate::dto::SessionDto;

/// The `Response` enum represents the possible responses sent by the WebX Session Manager server.
/// Each variant corresponds to a specific type of response.
#[derive(Serialize, Deserialize)]
#[serde(tag = "response", content = "content")]
pub enum Response {
    /// A response indicating a successful login, containing session details.
    #[serde(rename = "login")]
    Login(SessionDto),

    /// A response listing all active sessions.
    #[serde(rename = "who")]
    Who(Vec<SessionDto>),

    /// A response indicating an error, containing an error message.
    #[serde(rename = "error")]
    Error { message: String },

    /// A response indicating a successful logout.
    #[serde(rename = "logout")]
    Logout,
}
