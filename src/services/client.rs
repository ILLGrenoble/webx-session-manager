use prettytable::{Table, Row, Cell};

use crate::{authentication::Credentials, common::{ApplicationError, Response, Request}};

pub struct Client {
    socket: zmq::Socket
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
        if let Ok(response) =  self.send(Request::Who) {
            match response {
                Response::Who(sessions) => {
                    let mut table = Table::new();
                    table.add_row(Row::new(vec![
                        Cell::new("Display ID"),
                        Cell::new("Process ID"),
                        Cell::new("Username"),
                        Cell::new("UID"),
                        Cell::new("X Authority file path")
                    ]));
            
                    for session in sessions {
                        table.add_row(Row::new(vec![
                            Cell::new(session.display_id()),
                            Cell::new(&session.process_id().to_string()),
                            Cell::new(session.username()),
                            Cell::new(&session.uid().to_string()),
                            Cell::new(session.xauthority_file_path())
                        ]));  
                    }
                  
                    table.printstd();
                    
                },
                Response::Error { message } => println!("Got an error response: {}", message),
                _ => println!("Received an unknown response")
            }
        }

        Ok(())
    }

    /// login the user and return the create session
    pub fn login(&self, credentials: Credentials) -> Result<(), ApplicationError>{
        println!("Logging in user: {}", credentials.username());
        
        let request = Request::Login { 
            username: credentials.username().into(), 
            password: credentials.password().into() 
        };
        if let Ok(response) =  self.send(request) {
            match response {
                Response::Login(session) => {
                    println!("Session launched:{}", session);
                    
                },
                Response::Error { message } => println!("Got an error response: {}", message),
                _ => println!("Received an unknown response")
            }
        }

        Ok(())
    }

    pub fn terminate(&self, _: String) {
        eprintln!("Not implemented for the moment");
    }

    fn send(&self, request: Request) -> Result<Response, ApplicationError> {
        let request = self.encode(request).ok_or(ApplicationError::transport("could not encode the request"))?;
        self.socket.send(&request[..], 0)?;
        let mut msg = zmq::Message::new();
        self.socket.recv(&mut msg, 0)?;
        let response = msg.as_str().ok_or(ApplicationError::transport("could not decode the response"))?;
        let response = self.decode(response).ok_or(ApplicationError::transport("could not decode the response into an object"))?;
        Ok(response)
    }

    fn encode(&self, request: Request) -> Option<String> {
        match serde_json::to_string(&request) {
            Ok(json) => Some(json),
            Err(_) => None,
        }
    }

    fn decode(&self, json: &str) -> Option<Response> {
        match serde_json::from_str::<Response>(json) {
            Ok(response)=> Some(response),
            Err(_) => None,
        }
    }
    
}