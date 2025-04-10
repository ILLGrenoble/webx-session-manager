use nix::unistd::{User, Uid};
use structopt::StructOpt;

use webx_session_manager::{authentication::{Credentials, Authenticator}, common::{ApplicationError, ScreenResolution, Account}, services::Client};
use rpassword::read_password;
use std::io::Write;

/// The `Command` enum represents the various commands that the WebX Session Manager client can execute.
#[derive(StructOpt)]
#[structopt(about = "WebX Session Manager Client")]
enum Command {
    /// Lists all active sessions.
    Who {
        /// The IPC path to the WebX Session Manager server.
        #[structopt(long, default_value = "/tmp/webx-session-manager.ipc")]
        ipc: String,
    },
    /// Logs in a user and creates a new session.
    Login {
        /// The username of the user.
        #[structopt(short, long)]
        username: String,

        /// The screen width for the session.
        #[structopt(short, long)]
        width: u32,

        /// The screen height for the session.
        #[structopt(short, long)]
        height: u32,

        /// The IPC path to the WebX Session Manager server.
        #[structopt(long, default_value = "/tmp/webx-session-manager.ipc")]
        ipc: String,
    },
    /// Logs out a user and terminates the session.
    Logout {
        /// The session ID to terminate.
        #[structopt(short, long)]
        id: String,

        /// The IPC path to the WebX Session Manager server.
        #[structopt(long, default_value = "/tmp/webx-session-manager.ipc")]
        ipc: String,
    },
    /// Authenticates a user using the specified PAM service.
    Authenticate {
        /// The username of the user.
        #[structopt(short, long)]
        username: String,

        /// The PAM service to use for authentication.
        #[structopt(short, long)]
        service: String,
    },
}

/// The main entry point for the WebX Session Manager client.
/// This program allows users to interact with the WebX Session Manager server.
pub fn main() -> Result<(), ApplicationError> {

    if !Uid::effective().is_root() {
        eprintln!("You must run this executable with root permissions");
        std::process::exit(1);
    }
    
    let command = Command::from_args();
    match command {
        Command::Who {ipc} => {
            let client = Client::new(ipc)?;
            client.who()?
        }
        Command::Login { ipc, username, width, height } => {
            print!("Enter password:");
            std::io::stdout().flush().unwrap();
            let password = read_password().unwrap();
            let credentials = Credentials::new(username, password);
            let resolution = ScreenResolution::new(width, height);
            let client = Client::new(ipc)?;
            client.login(credentials, resolution)?;
        },
        Command::Logout  { ipc, id} => {
            let client = Client::new(ipc)?;
            client.logout(id)?; 
        }
        Command::Authenticate { service, username} => {
            print!("Enter password:");
            std::io::stdout().flush().unwrap();
            let password = read_password().unwrap();

            let credentials = Credentials::new(username, password);
            let authenticator = Authenticator::new(service);
        
            match authenticator.authenticate(&credentials) {
                Ok(environment) => {
                    println!("Authenticated user: {}", &credentials.username());
                    if let Ok(Some(user)) = User::from_name(credentials.username()) {
                        let account = Account::from_user(user);
                        println!("Account: {}", account.unwrap());
                        println!("Environment: {}", environment);
                    } else {
                        eprintln!("Could not find user account");
                    }
                },
                Err(error) => {
                    eprintln!("Could not autenticate user: {}", error);
                }
            }
  

        }
    }

    Ok(())
}

