use super::{Request, Response};

#[derive(Default)]
pub struct Encoder;

impl Encoder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn encode(&self, response: Response) -> Option<String> {
        match serde_json::to_string(&response) {
            Ok(json) => Some(json),
            Err(_) => None,
        }
    }

    pub fn decode(&self, json: &str) -> Option<Request> {
        match serde_json::from_str::<Request>(json) {
            Ok(request) => Some(request),
            Err(_) => None,
        }
    }
}
