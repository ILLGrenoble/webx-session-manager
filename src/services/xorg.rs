use std::fs::{self, File};
use std::os::unix::prelude::CommandExt;
use std::process::{Command};
use std::sync::Mutex;

use pam_client::env_list::EnvList;
use rand::Rng;
use uuid::Uuid;

use crate::common::{Account, ApplicationError, ProcessHandle, Session, XorgSettings, ScreenResolution};
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

    pub fn get_all_sessions(&self) -> Option<Vec<Session>> {
        if let Ok(sessions) = self.sessions.lock() {
            info!("Sessions: {}", sessions.len());
            return Some(sessions.to_vec());
        }
        None
    }

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
        let mut cleaned_up_total = 0;
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.retain(|session| {
                if session.xorg().is_running().is_err() {
                    true
                } else {
                    error!(
                        "removing session {} as the xorg server is no longer running",
                        session.id()
                    );
                    cleaned_up_total += 1;
                    false
                }
            });
        }
        cleaned_up_total
    }

     // Generate an xauth cookie
    // It must be a string of length 32 that can only contain hex values
    fn create_cookie(&self) -> String {
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
        session_id: &Uuid,
        display: u32,
        resolution: &ScreenResolution,
        account: &Account,
        environment: &EnvList
    ) -> Result<ProcessHandle, ApplicationError> {
        debug!("Launching x server on display :{}", display);
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
            session_id.to_simple()
        ))?;
        let stderr_file = File::create(&format!(
            "{}/{}.xorg.err.log",
            self.settings.log_path(),
            session_id.to_simple()
        ))?;

        let xdg_run_time_dir = format!("{}/{}", self.settings.authority_path(), account.uid());
        let (screen_width, screen_height) = resolution.split();
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
            .env_clear()
            .env("DISPLAY", display)
            .env("XAUTHORITY", authority_file_path)
            .env("HOME", account.home())
            .env("XORG_RUN_AS_USER_OK", "1")
            .env("XDG_RUNTIME_DIR", xdg_run_time_dir)
            .env("XRDP_START_WIDTH", screen_width.to_string())
            .env("XRDP_START_HEIGHT", screen_height.to_string())
            .envs(environment.iter_tuples())
            .current_dir(account.home())
            .stdout(std::process::Stdio::from(stdout_file))
            .stderr(std::process::Stdio::from(stderr_file))
            .uid(account.uid())
            .gid(account.gid());

        debug!("Spawning command: {}", format!("{:?}", command).replace("\"", ""));
        ProcessHandle::new(&mut command)
    }

    fn spawn_window_manager(
        &self,
        session_id: &Uuid,
        display: u32,
        account: &Account,
        environment: &EnvList
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
            session_id.to_simple()
        ))?;
        let stderr_file = File::create(&format!(
            "{}/{}.wm.err.log",
            self.settings.log_path(),
            session_id.to_simple()
        ))?;
        
        let xdg_run_time_dir = format!("{}/{}", self.settings.authority_path(), account.uid());

        let mut command = Command::new(self.settings.window_manager());

        command
            .env_clear()
            .env("DISPLAY", display)
            .env("XAUTHORITY", authority_file_path)
            .env("HOME", account.home())
            .env("XDG_RUNTIME_DIR", xdg_run_time_dir)
            .envs(environment.iter_tuples())
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
        account: &Account,
        resolution: ScreenResolution,
        environment: EnvList
    ) -> Result<Session, ApplicationError> {
        let display_id = self.get_next_display()?;
        self.create_token(display_id, account)?;

        let session_id = Uuid::new_v4();

        // spawn the x server and the window manager
        let xorg = self.spawn_x_server(&session_id, display_id, &resolution, account, &environment)?;       
        let window_manager = self.spawn_window_manager(&session_id, display_id, account, &environment)?;

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
            window_manager,
            resolution
        );
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.push(session.clone());
            return Ok(session);
        }
        return Err(ApplicationError::session(format!("Could not start session for user: {}", account)));
    }

    fn get_next_available_display(&self, id: u32) -> Result<u32, ApplicationError> {
        let lock_path = self.settings.lock_path();
        let path = format!("{}/.X{}-lock", lock_path, id);
        if fs::metadata(path).is_ok() {
            self.get_next_available_display(id + 1)
        } else {
            Ok(id)
        }
    }

    fn get_next_display(&self) -> Result<u32, ApplicationError> {
        let display_offset = self.settings.display_offset();
        self.get_next_available_display(display_offset)
    }
}
