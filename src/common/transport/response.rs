use serde::{Deserialize, Serialize};

use crate::common::Session;

#[derive(Serialize, Deserialize)]
#[serde(tag = "response", content = "content")]
pub enum Response {
    #[serde(rename = "login")]
    Login(Session),
    #[serde(rename = "who")]
    Who(Vec<Session>),
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "logout")]
    Logout,
}
