use nix::unistd::User;
use structopt::StructOpt;

use webx_session_manager::{authentication::{Credentials, Authenticator}, common::{ApplicationError, ScreenResolution, Account}, services::Client};
use rpassword::read_password;
use std::io::Write;
#[derive(StructOpt)]
#[structopt(about = "WebX Session Manager Client")]
enum Command {
    Who {
        #[structopt(long, default_value = "/tmp/webx-session-manager.ipc")]
        ipc: String
    },
    Login {
        #[structopt(short, long)]
        username: String,

        #[structopt(short, long)]
        width: u32,

        #[structopt(short, long)]
        height: u32,

        #[structopt(long, default_value = "/tmp/webx-session-manager.ipc")]
        ipc: String
    },
    Logout {
        #[structopt(short, long)]
        id: String,

        #[structopt(long, default_value = "/tmp/webx-session-manager.ipc")]
        ipc: String
    },
    Authenticate {
        #[structopt(short, long)]
        username: String,

        #[structopt(short, long)]
        service: String,
    },
}

pub fn main() -> Result<(), ApplicationError> {
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
                    if let Ok(Some(user)) = User::from_name(&credentials.username()) {
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

