use crate::{
    authentication::{Authenticator, Credentials},
    common::{ApplicationError, Encoder, Request, Response, Settings, ScreenResolution},
};

use super::{SessionService, XorgService};
use crate::dto::SessionDto;

pub struct Server {
    context: zmq::Context,
    is_running: bool,
    session_service: SessionService,
    encoder: Encoder,
    address: String,
}

impl Server {
    pub fn new(settings: Settings, context: zmq::Context) -> Self {
        let authenticator = Authenticator::new(settings.authentication().to_owned());
        let xorg_service = XorgService::new(settings.xorg().to_owned());
        let session_service = SessionService::new(authenticator, xorg_service);
        let address = format!("ipc://{}", settings.transport().ipc());
        let encoder = Encoder::new();
        Self {
            context,
            is_running: false,
            session_service,
            encoder,
            address,
        }
    }

    /// Launch the server and start listening for requests
    pub fn run(&mut self) -> Result<(), ApplicationError> {
        let rep_socket = self.create_rep_socket()?;

        let mut items = [rep_socket.as_poll_item(zmq::POLLIN)];

        self.is_running = true;
        while self.is_running {
            // Poll both sockets
            if zmq::poll(&mut items, 5000).is_ok() {

                // clean up zombie x session
                self.session_service.clean_up();

                // Check for REQ-REP message (if running)
                if items[0].is_readable() && self.is_running {
                    self.handle_request(&rep_socket);
                }
            }
        }

        debug!("Stopped server");

        Ok(())
    }

    fn create_rep_socket(&self) -> Result<zmq::Socket, ApplicationError> {
        let socket = self.context.socket(zmq::REP)?;
        socket.set_linger(0)?;

        if let Err(error) = socket.bind(&self.address) {
            return Err(ApplicationError::transport(format!("Failed to bind reply socket to {}: {}", self.address, error)));
        } else {
            info!("Server bound to address {} and listening for requests", self.address);
        }

        Ok(socket)
    }

    fn handle_request(&self, rep_socket: &zmq::Socket) {
        let mut message = zmq::Message::new();

        if let Err(error) = rep_socket.recv(&mut message, 0) {
            error!("Failed to received message on req-rep: {}", error);
        } else if let Some(request) = message.as_str() {
            match self.encoder.decode(request) {
                Some(request) => match request {
                    Request::Login { username, password, width, height } => {
                        let credentials = Credentials::new(username, password);
                        let resolution = ScreenResolution::new(width, height);
                        self.handle_login_request(rep_socket, credentials, resolution)
                    }
                    Request::Who => self.handle_who_request(rep_socket),
                    Request::Terminate { id } => self.handle_terminate_request(rep_socket, id),
                },
                None => self.handle_unknown_request(rep_socket),
            }
        } else {
            self.handle_unknown_request(rep_socket);
        }
    }

    fn handle_unknown_request(&self, rep_socket: &zmq::Socket) {
        if let Err(error) = rep_socket.send("unknown request", 0) {
            error!("failed to send response message: {}", error);
        }
    }

    fn handle_login_request(&self, 
        rep_socket: &zmq::Socket, 
        credentials: Credentials, 
        resolution: ScreenResolution
    ) {
        debug!("Creating session for user {} with resolution: {}", credentials.username(), resolution);
        let response = match self.session_service.create_session(&credentials, resolution) {
            Ok(session) => Response::Login(SessionDto::from(&session)),
            Err(error) => {
                error!("{}", error);
                Response::Error { message: format!("error creating session: {}", error.to_string()) }
            }
        };
        let json = self.encoder.encode(response).unwrap_or_else(|| "".into());
        if let Err(error) = rep_socket.send(&json[..], 0) {
            error!("Failed to send response message: {}", error);
        }
    }

    fn handle_who_request(&self, rep_socket: &zmq::Socket) {
        debug!("Listing sessions");
        let sessions =self.session_service.get_all().unwrap_or_default();
        let dtos = sessions.iter().map(|session|  session.into()).collect();
        let response = Response::Who(dtos);
        let json = self.encoder.encode(response).unwrap_or_else(|| "".into());
        if let Err(error) = rep_socket.send(&json[..], 0) {
            error!("Failed to send response message: {}", error);
        }
    }

    fn handle_terminate_request(&self, rep_socket: &zmq::Socket, id: u32) {
        debug!("Terminating session with id: {}", id);
        if let Err(error) = rep_socket.send("launched", 0) {
            error!("Failed to send response message: {}", error);
        }
    }
}
