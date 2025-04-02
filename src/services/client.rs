use prettytable::{Cell, Row, Table};

use crate::{authentication::{Credentials}, common::{ApplicationError, Request, Response, ScreenResolution}};

/// The `Client` struct provides functionality for interacting with the WebX Session Manager server,
/// including sending requests and handling responses.
pub struct Client {
    socket: zmq::Socket,
}

impl Client {
    /// Creates a new `Client` instance.
    ///
    /// # Arguments
    /// * `ipc` - The IPC path to the WebX Session Manager server.
    ///
    /// # Returns
    /// A `Result` containing the `Client` or an `ApplicationError` if the client could not be created.
    pub fn new(ipc: String) -> Result<Self, ApplicationError> {
        let context = zmq::Context::new();
        let socket = context.socket(zmq::REQ)?;
        let address = format!("ipc://{}", ipc);
        socket.connect(&address)?;

        Ok(Self {
            socket
        })
    }

    /// Retrieves a list of all active sessions.
    ///
    /// # Returns
    /// A `Result` indicating success or an `ApplicationError`.
    pub fn who(&self) -> Result<(), ApplicationError> {
        println!("Fetching a list of sessions");

        if let Ok(response) = self.send(Request::Who) {
            match response {
                Response::Who(sessions) => {
                    let mut table = Table::new();
                    table.add_row(Row::new(vec![
                        Cell::new("ID"),
                        Cell::new("Display"),
                        Cell::new("Xorg PID"),
                        Cell::new("Window Manager PID"),
                        Cell::new("User"),
                        Cell::new("XAuthority")
                    ]));

                    for session in sessions {
                        table.add_row(Row::new(vec![
                            Cell::new(session.id()),
                            Cell::new(session.display_id()),
                            Cell::new(&session.xorg_process_id().to_string()),
                            Cell::new(&session.window_manager_process_id().to_string()),
                            Cell::new(&format!("{} ({})", session.username(), &session.uid())),
                            Cell::new(session.xauthority_file_path())
                        ]));
                    }

                    table.printstd();
                }
                Response::Error { message } => println!("Got an error response: {}", message),
                _ => println!("Received an unknown response")
            }
        }

        Ok(())
    }

    /// Logs in a user and creates a new session.
    ///
    /// # Arguments
    /// * `credentials` - The user's credentials.
    /// * `resolution` - The screen resolution for the session.
    ///
    /// # Returns
    /// A `Result` indicating success or an `ApplicationError`.
    pub fn login(&self, credentials: Credentials, resolution: ScreenResolution) -> Result<(), ApplicationError> {
        println!("Logging in user: {}", credentials.username());

        let request = Request::Login {
            username: credentials.username().into(),
            password: credentials.password().into(),
            width: resolution.width(),
            height: resolution.height()
        };
        if let Ok(response) = self.send(request) {
            match response {
                Response::Login(session) => {
                    println!("Session launched: {}", session);
                }
                Response::Error { message } => println!("Received an error response: {}", message),
                _ => println!("Received an unknown response")
            }
        }

        Ok(())
    }

    /// Logs out a session by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the session to log out.
    ///
    /// # Returns
    /// A `Result` indicating success or an `ApplicationError`.
    pub fn logout(&self, id: String) -> Result<(), ApplicationError> {
        println!("Logging out session {}", id);

        let request = Request::Logout {
            id
        };
        if let Ok(response) = self.send(request) {
            match response {
                Response::Logout => {
                    println!("Session logged out successfully");
                }
                Response::Error { message } => println!("Received an error response: {}", message),
                _ => println!("Received an unknown response")
            }
        }

        Ok(())
    }

    /// Sends a request to the WebX Session Manager server and receives a response.
    ///
    /// # Arguments
    /// * `request` - The request to send.
    ///
    /// # Returns
    /// A `Result` containing the `Response` or an `ApplicationError`.
    fn send(&self, request: Request) -> Result<Response, ApplicationError> {
        let request = self.encode(request).ok_or_else(|| ApplicationError::transport("could not encode the request"))?;
        self.socket.send(&request[..], 0)?;
        let mut msg = zmq::Message::new();
        self.socket.recv(&mut msg, 0)?;
        let response = msg.as_str().ok_or_else(|| ApplicationError::transport("could not decode the response"))?;
        self.decode(response).ok_or_else(|| ApplicationError::transport("could not encode the request"))
    }

    /// Encodes a `Request` into a JSON string.
    ///
    /// # Arguments
    /// * `request` - The `Request` to encode.
    ///
    /// # Returns
    /// An `Option` containing the JSON string, or `None` if encoding fails.
    fn encode(&self, request: Request) -> Option<String> {
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
    fn decode(&self, json: &str) -> Option<Response> {
        match serde_json::from_str::<Response>(json) {
            Ok(response) => Some(response),
            Err(_) => None,
        }
    }
}