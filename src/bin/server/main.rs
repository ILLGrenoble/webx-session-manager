extern crate serde;
extern crate log;
extern crate pam_client;
extern crate dotenv;



use env_logger::Env;
use nix::unistd::Uid;
use structopt::StructOpt;
use webx_session_manager::{common::{ApplicationError, Settings}, services::Server};
use dotenv::dotenv;


#[derive(StructOpt, Debug)]
#[structopt(name = "webx-session-manager")]
struct Opt {
    /// Config path
    #[structopt(short, long, default_value = "")]
    config: String
}

pub fn main() -> Result<(), ApplicationError> {
    dotenv().ok();

    if !Uid::effective().is_root() {
        eprintln!("You must run this executable with root permissions");
        std::process::exit(1);

    }
    
    if let Err(error) = run() {
        eprintln!("There was an error launching webx session manger: {}", error);
    }
    Ok(())
}

pub fn run() -> Result<(), ApplicationError> {
    let opt = Opt::from_args();

    let settings = Settings::new(&opt.config)?;

    if settings.is_valid() {
        let env = Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, settings.logging().level());
        env_logger::init_from_env(env);

        let context = zmq::Context::new();
        let mut server = Server::new(settings, context);
        server.run()?;
    }
    Ok(())
}

