use std::{path::Path};

use serde::Deserialize;

use super::ApplicationError;

/// The `Settings` struct represents the configuration settings for the WebX Session Manager.
#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    logging: LoggingSettings,
    authentication: AuthenticationSettings,
    transport: TransportSettings,
    xorg: XorgSettings,
}

/// The `TransportSettings` struct contains settings related to IPC transport.
#[derive(Debug, Deserialize, Clone)]
pub struct TransportSettings {
    ipc: String,
}

/// The `XorgSettings` struct contains settings related to the Xorg server.
#[derive(Debug, Deserialize, Clone)]
pub struct XorgSettings {
    log_path: String,
    lock_path: String,
    sessions_path: String,
    config_path: String,
    display_offset: u32,
    window_manager: String,
}

/// The `FileLoggingSettings` struct contains settings for file-based logging.
#[derive(Debug, Deserialize, Clone)]
pub struct FileLoggingSettings {
    enabled: Option<bool>,
    path: String,
}

/// The `LoggingSettings` struct contains settings for logging configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct LoggingSettings {
    level: String,
    console: Option<bool>,
    file: Option<FileLoggingSettings>,
    format: Option<String>,
}

/// The `AuthenticationSettings` struct contains settings for user authentication.
#[derive(Debug, Deserialize, Clone)]
pub struct AuthenticationSettings {
    service: String,
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

    pub fn console(&self) -> &Option<bool> {
        &self.console
    }

    pub fn file(&self) -> &Option<FileLoggingSettings> {
        &self.file
    }

    pub fn format(&self) -> &Option<String> {
        &self.format
    }
}

impl FileLoggingSettings {
    pub fn enabled(&self) -> &Option<bool> {
        &self.enabled
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

impl XorgSettings {
    pub fn lock_path(&self) -> &str {
        &self.lock_path
    }

    pub fn sessions_path(&self) -> &str {
        &self.sessions_path
    }

    pub fn sessions_path_for_uid(&self, uid: u32) -> String {
        format!("{}/{}", self.sessions_path, uid)
    }
    pub fn display_offset(&self) -> u32 {
        self.display_offset
    }

    pub fn window_manager(&self) -> &str {
        &self.window_manager
    }

    pub fn config_path(&self) -> &str {
        &self.config_path
    }

    pub fn log_path(&self) -> &str {
        &self.log_path
    }
}

impl TransportSettings {
    pub fn ipc(&self) -> &str {
        &self.ipc
    }
}

static DEFAULT_CONFIG_PATHS: [&str; 2] = ["/etc/webx/webx-session-manager-config.yml", "./config.yml"];

impl Settings {
    /// Creates a new `Settings` instance by loading configuration from a file or environment variables.
    ///
    /// # Arguments
    /// * `config_path` - The path to the configuration file. If empty, default paths will be used.
    ///
    /// # Returns
    /// A `Result` containing the `Settings` or an `ApplicationError` if the configuration could not be loaded.
    pub fn new(config_path: &str) -> Result<Self, ApplicationError> {
        let config_path = Settings::get_config_path(config_path);

        let settings_raw = config::Config::builder()
            .add_source(config::File::new(config_path, config::FileFormat::Yaml))
            .add_source(config::Environment::with_prefix("WEBX_SESSION_MANAGER").separator("_"))
            .build()?;        
 
        settings_raw.try_deserialize().map_err(|error| error.into())
    }

    /// Returns the logging settings.
    ///
    /// # Returns
    /// A reference to the `LoggingSettings`.
    pub fn logging(&self) -> &LoggingSettings {
        &self.logging
    }

    /// Returns the transport settings.
    ///
    /// # Returns
    /// A reference to the `TransportSettings`.
    pub fn transport(&self) -> &TransportSettings {
        &self.transport
    }

    /// Returns the authentication settings.
    ///
    /// # Returns
    /// A reference to the `AuthenticationSettings`.
    pub fn authentication(&self) -> &AuthenticationSettings {
        &self.authentication
    }

    /// Returns the Xorg settings.
    ///
    /// # Returns
    /// A reference to the `XorgSettings`.
    pub fn xorg(&self) -> &XorgSettings {
        &self.xorg
    }

    /// Determines the configuration file path to use.
    ///
    /// # Arguments
    /// * `config_path` - The provided configuration file path. If empty, default paths will be checked.
    ///
    /// # Returns
    /// The resolved configuration file path as a string slice.
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

    /// Validates the settings to ensure they are suitable for running the session manager.
    ///
    /// # Returns
    /// `true` if the settings are valid, otherwise `false`.
    ///
    /// # Errors
    /// Prints error messages to `stderr` for any invalid configuration values.
    pub fn is_valid(&self) -> bool {
        // check that settings are valid for running the session manager

        if self.logging.level.is_empty() {
            eprintln!("Please specify a logging level (trace, debug, info, error)");
            return false;
        }

        if self.logging.file.is_some() {
            let file = self.logging.file.as_ref().unwrap();

            if file.enabled.unwrap() && file.path.is_empty() {
                eprintln!("Please specify a path for the log file");
                return false;
            }
        }

        if self.authentication.service.is_empty() {
            eprintln!("Please specify a PAM service to use (i.e. login)");
            return false;
        }

        if self.transport.ipc.is_empty() {
            eprintln!("Please specify a path to the ipc socket (i.e. /tmp/webx-session-manager.ipc)");
            return false;
        }

        if self.xorg.sessions_path.is_empty() {
            eprintln!("Please specify a path for where to store the session files (i.e. /run/webx/sessions");
            return false;
        }

        if self.xorg.lock_path.is_empty() {
            eprintln!("Please specify a path for where to look for x lock files (i.e. /tmp/.X11-unix");
            return false;
        }

        if self.xorg.window_manager.is_empty() {
            eprintln!("Please specify a path to a command that will launch your chosen session manager");
            return false;
        }

        if self.xorg.log_path.is_empty() {
            eprintln!("Please specify a path to store the session logs i.e. /var/log/webx/webx-session-manager/sessions");
            return false;
        }

        true
    }
}

