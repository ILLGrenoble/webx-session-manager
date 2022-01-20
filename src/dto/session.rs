use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::Session;

#[derive(Serialize, Deserialize, Clone)]
pub struct SessionDto {
    id: String,
    username: String,
    uid: u32,
    display_id: String,
    xorg_process_id: u32,
    window_manager_process_id: u32,
    xauthority_file_path: String,
}

#[allow(dead_code)]
impl SessionDto {
    pub fn new(
        id: String,
        username: String,
        uid: u32,
        display_id: String,
        xorg_process_id: u32,
        window_manager_process_id: u32,
        xauthority_file_path: String,
    ) -> Self {
        Self {
            id,
            username,
            uid,
            display_id,
            xorg_process_id,
            window_manager_process_id,
            xauthority_file_path,
        }
    }

    pub fn id(&self) -> &str {
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

    pub fn xorg_process_id(&self) -> u32 {
        self.xorg_process_id
    }

    pub fn window_manager_process_id(&self) -> u32 {
        self.window_manager_process_id
    }

    pub fn xauthority_file_path(&self) -> &str {
        &self.xauthority_file_path
    }
}

impl fmt::Display for SessionDto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionDto")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("uid", &self.uid)
            .field("display_id", &self.display_id)
            .field("xorg_process_id", &self.xorg_process_id)
            .field("window_manager_process_id", &self.window_manager_process_id)
            .field("xauthority_file_path", &self.xauthority_file_path)
            .finish()
    }
}


impl From<&Session> for SessionDto {
    fn from(session: &Session) -> Self {
        let username = session.username();
        let uid = session.uid();
        let display_id = session.display_id();
        let xorg_process_id = session.xorg().pid();
        let window_manager_process_id = session.window_manager().pid();

        let xauthority_file_path = session.xauthority_file_path();
        let id  = session.id().to_string();
        SessionDto::new(
            id,
            username.into(),
            uid,
            display_id.into(),
            xorg_process_id,
            window_manager_process_id,
            xauthority_file_path.into(),
        )
    }
}
