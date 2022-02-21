extern crate dotenv;
extern crate log;
extern crate pam_client;
extern crate serde;


use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use dotenv::dotenv;
use env_logger::Env;
use libc::{SIGINT, SIGQUIT, SIGTERM};
use log::info;
use nix::unistd::{Uid, User};
use signal_hook::iterator::Signals;
use structopt::StructOpt;

use webx_session_manager::{common::{ApplicationError, Settings}, fs, services::Server};

#[derive(StructOpt, Debug)]
#[structopt(name = "webx-session-manager")]
struct Opt {
    /// Config path
    #[structopt(short, long, default_value = "")]
    config: String,
}

#[tokio::main]
pub async fn main() -> Result<(), ApplicationError> {
    dotenv().ok();

    if !Uid::effective().is_root() {
        eprintln!("You must run this executable with root permissions");
        std::process::exit(1);
    }

    if let Err(error) = run().await {
        eprintln!("There was an error launching webx session manger: {}", error);
    }
    Ok(())
}

pub async fn run() -> Result<(), ApplicationError> {
    let opt = Opt::from_args();

    let settings = Settings::new(&opt.config)?;

    if settings.is_valid() {
        let env = Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, settings.logging().level());
        env_logger::init_from_env(env);

        bootstrap(&settings)?;

        let stop_signal = Arc::new(AtomicBool::new(false));

        signal_hook::flag::register(SIGTERM, Arc::clone(&stop_signal))?;
        signal_hook::flag::register(SIGINT, Arc::clone(&stop_signal))?;
        signal_hook::flag::register(SIGQUIT, Arc::clone(&stop_signal))?;

        let context = zmq::Context::new();
        let mut server = Server::new(settings, context);

        let mut signals = Signals::new(&[
            SIGTERM,
            SIGINT,
            SIGQUIT,
        ]).expect("Signals::new() failed");
        let handle = tokio::spawn(async move {
            server.run(stop_signal).unwrap();
        });

        signals.forever();

        info!("Termination signal received. Shutting down session manager...");

        if handle.await.is_err() {
            eprintln!("Error joining server handle");
        }

    }
    Ok(())
}

pub fn bootstrap(settings: &Settings) -> Result<(), ApplicationError> {

    fs::mkdir(settings.logging().path())?;
    fs::mkdir(settings.xorg().log_path())?;

    // create the sessions directory
    if let Ok(Some(user)) = User::from_name("webx") {
        let sessions_path = settings.xorg().sessions_path();
        fs::mkdir(sessions_path)?;
        // ensure permissions and ownership are correct
        fs::chown(sessions_path, user.uid.as_raw(), user.gid.as_raw())?;
        fs::chmod(sessions_path, 0o775)?;
    } else {
        return Err(ApplicationError::environment("Could not create sessions directory. Check the user 'webx' exists"));
    }
    
    Ok(())

}