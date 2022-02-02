use std::fmt;

use uuid::Uuid;

use crate::common::ProcessHandle;

use super::ScreenResolution;

#[derive(Clone)]
pub struct Session {
    id: Uuid,
    username: String,
    uid: u32,
    display_id: String,
    xauthority_file_path: String,
    xorg: ProcessHandle,
    window_manager: ProcessHandle,
    resolution: ScreenResolution,
}

#[allow(dead_code)]
impl Session {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        username: String,
        uid: u32,
        display_id: String,
        xauthority_file_path: String,
        xorg: ProcessHandle,
        window_manager: ProcessHandle,
        resolution: ScreenResolution,
    ) -> Self {
        Self {
            id,
            username,
            uid,
            display_id,
            xauthority_file_path,
            xorg,
            window_manager,
            resolution,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn uid(&self) -> u32 {
        self.uid
    }

    pub fn display_id(&self) -> &str {
        &self.display_id
    }

    pub fn xauthority_file_path(&self) -> &str {
        &self.xauthority_file_path
    }

    pub fn xorg(&self) -> &ProcessHandle {
        &self.xorg
    }

    pub fn window_manager(&self) -> &ProcessHandle {
        &self.window_manager
    }

    pub fn resolution(&self) -> &ScreenResolution {
        &self.resolution
    }
}

impl fmt::Display for Session {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Session")
            .field("username", &self.username)
            .field("uid", &self.uid)
            .field("display_id", &self.display_id)
            .field("xauthority_file_path", &self.xauthority_file_path)
            .field("resolution", &format!("{}", &self.resolution))
            .field("xorg pid", &self.xorg.pid())
            .field("window_manager pid", &self.window_manager.pid())
            .finish()
    }
}
