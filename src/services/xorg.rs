use std::fs::{self};
use std::os::unix::prelude::CommandExt;
use std::process::{Command, Stdio};

use pam_client::env_list::EnvList;
use rand::Rng;
use regex::Regex;
use sysinfo::{System, SystemExt};
use walkdir::WalkDir;

use crate::common::{Account, ApplicationError, Session, Xlock, XorgSettings};
use crate::fs::{chmod, chown, mkdir, touch};
use nix::unistd::{User, Uid};

pub struct XorgService {
    settings: XorgSettings
}

impl XorgService {
    pub fn new(settings: XorgSettings) -> Self {
        Self {
            settings
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

    fn create_token(&self, display: u32, account: &Account) -> Result<(), ApplicationError> {
        debug!("Creating xauth token for display {} and user {}", display, account.username());
        let cookie = self.create_cookie();
        let file_path = format!(
            "{}/{}/webx-session-manager/Xauthority",
            self.settings.authority_path(),
            account.uid()
        );
        let display = format!(":{}", display);
        Command::new("xauth")
            .arg("-f")
            .arg(file_path)
            .arg("add")
            .arg(display)
            .arg(".")
            .arg(cookie)
            .uid(account.uid())
            .gid(account.gid())
            .output()?;
        Ok(())
    }

    fn create_x_server(
        &self,
        display: u32,
        environment: &EnvList,
        account: &Account,
    ) -> Result<u32, ApplicationError> {
        debug!("launching x server on display :{}", display);
        let authority_file_path = format!(
            "{}/{}/webx-session-manager/Xauthority",
            self.settings.authority_path(),
            account.uid()
        );
        let display = format!(":{}", display);
        let config = self.settings.config_path();
        let mut command = Command::new("Xorg");
        command.args([
            display.as_str(),
            "-auth",
            authority_file_path.as_str(),
            "-config",
            config,
            "-verbose"
        ])
        .envs(environment.iter_tuples())
        .env("DISPLAY", display)
        .env("XAUTHORITY", authority_file_path)
        .env("HOME", account.home())
        .env("XORG_RUN_AS_USER_OK", "1")
        .current_dir(account.home())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .uid(account.uid())
        .gid(account.gid());

        debug!("Spawning command: {}", format!("{:?}", command).replace("\"", ""));

        match command.spawn() {
            Ok(child) => Ok(child.id()),
            Err(error) => {
                return Err(ApplicationError::session(format!("Could not start x server :{}", error)));
            }
        }
    }

    fn launch_window_manager(
        &self,
        display: u32,
        account: &Account
    ) -> Result<u32, ApplicationError> {
        let authority_file_path = format!(
            "{}/{}/webx-session-manager/Xauthority",
            self.settings.authority_path(),
            account.uid()
        );
        let display = format!(":{}", display);
        let mut command = Command::new(self.settings.window_manager());
        command
        .env("DISPLAY", display)
        .env("XAUTHORITY", authority_file_path)
        .env("HOME", account.home())
        .current_dir(account.home())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .uid(account.uid())
        .gid(account.gid());

        debug!("Spawning command: {}", format!("{:?}", command).replace("\"", ""));

        match command.spawn() {
            Ok(child) => Ok(child.id()),
            Err(error) => {
                return Err(ApplicationError::session(format!("Could not start the window manager: {}", error)));
            }
        }
    }

    fn create_directory<S>(
        &self,
        path: S,
        mode: u32,
        account: &Account,
    ) -> Result<(), ApplicationError>
    where
        S: AsRef<str>,
    {
        mkdir(path.as_ref())?;
        // ensure permissions and ownership are correct
        chown(path.as_ref(), account.uid(), account.gid())?;
        chmod(path.as_ref(), mode)?;
        Ok(())
    }

    fn create_file<S>(
        &self,
        path: S,
        mode: u32,
        account: &Account,
    ) -> Result<(), ApplicationError>
    where
        S: AsRef<str>,
    {
        let path = path.as_ref();

        if fs::metadata(path).is_err() {
            touch(path)?;
        }

        // ensure permissions and ownership are correct
        chown(path, account.uid(), account.gid())?;
        chmod(path, mode)?;
        Ok(())
    }

    // create the required directories and files
    pub fn create_user_files(&self, account: &Account) -> Result<(), ApplicationError> {
        debug!("Creating user files for user: {}", account.username());
        self.create_directory(
            format!("{}/{}", self.settings.authority_path(), account.uid()),
            0o700,
            account,
        )?;
        self.create_directory(
            format!(
                "{}/{}/webx-session-manager",
                self.settings.authority_path(),
                account.uid()
            ),
            0o700,
            account,
        )?;
        self.create_file(
            format!(
                "{}/{}/webx-session-manager/Xauthority",
                self.settings.authority_path(),
                account.uid()
            ),
            0o700,
            account,
        )?;
        Ok(())
    }

    // create the xauth token and launch x11 server
    pub fn execute(
        &self,
        environment: &EnvList,
        account: &Account,
    ) -> Result<Session, ApplicationError> {
        let display_id = self.get_next_diplay()?;
        self.create_token(display_id, account)?;
        let process_id = self.create_x_server(display_id, environment, account)?;

        // let's launch te window manager...
        let window_manager_process_id = self.launch_window_manager(display_id, account)?;
        info!("Running display {} on process id {} with window manager process id {}", display_id, process_id, window_manager_process_id);
        if let Some(session) = self.get_session_by_process_and_display_id(display_id, process_id as i32) {
            return Ok(session);
        }
        // improve the error handling
        return Err(ApplicationError::session(format!("Could not start session for user: {}", account)));
    }

    pub fn get_next_available_display(&self, id: u32) -> Result<u32, ApplicationError> {
        let lock_path = self.settings.lock_path();
        let path = format!("{}/{}", lock_path, id);
        if fs::metadata(path).is_ok() {
            self.get_next_available_display(id + 1)
        } else {
            Ok(id)
        }
    }

    pub fn get_next_diplay(&self) -> Result<u32, ApplicationError> {
      let display_offset = self.settings.display_offset();
      self.get_next_available_display(display_offset)
    }

    pub fn get_all_xlock_files(&self) -> Vec<Xlock> {
        debug!("Looking for lock files in: {}", self.settings.lock_path());
        let pattern = Regex::new(r"^.X(\d+)-lock$").unwrap();
        WalkDir::new(self.settings.lock_path())
            .into_iter()
            .filter_map(|lock_file| lock_file.ok())
            .filter_map(|lock_file | -> Option<Xlock> {
                let file_name = lock_file.file_name().to_str().unwrap();
                if let Some(captures) = pattern.captures(file_name) {
                    debug!("Found lock file: {}", file_name);
                    // lovely, isn't it?
                    let display_id = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();
                    if let Ok(contents) = fs::read_to_string(lock_file.path()) {
                        if let Ok(process_id) = contents.trim().parse::<i32>() {
                            let path = lock_file.path().to_str().unwrap();
                            let lock = Xlock::new(path,  display_id, process_id);
                            return Some(lock);
                        }
                    }
                }
                None
            })
            .collect()

    }

    pub fn get_all_sessions(&self) -> Vec<Session> {
        self.get_all_xlock_files()
        .into_iter()
        .filter_map(|file| {
            let display_id = file.display_id();
            let process_id = file.process_id();
            debug!("Looking for session with: {} {}", display_id, process_id);
            self.get_session_by_process_and_display_id(display_id, process_id)
        })
        .collect()
    }

    pub fn get_session_by_process_and_display_id(
        &self,
        display_id: u32,
        process_id: i32) -> Option<Session> {
        let system = System::new_all();
        if let Some(process) = system.process(process_id) {
            match User::from_uid(Uid::from_raw(process.uid)) {
                Ok(user) => {
                    if let Some(user) = user {
                        info!("found user for process: {}", user.name);
                        let username = user.name.as_str();
                        let authority_file_path = format!(
                            "{}/{}/webx-session-manager/Xauthority",
                            self.settings.authority_path(),
                            process.uid
                        );
                        let session = Session::new(
                            username.to_owned(),
                            process.uid,
                            format!(":{}", display_id),
                            process_id,
                            authority_file_path,
                        );
                        return Some(session);
                    }
                }
                Err(error) => {
                    error!("Error finding user id for user: {}", error);
                }
            }
        }
        None
    }

   
    /// get the display for a given user
    pub fn get_session_for_user(&self, uid: u32) -> Option<Session> {
        debug!("Finding session for user id: {}", uid);
        let sessions = self.get_all_sessions();   
        sessions.into_iter().find(|session| {
            info!("{} = {}", session.uid(), uid);
            return session.uid() == uid;
        })
    }

    pub fn clean_up(&self) -> u32 {
        info!("Cleaning up zombie sessions");
        let mut cleaned_up_total = 0;
        for lock_file in self.get_all_xlock_files() {
            let display_id = lock_file.display_id();
            let process_id = lock_file.process_id();
            let path = lock_file.path();
            if self.get_session_by_process_and_display_id(display_id, process_id).is_none() {
                 if fs::remove_file(path).is_err() {
                    error!("Could not remove zombie display session {}", &lock_file);
                } else {
                    debug!("Cleaned up zombie display session {}", &lock_file);
                    cleaned_up_total += 1;
                }
            }
        }
        cleaned_up_total
    }
}
