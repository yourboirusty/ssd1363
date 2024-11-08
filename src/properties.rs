//! Container to store and set display properties

use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};

use crate::{command::Command, displayrotation::DisplayRotation, displaysize::DisplaySize};

/// Display properties struct
pub struct DisplayProperties<DI> {
    iface: DI,
    display_size: DisplaySize,
    display_rotation: DisplayRotation,
    draw_column: u8,
    draw_row: u8,
    draw_row_end: u8,
    draw_column_end: u8,
}

impl<DI> DisplayProperties<DI>
where
    DI: WriteOnlyDataCommand,
{
    /// Create new DisplayProperties instance
    pub fn new(
        iface: DI,
        display_size: DisplaySize,
        display_rotation: DisplayRotation,
    ) -> DisplayProperties<DI> {
        DisplayProperties {
            iface,
            display_size,
            display_rotation,
            draw_column: 0,
            draw_row: 0,
            draw_column_end: 0,
            draw_row_end: 0,
        }
    }

    /// Initialise the display in column mode (i.e. a byte walks down a column of 8 pixels) with
    /// column 0 on the left and column _(display_width - 1)_ on the right.
    pub fn init_column_mode(&mut self) -> Result<(), DisplayError> {
        let display_rotation = self.display_rotation;

        Command::DisplayClockDiv(0xa, 0x0).send(&mut self.iface)?;

        self.set_rotation(display_rotation)?;

        Command::Contrast(0x6f).send(&mut self.iface)?;
        Command::PreChargePeriod(0x3, 0xd).send(&mut self.iface)?;
        Command::Sleep(true).send(&mut self.iface)?;

        Ok(())
    }

    /// Set the position in the framebuffer of the display where any sent data should be
    /// drawn. This method can be used for changing the affected area on the screen as well
    /// as (re-)setting the start point of the next `draw` call.
    pub fn set_draw_area(&mut self, start: (u8, u8), end: (u8, u8)) -> Result<(), DisplayError> {
        self.draw_column = start.0;
        self.draw_row = start.1;
        self.draw_column_end = end.0;
        self.draw_row_end = end.1;

        self.send_draw_address()
    }

    /// Send the data to the display for drawing at the current position in the framebuffer
    /// and advance the position accordingly. Cf. `set_draw_area` to modify the affected area by
    /// this method.
    pub fn draw(&mut self, buffer: &[u8]) -> Result<(), DisplayError> {
        let mut iter = buffer.iter().cloned();

        Command::StartWrite.send(&mut self.iface)?;
        self.iface.send_data(DataFormat::U8Iter(&mut iter))?;
        Command::Noop.send(&mut self.iface)?;

        Ok(())
    }

    fn send_draw_address(&mut self) -> Result<(), DisplayError> {
        Command::SetRowAddress(self.draw_row.into(), self.draw_row_end.into())
            .send(&mut self.iface)?;
        Command::SetColumnAddress(self.draw_column.into(), self.draw_column_end.into())
            .send(&mut self.iface)?;
        Ok(())
    }

    /// Get the configured display size
    pub fn get_size(&self) -> DisplaySize {
        self.display_size
    }

    /// Get display dimensions, taking into account the current rotation of the display
    ///
    /// ```rust
    ///# #[path = "test_helpers.rs"]
    ///# mod test_helpers;
    ///# use test_helpers::StubInterface;
    ///# let interface = StubInterface;
    /// use 1363::prelude::*;
    ///
    /// let mut display: GraphicsMode<_> = 1363::Builder::new().connect(interface).into();
    /// assert_eq!(display.get_dimensions(), (128, 64));
    ///
    /// # let interface = StubInterface;
    /// let mut rotated_display: GraphicsMode<_> = 1363::Builder::new().with_rotation(DisplayRotation::Rotate90).connect(interface).into();
    /// assert_eq!(rotated_display.get_dimensions(), (64, 128));
    /// ```
    pub fn get_dimensions(&self) -> (u8, u8) {
        let (w, h) = self.display_size.dimensions();

        match self.display_rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => (w, h),
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => (h, w),
        }
    }

    /// Get the display rotation
    pub fn get_rotation(&self) -> DisplayRotation {
        self.display_rotation
    }

    /// Set the display rotation
    pub fn set_rotation(&mut self, display_rotation: DisplayRotation) -> Result<(), DisplayError> {
        self.display_rotation = display_rotation;

        match display_rotation {
            DisplayRotation::Rotate0 => {
                Command::SegmentRemap(true).send(&mut self.iface)?;
                Command::ReverseComDir(true).send(&mut self.iface)
            }
            DisplayRotation::Rotate90 => {
                Command::SegmentRemap(false).send(&mut self.iface)?;
                Command::ReverseComDir(true).send(&mut self.iface)
            }
            DisplayRotation::Rotate180 => {
                Command::SegmentRemap(false).send(&mut self.iface)?;
                Command::ReverseComDir(false).send(&mut self.iface)
            }
            DisplayRotation::Rotate270 => {
                Command::SegmentRemap(true).send(&mut self.iface)?;
                Command::ReverseComDir(false).send(&mut self.iface)
            }
        }
    }

    /// Turn the display on or off. The display can be drawn to and retains all
    /// of its memory even while off.
    pub fn display_on(&mut self, on: bool) -> Result<(), DisplayError> {
        Command::Sleep(on).send(&mut self.iface)
    }

    /// Set the display contrast
    pub fn set_contrast(&mut self, contrast: u8) -> Result<(), DisplayError> {
        Command::Contrast(contrast).send(&mut self.iface)
    }
}
