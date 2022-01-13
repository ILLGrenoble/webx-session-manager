use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "request", content = "content")]
pub enum Request {
    #[serde(rename = "login")]
    Login { username: String, password: String },
    #[serde(rename = "who")]
    Who,
    #[serde(rename = "terminate")]
    Terminate { id: u32 },
}




