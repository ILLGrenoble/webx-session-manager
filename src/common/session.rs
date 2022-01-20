use std::fmt;

use uuid::Uuid;

use crate::common::ProcessHandle;

#[derive(Clone)]
pub struct Session {
    id: Uuid,
    username: String,
    uid: u32,
    display_id: String,
    xauthority_file_path: String,
    xorg: ProcessHandle,
    window_manager: ProcessHandle
}

#[allow(dead_code)]
impl Session {
    pub fn new(
        id: Uuid,
        username: String,
        uid: u32,
        display_id: String,
        xauthority_file_path: String,
        xorg: ProcessHandle,
        window_manager: ProcessHandle
    ) -> Self {
        Self {
            id,
            username,
            uid,
            display_id,
            xauthority_file_path,
            xorg,
            window_manager,
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
        &self.xorg
    }
}

impl fmt::Display for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Session")
            .field("username", &self.username)
            .field("uid", &self.uid)
            .field("display_id", &self.display_id)
            .field("xauthority_file_path", &self.xauthority_file_path)
            .finish()
    }
}
