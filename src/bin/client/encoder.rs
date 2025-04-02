use webx_session_manager::common::{Request, Response};

/// The `Encoder` struct provides methods for encoding and decoding
/// requests and responses to and from JSON format.
#[derive(Default)]
pub struct Encoder;

impl Encoder {
    /// Creates a new `Encoder` instance.
    ///
    /// # Returns
    /// A new `Encoder` instance.
    pub fn new() -> Self {
        Default::default()
    }

    /// Encodes a `Request` into a JSON string.
    ///
    /// # Arguments
    /// * `request` - The `Request` to encode.
    ///
    /// # Returns
    /// An `Option` containing the JSON string, or `None` if encoding fails.
    pub fn encode(&self, request: Request) -> Option<String> {
        match serde_json::to_string(&request) {
            Ok(json) => Some(json),
            Err(_) => None,
        }
    }

    /// Decodes a JSON string into a `Response`.
    ///
    /// # Arguments
    /// * `json` - The JSON string to decode.
    ///
    /// # Returns
    /// An `Option` containing the `Response`, or `None` if decoding fails.
    pub fn decode(&self, json: &str) -> Option<Response> {
        match serde_json::from_str::<Response>(json) {
            Ok(response) => Some(response),
            Err(_) => None,
        }
    }
}
