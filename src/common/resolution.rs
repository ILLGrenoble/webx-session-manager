use std::fmt;

#[derive(Clone)]
pub struct ScreenResolution {
    width: u32,
    height: u32
}


impl ScreenResolution {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn split(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

impl fmt::Display for ScreenResolution {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}x{}", self.width, self.height)

    }
}
