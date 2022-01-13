use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone)]
pub struct Session {
    username: String,
    uid: u32,
    display_id: String,
    process_id: i32,
    xauthority_file_path: String,
}

#[allow(dead_code)]
impl Session {
    pub fn new(
        username: String,
        uid: u32,
        display_id: String,
        process_id: i32,
        xauthority_file_path: String,
    ) -> Self {
        Self {
            username,
            uid,
            display_id,
            process_id,
            xauthority_file_path,
        }
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
    
    pub fn process_id(&self) -> i32 {
        self.process_id
    }

    pub fn xauthority_file_path(&self) -> &str {
        &self.xauthority_file_path
    }
}

impl fmt::Display for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Session")
            .field("username", &self.username)
            .field("uid", &self.uid)
            .field("display_id", &self.display_id)
            .field("process_id", &self.process_id)
            .field("xauthority_file_path", &self.xauthority_file_path)
            .finish()
    }
}
