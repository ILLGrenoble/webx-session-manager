use structopt::StructOpt;

use webx_session_manager::{authentication::Credentials, common::{ApplicationError, ScreenResolution}, services::Client};

#[derive(StructOpt)]
#[structopt(about = "WebX Session Manager Client")]
enum Command {
    Who,
    Login {
        #[structopt(short, long)]
        username: String,

        #[structopt(short, long)]
        password: String,

        #[structopt(short, long)]
        width: u32,

        #[structopt(short, long)]
        height: u32,
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
        Command::Login { username, password, width, height } => {
            let credentials = Credentials::new(username, password);
            let resolution = ScreenResolution::new(width, height);
            client.login(credentials, resolution)?;
        }
        Command::Terminate { username } => client.terminate(username)
    }

    Ok(())
}

