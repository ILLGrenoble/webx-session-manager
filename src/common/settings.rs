use std::{path::Path};
use serde::Deserialize;

use super::ApplicationError;


#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    logging: LoggingSettings,
    authentication: AuthenticationSettings,
    transport: TransportSettings,
    xorg: XorgSettings
}


#[derive(Debug, Deserialize, Clone)]
pub struct TransportSettings {
    ipc: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct XorgSettings {
    lock_path: String,
    authority_path: String,
    display_offset: u32
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingSettings {
    level: String,
    path: String
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthenticationSettings {
    service: String
}

impl AuthenticationSettings {
    pub fn service(&self) -> &str {
        &self.service
    }
}

impl LoggingSettings {

    pub fn level(&self) -> &str {
        &self.level
    }

    pub fn path(&self) -> &str {
        &self.path
    }

}

impl XorgSettings {

    pub fn lock_path(&self) -> &str {
        &self.lock_path
    }

    pub fn authority_path(&self) -> &str {
        &self.authority_path
    }

    pub fn display_offset(&self) -> u32 {
        self.display_offset
    }

}

impl TransportSettings {
    pub fn ipc(&self) -> &str {
        &self.ipc
    }
}

static DEFAULT_CONFIG_PATHS: [&str; 2] = ["/etc/webx/webx-session-manager-config.yml", "./config.yml"];



impl Settings {
    pub fn new(config_path: &str) -> Result<Self, ApplicationError> {

        let config_path = Settings::get_config_path(config_path);

        let mut settings_raw = config::Config::default();

        settings_raw.merge(config::File::new(config_path, config::FileFormat::Yaml))?;
        settings_raw.merge(config::Environment::with_prefix("WEBX_SESSION_MANAGER").separator("_"))?;

        settings_raw.try_into().map_err(|error| error.into())
    }


    pub fn logging(&self) -> &LoggingSettings {
        &self.logging
    }

    pub fn transport(&self) -> &TransportSettings {
        &self.transport
    }

    pub fn authentication(&self) -> &AuthenticationSettings {
        &self.authentication
    }

    pub fn xorg(&self) -> &XorgSettings {
        &self.xorg
    }


    fn get_config_path(config_path: &str) -> &str {
        if config_path.is_empty() {
            for path in DEFAULT_CONFIG_PATHS.iter() {
                if Path::new(path).exists() {
                    return path;
                }
            }
        }
        config_path
    }

    pub fn is_valid(&self) -> bool {
        // check that settings are valid for running the session manager

        if self.logging.path.is_empty() {
            eprintln!("Please specify a log path");
            return false;
        }

        if self.logging.level.is_empty() {
            eprintln!("Please specify a logging level (trace, debug, info, error)");
            return false;
        }

        if self.authentication.service.is_empty() {
            eprintln!("Please specify a PAM service to use (i.e. login)");
            return false
        }

        if self.transport.ipc.is_empty() {
            eprintln!("Please specify a path to the ipc socket (i.e. /tmp/webx-session-manager.ipc)");
            return false;
        }
        

        if self.xorg.authority_path.is_empty() {
            eprintln!("Please specify a path for where to store the xauthority files (i.e. /run/user");
            return false;
        }

        if self.xorg.lock_path.is_empty() {
            eprintln!("Please specify a path for where to look for x lock files (i.e. /tmp/.X11-unix");
            return false;
        }

    
        true
    }

    
}

