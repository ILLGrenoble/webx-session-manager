use std::fmt;

use nix::unistd::User;
use users::get_user_groups;

pub struct Account {
    username: String,
    home: String,
    uid: u32,
    gid: u32,
    groups: Vec<u32>
}

impl Account {
    pub fn new(username: &str, home: &str, uid: u32, gid: u32, groups: Vec<u32>) -> Self {
        Account {
            username: username.into(),
            home: home.into(),
            uid,
            gid,
            groups
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

    pub fn groups(&self) -> &[u32] {
        &self.groups
    }

    pub fn from_user(user: User) -> Option<Account> {
        let uid = user.uid.as_raw();
        let gid = user.gid.as_raw();
        let username = user.name.as_str();
        if let Some(home) = user.dir.to_str() {
            let groups: Vec<u32> = get_user_groups(username, gid)
            .unwrap_or_default()
            .iter()
            .filter(|group| {
                // only return the root group if the user is the root user
                if uid == 0 {
                    return true;
                }
                group.gid() > 0
            })
            .map(|group| group.gid())
            .collect();

            let account = Account::new(username, home, uid, gid, groups);
            return Some(account);
        }

        None
    }
}


impl fmt::Display for Account {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "username = {}, home = {} uid = {}, gid = {}, groups = {:?}", self.username, self.home, self.uid, self.gid, &self.groups)
    }
}
