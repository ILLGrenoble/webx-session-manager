use super::{Request, Response};

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

    /// Encodes a `Response` into a JSON string.
    ///
    /// # Arguments
    /// * `response` - The `Response` to encode.
    ///
    /// # Returns
    /// An `Option` containing the JSON string, or `None` if encoding fails.
    pub fn encode(&self, response: Response) -> Option<String> {
        match serde_json::to_string(&response) {
            Ok(json) => Some(json),
            Err(_) => None,
        }
    }

    /// Decodes a JSON string into a `Request`.
    ///
    /// # Arguments
    /// * `json` - The JSON string to decode.
    ///
    /// # Returns
    /// An `Option` containing the `Request`, or `None` if decoding fails.
    pub fn decode(&self, json: &str) -> Option<Request> {
        match serde_json::from_str::<Request>(json) {
            Ok(request) => Some(request),
            Err(_) => None,
        }
    }
}
