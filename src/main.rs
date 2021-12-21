#[macro_use]
extern crate log;
extern crate pam_client;

use env_logger::Env;
use nix::unistd::User;
use structopt::StructOpt;

use crate::authentication::{Authenticator, Credentials};
use crate::common::{Account, ApplicationError};
use crate::services::Xorg;

mod authentication;
mod cli;
mod common;
mod fs;
mod services;

#[derive(StructOpt, Debug)]
#[structopt(name = "webx")]
struct Opt {
    /// PAM username
    #[structopt(short, long)]
    username: String,

    /// PAM password
    #[structopt(short, long)]
    password: String,

    /// PAM service
    #[structopt(short, long)]
    service: String,

    /// Directory for storing xauthority file
    #[structopt(short, long, default_value = "/run/user")]
    directory: String,
}


pub fn main() -> Result<(), ApplicationError> {
    let env = Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
    env_logger::init_from_env(env);

    info!("Starting session manager...");

    let opt = Opt::from_args();
    let authenticator = Authenticator::new(opt.service);
    let credentials = Credentials::new(opt.username, opt.password);
    match authenticator.authenticate(&credentials) {
        Ok(environment) => {
            if let Some(user) = User::from_name(credentials.username()).unwrap() {
                if let Some(account) = Account::from_user(user) {
                    let xorg = Xorg::new("/run/user", account);
                    match xorg.get_current_display() {
                        Some(display_id) => {
                            info!("Display {} is already running for user: {}", display_id, credentials.username());
                        }
                        None => {
                            let display_id = xorg.get_next_available_display(40)?;
                            info!("Starting display {} for user: {}", display_id, credentials.username());
                            if let Err(error) = xorg.setup() {
                                error!("Error occurred {}", error);
                            } else if let Err(error) = xorg.execute(display_id, &environment) {
                                error!("Could not launch x server: {}", error);
                            }
                        }
                    }
                } else {
                    error!("User doesn't appear to have a home directory");
                }
            }
        }
        Err(error) => {
            error!("Error authenticating: {}", error);
        }
    }
    Ok(())
}
