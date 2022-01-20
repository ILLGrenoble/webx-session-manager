use structopt::StructOpt;

use webx_session_manager::{authentication::Credentials, common::ApplicationError, services::Client};

#[derive(StructOpt)]
#[structopt(about = "WebX Session Manager Client")]
enum Command {
    Who,
    Login {
        #[structopt(short, long)]
        username: String,

        #[structopt(short, long)]
        password: String,
    },
    Terminate {
        #[structopt(short, long)]
        username: String,
    },
}

pub fn main() -> Result<(), ApplicationError> {
    let command = Command::from_args();
    let client = Client::new("/tmp/webx-session-manager.ipc".to_string())?;
    match command {
        Command::Who => client.who()?,
        Command::Login { username, password } => {
            let credentials = Credentials::new(username, password);
            client.login(credentials)?;
        }
        Command::Terminate { username } => client.terminate(username)
    }

    Ok(())
}

