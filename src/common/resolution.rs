use std::fmt;

/// The `ScreenResolution` struct represents the screen resolution for a session.
#[derive(Clone)]
pub struct ScreenResolution {
    width: u32,
    height: u32
}

impl ScreenResolution {
    /// Creates a new `ScreenResolution` instance.
    ///
    /// # Arguments
    /// * `width` - The width of the screen in pixels.
    /// * `height` - The height of the screen in pixels.
    ///
    /// # Returns
    /// A new `ScreenResolution` instance.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height
        }
    }

    /// Returns the width of the screen in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the screen in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Splits the resolution into its width and height components.
    ///
    /// # Returns
    /// A tuple containing the width and height.
    pub fn split(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

impl fmt::Display for ScreenResolution {
    /// Formats the `ScreenResolution` as a string in the format "widthxheight".
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}x{}", self.width, self.height)
    }
}
