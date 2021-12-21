use std::fs;
use std::os::unix::prelude::CommandExt;
use std::process::{Command, Stdio};

use pam_client::env_list::EnvList;
use rand::Rng;

use crate::common::{Account, ApplicationError};
use crate::fs::{chmod, chown, mkdir, touch};

pub struct Xorg {
    base_directory: String,
    account: Account,
}

impl Xorg {
    pub fn new(base_directory: &str, account: Account) -> Self {
        Self {
            base_directory: base_directory.into(),
            account,
        }
    }

    // Generate an xauth cookie
    // It must be a string of length 32 that can only contain hex values
    pub fn create_cookie(&self) -> String {
        let characters: &[u8] = b"ABCDEF0123456789";
        let mut rng = rand::thread_rng();
        (0..32)
            .map(|_| {
                let idx = rng.gen_range(0..characters.len());
                characters[idx] as char
            })
            .flat_map(|c| c.to_lowercase())
            .collect()
    }

    pub fn create_token(&self, display: u32) -> Result<(), ApplicationError> {
        // debug!("Creating xauth token for display {}", display);
        let cookie = self.create_cookie();
        let file_path = format!("{}/{}/webx-session-manager/Xauthority", self.base_directory, self.account.uid());
        let display = format!(":{}", display);
        Command::new("xauth")
            .arg("-f")
            .arg(file_path)
            .arg("add")
            .arg(display)
            .arg(".")
            .arg(cookie)
            .uid(self.account.uid())
            .gid(self.account.gid())
            .output()?;
        Ok(())
    }

    pub fn create_x_server(&self, display: u32, environment: &EnvList) -> Result<(), ApplicationError> {
        debug!("launching x server on display :{}", display);
        let authority_file_path = format!("{}/{}/webx-session-manager/Xauthority", self.base_directory, self.account.uid());
        let display = format!(":{}", display);
        let config = "/data/xorg-dummy.conf";
        if let Err(error) = Command::new("Xorg")
            .args([
                display.as_str(),
                "-auth",
                authority_file_path.as_str(),
                "-config", config
            ])
            .envs(environment.iter_tuples())
            .env("DISPLAY", display)
            .env("XAUTHORITY", authority_file_path)
            .env("HOME", &self.account.home())
            // .env("XORG_RUN_AS_USER_OK", "1")
            .stdout(Stdio::piped())
            .uid(self.account.uid())
            .gid(self.account.gid())
            .spawn()
        {
            return Err(ApplicationError::Command(format!("Could not start x server :{}", error.to_string())));
        }
        Ok(())
    }

    pub fn create_directory<S>(&self, path: S, mode: u32) -> Result<(), ApplicationError>
        where S: AsRef<str> {
        mkdir(path.as_ref())?;
        // ensure permissions and ownership are correct
        chown(path.as_ref(), self.account.uid(), self.account.gid())?;
        chmod(path.as_ref(), mode)?;
        Ok(())
    }

    pub fn create_file<S>(&self, path: S, mode: u32) -> Result<(), ApplicationError>
        where S: AsRef<str> {
        let path = path.as_ref();

        if fs::metadata(path).is_err() {
            touch(path)?;
        }

        // ensure permissions and ownership are correct
        chown(path, self.account.uid(), self.account.gid())?;
        chmod(path, mode)?;
        Ok(())
    }

    // create the required directories and files
    pub fn setup(&self) -> Result<(), ApplicationError> {
        self.create_directory(format!("{}/{}", self.base_directory, self.account.uid()), 0o700)?;
        self.create_directory(format!("{}/{}/webx-session-manager", self.base_directory, self.account.uid()), 0o700)?;
        self.create_file(format!("{}/{}/webx-session-manager/Xauthority", self.base_directory, self.account.uid()), 0o700)?;
        Ok(())
    }

    // create the xauth token and launch x11 server
    pub fn execute(&self, display: u32, environment: &EnvList) -> Result<(), ApplicationError> {
        self.create_token(display)?;
        self.create_x_server(display, environment)?;
        Ok(())
    }

    pub fn get_next_available_display(&self, id: u32) -> Result<u32, ApplicationError> {
        let path = format!("/tmp/.X11-unix/X{}", id);
        if fs::metadata(path).is_ok() {
            self.get_next_available_display(id + 1)
        } else {
            Ok(id)
        }
    }
    pub fn get_current_display(&self) -> Option<u32> {
        if let Ok(output) = std::process::Command::new("ls")
            .args(vec!["-l", "/tmp/.X11-unix/"])
            .output()
        {
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                log::debug!("  {}", line);
                let mut iter = line.split_whitespace();
                let user_field = iter.nth(2).unwrap_or("");
                if let Some(x) = iter.last() {
                    if x.starts_with('X') {
                        if let Ok(display_id) = x.replace("X", "").parse::<u32>() {
                            if user_field == self.account.username() {
                                return Some(display_id);
                            }
                        }
                    }
                }
            }
        }
        None
    }
}