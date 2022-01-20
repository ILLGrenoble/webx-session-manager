use std::fs::{self, File};
use std::os::unix::prelude::CommandExt;
use std::process::{Command};
use std::sync::Mutex;

use pam_client::env_list::EnvList;
use rand::Rng;
use regex::Regex;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::common::{Account, ApplicationError, ProcessHandle, Session, Xlock, XorgSettings};
use crate::fs::{chmod, chown, mkdir, touch};

pub struct XorgService {
    settings: XorgSettings,
    sessions: Mutex<Vec<Session>>,
}

impl XorgService {
    pub fn new(settings: XorgSettings) -> Self {
        let sessions = Mutex::new(Vec::new());
        Self { settings, sessions }
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
        debug!(
            "Creating xauth token for display {} and user {}",
            display,
            account.username()
        );
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

    fn spawn_x_server(
        &self,
        display: u32,
        environment: &EnvList,
        account: &Account,
    ) -> Result<ProcessHandle, ApplicationError> {
        debug!("launching x server on display :{}", display);
        let authority_file_path = format!(
            "{}/{}/webx-session-manager/Xauthority",
            self.settings.authority_path(),
            account.uid()
        );
        let display = format!(":{}", display);
        let config = self.settings.config_path();
        let stdout_file = File::create(&format!(
            "{}/{}.xorg.out.log",
            self.settings.log_path(),
            account.uid()
        ))?;
        let stderr_file = File::create(&format!(
            "{}/{}.xorg.err.log",
            self.settings.log_path(),
            account.uid()
        ))?;

        let mut command = Command::new("Xorg");
        command
            .args([
                display.as_str(),
                "-auth",
                authority_file_path.as_str(),
                "-config",
                config,
                "-verbose",
            ])
            .envs(environment.iter_tuples())
            .env("DISPLAY", display)
            .env("XAUTHORITY", authority_file_path)
            .env("HOME", account.home())
            .env("XORG_RUN_AS_USER_OK", "1")
            .env("XDG_RUNTIME_DIR", "/run/user/1001")
            .current_dir(account.home())
            .stdout(std::process::Stdio::from(stdout_file))
            .stderr(std::process::Stdio::from(stderr_file))
            .uid(account.uid())
            .gid(account.gid());

        debug!(
            "Spawning command: {}",
            format!("{:?}", command).replace("\"", "")
        );
        ProcessHandle::new(&mut command)
    }

    fn spawn_window_manager(
        &self,
        display: u32,
        account: &Account,
    ) -> Result<ProcessHandle, ApplicationError> {
        let authority_file_path = format!(
            "{}/{}/webx-session-manager/Xauthority",
            self.settings.authority_path(),
            account.uid()
        );
        let display = format!(":{}", display);

        let stdout_file = File::create(&format!(
            "{}/{}.wm.out.log",
            self.settings.log_path(),
            account.uid()
        ))?;
        let stderr_file = File::create(&format!(
            "{}/{}.wm.err.log",
            self.settings.log_path(),
            account.uid()
        ))?;
        let xdg_run_time_dir = format!("{}/{}", self.settings.authority_path(), account.uid());

        let mut command = Command::new(self.settings.window_manager());

        command
            .env("DISPLAY", display)
            .env("XAUTHORITY", authority_file_path)
            .env("HOME", account.home())
            .env("XDG_RUNTIME_DIR", xdg_run_time_dir)
            .current_dir(account.home())
            .stdout(std::process::Stdio::from(stdout_file))
            .stderr(std::process::Stdio::from(stderr_file))
            .uid(account.uid())
            .gid(account.gid());

        debug!(
            "Spawning command: {}",
            format!("{:?}", command).replace("\"", "")
        );
        ProcessHandle::new(&mut command)
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

    fn create_file<S>(&self, path: S, mode: u32, account: &Account) -> Result<(), ApplicationError>
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
        let display_id = self.get_next_display()?;
        self.create_token(display_id, account)?;
        let xorg = self.spawn_x_server(display_id, environment, account)?;
        let window_manager = self.spawn_window_manager(display_id, account)?;

        info!(
            "Running display {} on process id {} with window manager process id {}",
            display_id,
            xorg.pid(),
            window_manager.pid()
        );

        let authority_file_path = format!(
            "{}/{}/webx-session-manager/Xauthority",
            self.settings.authority_path(),
            account.uid()
        );

        let session = Session::new(
            Uuid::new_v4(),
            account.username().into(),
            account.uid(),
            format!(":{}", display_id),
            authority_file_path,
            xorg,
            window_manager
        );
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.push(session.clone());
            return Ok(session);
        }
        return Err(ApplicationError::session(format!(
            "Could not start session for user: {}",
            account
        )));
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

    pub fn create_session_id() -> Uuid {
        Uuid::new_v4()
    }

    pub fn get_next_display(&self) -> Result<u32, ApplicationError> {
        let display_offset = self.settings.display_offset();
        self.get_next_available_display(display_offset)
    }

    pub fn get_all_xlock_files(&self) -> Vec<Xlock> {
        debug!("Looking for lock files in: {}", self.settings.lock_path());
        let pattern = Regex::new(r"^.X(\d+)-lock$").unwrap();
        WalkDir::new(self.settings.lock_path())
            .into_iter()
            .filter_map(|lock_file| lock_file.ok())
            .filter_map(|lock_file| -> Option<Xlock> {
                let file_name = lock_file.file_name().to_str().unwrap();
                if let Some(captures) = pattern.captures(file_name) {
                    debug!("Found lock file: {}", file_name);
                    // lovely, isn't it?
                    let display_id = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();
                    if let Ok(contents) = fs::read_to_string(lock_file.path()) {
                        if let Ok(process_id) = contents.trim().parse::<i32>() {
                            let path = lock_file.path().to_str().unwrap();
                            let lock = Xlock::new(path, display_id, process_id);
                            return Some(lock);
                        }
                    }
                }
                None
            })
            .collect()
    }

    pub fn get_all_sessions(&self) -> Option<Vec<Session>> {
        if let Ok(sessions) = self.sessions.lock() {
            info!("Sessions: {}", sessions.len());
            return Some(sessions.to_vec());
        }
        None
    }

    // pub fn get_session_by_process_and_display_id(
    //     &self,
    //     display_id: u32,
    //     process_id: i32) -> Option<Session> {
    //     let system = System::new_all();
    //     if let Some(process) = system.process(process_id) {
    //         match User::from_uid(Uid::from_raw(process.uid)) {
    //             Ok(user) => {
    //                 if let Some(user) = user {
    //                     info!("found user for process: {}", user.name);
    //                     let username = user.name.as_str();
    //                     let authority_file_path = format!(
    //                         "{}/{}/webx-session-manager/Xauthority",
    //                         self.settings.authority_path(),
    //                         process.uid
    //                     );
    //                     let session = Session::new(
    //                         username.to_owned(),
    //                         process.uid,
    //                         format!(":{}", display_id),
    //                         process_id,
    //                         authority_file_path,
    //                     );
    //                     return Some(session);
    //                 }
    //             }
    //             Err(error) => {
    //                 error!("Error finding user id for user: {}", error);
    //             }
    //         }
    //     }
    //     None
    // }

    /// get the display for a given user
    pub fn get_session_for_user(&self, uid: u32) -> Option<Session> {
        debug!("Finding session for user id: {}", uid);
        if let Ok(sessions) = self.sessions.lock() {
            return sessions
                .iter()
                .find(|session|  session.uid() == uid)
                .cloned();
        }
        None
    }

    pub fn clean_up(&self) -> u32 {
        info!("Cleaning up zombie sessions");
        let mut cleaned_up_total = 0;
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.retain(|session| {
                if session.xorg().is_running().is_err() {
                    true
                } else {
                    info!(
                        "removing session {} as the xorg server is no longer running",
                        session
                    );
                    cleaned_up_total += 1;
                    false
                }
            });
        }
        cleaned_up_total
    }
}
