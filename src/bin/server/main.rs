extern crate dotenv;
extern crate log;
extern crate serde;


use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::process;

use dotenv::dotenv;
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

/// The main entry point for the WebX Session Manager server.
/// This program initializes the server, sets up logging, and handles termination signals.
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

/// Runs the main logic of the WebX Session Manager server.
///
/// # Returns
/// A `Result` indicating success or an `ApplicationError`.
pub async fn run() -> Result<(), ApplicationError> {
    let opt = Opt::from_args();

    let settings = Settings::new(&opt.config)?;

    if settings.is_valid() {

        // Initialize logging
        if let Err(e) = setup_logging(&settings) {
            eprintln!("Failed to initialize logging: {}", e);
            process::exit(1);
        }

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

/// Performs initial setup for the server, including creating necessary directories
/// and ensuring correct permissions.
///
/// # Arguments
/// * `settings` - The configuration settings for the server.
///
/// # Returns
/// A `Result` indicating success or an `ApplicationError`.
fn bootstrap(settings: &Settings) -> Result<(), ApplicationError> {

    fs::mkdir(settings.xorg().log_path())?;

    // create the sessions directory
    if let Ok(Some(user)) = User::from_name("webx") {
        let sessions_path = settings.xorg().sessions_path();
        fs::mkdir(sessions_path)?;
        // ensure permissions and ownership are correct
        fs::chown(sessions_path, user.uid.as_raw(), user.gid.as_raw())?;
        fs::chmod(sessions_path, 0o755)?;
    } else {
        return Err(ApplicationError::environment("Could not create sessions directory. Check the user 'webx' exists"));
    }
    
    Ok(())
}

/// Configures logging for the server based on the provided settings.
///
/// # Arguments
/// * `settings` - The configuration settings for logging.
///
/// # Returns
/// A `Result` indicating success or a `fern::InitError` if logging setup fails.
fn setup_logging(settings: &Settings) -> Result<(), fern::InitError> {
    let logging_config = &settings.logging();

    let format_string = logging_config.format().clone();
    let mut base_config = fern::Dispatch::new()
        .format(move |out, message, record| {
            let format = format_string
                .as_deref()
                .unwrap_or("[{timestamp}][{level}] {message}");
            let formatted_message = format
                .replace("{timestamp}", &chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string())
                .replace("{level}", &record.level().to_string())
                .replace("{message}", &message.to_string());
            out.finish(format_args!("{}", formatted_message))
        })
        .level(logging_config.level().parse::<log::LevelFilter>().unwrap_or(log::LevelFilter::Info));

    if logging_config.console().unwrap_or(true) {
        base_config = base_config.chain(std::io::stdout());
    }

    if let Some(file_config) = &logging_config.file() {
        if file_config.enabled().unwrap_or(false) {
            let log_file = std::fs::OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(&file_config.path())?;
            base_config = base_config.chain(log_file);
        }
    }

    base_config.apply()?;
    Ok(())
}
