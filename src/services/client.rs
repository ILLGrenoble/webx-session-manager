use prettytable::{Cell, Row, Table};

use crate::{authentication::{Credentials}, common::{ApplicationError, Request, Response, ScreenResolution}};

pub struct Client {
    socket: zmq::Socket,
}


impl Client {
    pub fn new(ipc: String) -> Result<Self, ApplicationError> {
        let context = zmq::Context::new();
        let socket = context.socket(zmq::REQ)?;
        let address = format!("ipc://{}", ipc);
        socket.connect(&address)?;

        Ok(Self {
            socket
        })
    }


    /// get a list of active sessions
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

    /// login the user and return the create session
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
                Response::Error { message } => println!("Got an error response: {}", message),
                _ => println!("Received an unknown response")
            }
        }

        Ok(())
    }

    fn send(&self, request: Request) -> Result<Response, ApplicationError> {
        let request = self.encode(request).ok_or_else(|| ApplicationError::transport("could not encode the request"))?;
        self.socket.send(&request[..], 0)?;
        let mut msg = zmq::Message::new();
        self.socket.recv(&mut msg, 0)?;
        let response = msg.as_str().ok_or_else(|| ApplicationError::transport("could not decode the response"))?;
        self.decode(response).ok_or_else(|| ApplicationError::transport("could not encode the request"))
    }

    fn encode(&self, request: Request) -> Option<String> {
        match serde_json::to_string(&request) {
            Ok(json) => Some(json),
            Err(_) => None,
        }
    }

    fn decode(&self, json: &str) -> Option<Response> {
        match serde_json::from_str::<Response>(json) {
            Ok(response) => Some(response),
            Err(_) => None,
        }
    }
}