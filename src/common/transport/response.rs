use serde::{Deserialize, Serialize};

use crate::dto::SessionDto;
#[derive(Serialize, Deserialize)]
#[serde(tag = "response", content = "content")]
pub enum Response {
    #[serde(rename = "login")]
    Login(SessionDto),
    #[serde(rename = "who")]
    Who(Vec<SessionDto>),
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "logout")]
    Logout,
    
}
