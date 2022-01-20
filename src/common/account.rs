use std::fmt;

use nix::unistd::User;

pub struct Account {
    username: String,
    home: String,
    uid: u32,
    gid: u32,
}

impl Account {
    pub fn new(username: &str, home: &str, uid: u32, gid: u32) -> Self {
        Account {
            username: username.into(),
            home: home.into(),
            uid,
            gid,
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn home(&self) -> &str {
        &self.home
    }

    pub fn uid(&self) -> u32 {
        self.uid
    }

    pub fn gid(&self) -> u32 {
        self.gid
    }

    pub fn from_user(user: User) -> Option<Account> {
        let uid = user.uid.as_raw();
        let gid = user.gid.as_raw();
        let username = user.name.as_str();
        if let Some(home) = user.dir.to_str() {
            let account = Account::new(username, home, uid, gid);
            return Some(account);
        }
        None
    }
}


impl fmt::Display for Account {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "username = {}, home = {} uid = {}, gid = {}", self.username, self.home, self.uid, self.gid)
    }
}
