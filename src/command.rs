use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};

/// 1363 Commands

#[derive(Debug)]
#[allow(dead_code)]
pub enum DisplayModes {
    On,
    Off,
    Normal,
    Inverse,
}

// Transform command into a fixed size array of 7 u8 and the real length for sending
fn create_data_array(args: &[u8]) -> [u8; 15] {
    let mut data = [0; 15];
    for (i, &arg) in args.iter().enumerate() {
        data[i] = arg;
    }
    data
}
/// Commands
#[derive(Debug)]
#[allow(dead_code)]
pub enum Command {
    /// Write RAM until next command is received
    StartWrite,
    /// Read RAM until next command is received
    StartRead,
    /// Set contrast. Higher number is higher contrast. Default = 0x7F
    Contrast(u8),
    /// Turn entire display on. If set, all pixels will
    /// be set to on, if not, the value in memory will be used.
    AllOn(bool),
    /// Invert display.
    Invert(bool),
    /// Turn display on or off.
    Sleep(bool),
    // Set IREF to internal
    SetInternalIREF(bool),
    // Set column address
    SetColumnAddress(u8, u8),
    // Set row address
    SetRowAddress(u8, u8),
    /// Reverse columns from 127-0
    SegmentRemap(bool),
    /// Set multipex ratio from 15-63 (MUX-1)
    Multiplex(u8),
    /// Scan from COM[n-1] to COM0 (where N is mux ratio)
    ReverseComDir(bool),
    /// Set vertical shift
    DisplayOffset(u8),
    /// Set grayscale table
    SetGrayScale([u8; 15]),
    /// Set grayscale table to default linear
    SetLinearGrayScale,
    /// Set display start line from 0-63
    StartLine(u8),
    /// Setup com hardware configuration
    /// First value indicates sequential (false) or alternative (true)
    /// pin configuration.
    /// ComPinConfig(bool),
    /// Set up display clock.
    /// First value is oscillator frequency, increasing with higher value
    /// Second value is divide ratio - 1
    DisplayClockDiv(u8, u8),
    /// Set up phase 1 and 2 of precharge period. each value is from 0-63
    PreChargePeriod(u8, u8),
    /// Set Vcomh Deselect level
    VcomhDeselect(VcomhLevel),
    /// NOOP
    Noop,
}

impl Command {
    pub fn send<DI>(self, iface: &mut DI) -> Result<(), DisplayError>
    where
        DI: WriteOnlyDataCommand,
    {
        let (cmd, data, data_len) = match self {
            Command::StartWrite => (0x5C, create_data_array(&[]), 0),
            Command::StartRead => (0x75, create_data_array(&[]), 0),
            Command::Contrast(val) => (0xC1, create_data_array(&[val]), 1),
            Command::AllOn(on) => (0xA5 | (((!on) as u8) << 1), create_data_array(&[]), 0),
            Command::Invert(inv) => (0xA6 | (inv as u8), create_data_array(&[]), 0),
            Command::SetInternalIREF(enable) => {
                (0xAD, create_data_array(&[0x80 | ((enable as u8) << 4)]), 1)
            }
            Command::SetColumnAddress(start, end) => (0x15, create_data_array(&[start, end]), 2),
            Command::SetRowAddress(start, end) => (0x75, create_data_array(&[start, end]), 2),
            Command::StartLine(line) => (0xA1, create_data_array(&[line]), 1),
            Command::SegmentRemap(remap) => (0xA0, create_data_array(&[(remap as u8) << 1]), 2),
            Command::Multiplex(ratio) => (0xCA, create_data_array(&[ratio]), 1),
            Command::ReverseComDir(rev) => (0xA0, create_data_array(&[((rev as u8) << 4)]), 2),
            Command::DisplayOffset(offset) => (0xA2, create_data_array(&[offset]), 1),
            Command::SetGrayScale(table) => (0xB8, create_data_array(&table), 15),
            Command::SetLinearGrayScale => (0xB9, create_data_array(&[]), 0),
            Command::DisplayClockDiv(fosc, div) => (
                0xB3,
                create_data_array(&[((0xF & fosc) << 4) | (0xF & div)]),
                1,
            ),
            Command::PreChargePeriod(phase1, phase2) => (
                0xB1,
                create_data_array(&[((0xF & phase2) << 4) | (0xF & phase1)]),
                1,
            ),
            Command::VcomhDeselect(level) => (0xDB, create_data_array(&[(level as u8) << 2]), 1),
            Command::Sleep(sleep) => (0xAE | (sleep as u8), create_data_array(&[]), 0),
            Command::Noop => (0xA6, create_data_array(&[]), 0),
        };

        // Send command byte
        iface.send_commands(DataFormat::U8(&[cmd]))?;

        // Send data if there is any
        if data_len > 0 {
            for i in 0..data_len {
                iface.send_data(DataFormat::U8(&data[i..i + 1]))?;
            }
        }

        Ok(())
    }
}

/// Display page
#[derive(Debug, Clone, Copy)]
pub enum Page {
    /// Page 0
    Page0 = 0,
    /// Page 1
    Page1 = 1,
    /// Page 2
    Page2 = 2,
    /// Page 3
    Page3 = 3,
    /// Page 4
    Page4 = 4,
    /// Page 5
    Page5 = 5,
    /// Page 6
    Page6 = 6,
    /// Page 7
    Page7 = 7,
}

impl From<u8> for Page {
    fn from(val: u8) -> Page {
        match val / 8 {
            0 => Page::Page0,
            1 => Page::Page1,
            2 => Page::Page2,
            3 => Page::Page3,
            4 => Page::Page4,
            5 => Page::Page5,
            6 => Page::Page6,
            7 => Page::Page7,
            _ => panic!("Page too high"),
        }
    }
}

/// Frame interval
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum NFrames {
    /// 2 Frames
    F2 = 0b111,
    /// 3 Frames
    F3 = 0b100,
    /// 4 Frames
    F4 = 0b101,
    /// 5 Frames
    F5 = 0b000,
    /// 25 Frames
    F25 = 0b110,
    /// 64 Frames
    F64 = 0b001,
    /// 128 Frames
    F128 = 0b010,
    /// 256 Frames
    F256 = 0b011,
}

/// Vcomh Deselect level
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum VcomhLevel {
    /// 0.65 * Vcc
    V064 = 0b0000,
    /// 0.77 * Vcc
    V078 = 0b1101,
    /// 0.83 * Vcc
    V084 = 0b1111,
}
