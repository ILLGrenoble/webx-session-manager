use std::fmt;

pub struct Xlock {
    path: String,
    display_id: u32,
    process_id: i32
}


impl Xlock {
    pub fn new(path: &str, display_id: u32, process_id: i32) -> Self{
        Self {
            path: path.into(),
            display_id,
            process_id
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn display_id(&self) -> u32 {
        self.display_id
    }
    
    pub fn process_id(&self) -> i32 {
        self.process_id
    }

}



impl fmt::Display for Xlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "display_id = {}, process_id = {} path = {}", self.display_id, self.process_id, self.path)
    }
}

