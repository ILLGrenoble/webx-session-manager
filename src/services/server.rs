use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use nix::unistd::User;

use crate::{
    authentication::{Authenticator, Credentials},
    common::{ApplicationError, Encoder, Request, Response, ScreenResolution, Settings},
};
use crate::common::Account;
use crate::dto::SessionDto;
use crate::fs::chown;

use super::{SessionService, XorgService};

pub struct Server {
    context: zmq::Context,
    session_service: SessionService,
    encoder: Encoder,
    ipc: String,
}

impl Server {
    pub fn new(settings: Settings, context: zmq::Context) -> Self {
        let authenticator = Authenticator::new(settings.authentication().service().to_owned());
        let xorg_service = XorgService::new(settings.xorg().to_owned());
        let session_service = SessionService::new(authenticator, xorg_service);
        let ipc = settings.transport().ipc().to_owned();
        let encoder = Encoder::new();
        Self {
            context,
            session_service,
            encoder,
            ipc
        }
    }

    /// Launch the server and start listening for requests
    pub fn run(&mut self, stop_signal: Arc<AtomicBool>) -> Result<(), ApplicationError> {
        let rep_socket = self.create_rep_socket()?;

        let mut items = [rep_socket.as_poll_item(zmq::POLLIN)];

        // listen for messages until a kill signal is received
        while !stop_signal.load(Ordering::SeqCst) {
            // Poll both sockets
            if zmq::poll(&mut items, 1000).is_ok() {

                // clean up zombie x session
                self.session_service.clean_up();

                // Check for REQ-REP message (if running)
                if items[0].is_readable() {
                    self.handle_request(&rep_socket);
                }
            }
        }
        
        self.clean_up()?;

        info!("Stopped session manager");

        Ok(())
    }

    fn get_socket_user(&self) -> Result<User, ApplicationError> {
        match User::from_name("webx") {
            Ok(Some(user)) => Ok(user),
            _ => Err(ApplicationError::environment("could not find user webx"))
        }
    }

    fn clean_up(&self) -> Result<(), ApplicationError> {
        debug!("Deleting ipc socket descriptor");
        fs::remove_file(&self.ipc)?;
        // killing all sessions
        debug!("Killing all sessions...");
        self.session_service.kill_all()?;
        Ok(())
    }

    fn create_rep_socket(&self) -> Result<zmq::Socket, ApplicationError> {
        let address = format!("ipc://{}", &self.ipc);
        let socket = self.context.socket(zmq::REP)?;
        socket.set_linger(0)?;

        if let Err(error) = socket.bind(&address) {
            return Err(ApplicationError::transport(format!("Failed to bind reply socket to {}: {}", &address, error)));
        } else {
            info!("Server bound to address {} and listening for requests", &address);

            // change ownership of the socket
            let socket_user = self.get_socket_user()?;
            let socket_user_account = Account::from_user(socket_user).unwrap();
            debug!("Changing ownership of ipc address to webx user");
            chown(&self.ipc, socket_user_account.uid(), socket_user_account.gid())?;
        }

        Ok(socket)
    }

    fn handle_request(&self, rep_socket: &zmq::Socket) {
        let mut message = zmq::Message::new();

        if let Err(error) = rep_socket.recv(&mut message, 0) {
            error!("Failed to received message on req-rep: {}", error);
        } else if let Some(request) = message.as_str() {
            info!("Received a request");
            match self.encoder.decode(request) {
                Some(request) => match request {
                    Request::Login { username, password, width, height } => {
                        debug!("Handling login request");
                        let credentials = Credentials::new(username, password);
                        let resolution = ScreenResolution::new(width, height);
                        self.handle_login_request(rep_socket, credentials, resolution)
                    }
                    Request::Who => self.handle_who_request(rep_socket),
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
                            resolution: ScreenResolution,
    ) {
        debug!("Creating session for user {} with resolution: {}", credentials.username(), resolution);
        let response = match self.session_service.create_session(&credentials, resolution) {
            Ok(session) => {
                Response::Login(SessionDto::from(&session))
            },
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
        let sessions = self.session_service.get_all().unwrap_or_default();
        let dtos = sessions.iter().map(|session| session.into()).collect();
        let response = Response::Who(dtos);
        let json = self.encoder.encode(response).unwrap_or_else(|| "".into());
        if let Err(error) = rep_socket.send(&json[..], 0) {
            error!("Failed to send response message: {}", error);
        }
    }

}
