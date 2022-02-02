use structopt::StructOpt;

use webx_session_manager::{authentication::Credentials, common::{ApplicationError, ScreenResolution}, services::Client};

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
        password: String,

        #[structopt(short, long)]
        width: u32,

        #[structopt(short, long)]
        height: u32,

        #[structopt(long, default_value = "/tmp/webx-session-manager.ipc")]
        ipc: String
    }
}

pub fn main() -> Result<(), ApplicationError> {
    let command = Command::from_args();
    match command {
        Command::Who {ipc} => {
            let client = Client::new(ipc)?;
            client.who()?
        }
        Command::Login { ipc, username, password, width, height } => {
            let credentials = Credentials::new(username, password);
            let resolution = ScreenResolution::new(width, height);
            let client = Client::new(ipc)?;
            client.login(credentials, resolution)?;
        }
    }

    Ok(())
}

