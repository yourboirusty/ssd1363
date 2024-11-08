//! Display size

/// Display size enumeration
#[derive(Clone, Copy)]
pub enum DisplaySize {
    /// 128 by 64 pixels
    Display256x128,
}

impl DisplaySize {
    /// Get integral dimensions from DisplaySize
    pub fn dimensions(self) -> (u8, u8) {
        match self {
            DisplaySize::Display256x128 => (255, 127),
        }
    }

    /// Get the panel column offset from DisplaySize
    pub fn column_offset(self) -> u8 {
        match self {
            DisplaySize::Display256x128 => 0,
        }
    }
}
