use webx_session_manager::common::{Request, Response};

#[derive(Default)]
pub struct Encoder;

impl Encoder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn encode(&self, request: Request) -> Option<String> {
        match serde_json::to_string(&request) {
            Ok(json) => Some(json),
            Err(_) => None,
        }
    }

    pub fn decode(&self, json: &str) -> Option<Response> {
        match serde_json::from_str::<Response>(json) {
            Ok(response) => Some(response),
            Err(_) => None,
        }
    }
}
