


use embedded_hal::blocking::{i2c::{Read, Write, WriteRead}, delay::DelayMs};

const BACKLIGHT: u8 = 0b0000_1000;
const NO_BACKLIGHT: u8 = 0b0000_0000;

const ENABLE: u8 = 0b0000_0100;
// const READ_WRITE: u8 = 0b0000_0010; // Not used as no reading of the `HD44780` is done
const REGISTER_SELECT: u8 = 0b0000_0001;
const DISPLAY_CONTROL: u8 = 0b0000_1000;

const ONE_LINE: u8 = 0b0000_0000;
const TWO_LINE: u8 = 0b0000_1000;
const FOUR_BIT_MODE: u8 = 0b0000_0000;
const FIVE_X8_DOTS: u8 = 0b0000_0000;
const FIVE_X10_DOTS: u8 = 0b0000_0100;

const FUNCTION_SET: u8 = 0b0010_0000;

const DISPLAY_ON: u8 = 0b0000_0100;
const DISPLAY_OFF: u8 = 0b0000_0000;
const CURSOR_ON: u8 = 0b0000_0010;
const CURSOR_OFF: u8 = 0b0000_0000;
const BLINK_ON: u8 = 0b0000_0001;
const BLINK_OFF: u8 = 0b0000_0000;

const ENTRY_LEFT: u8 = 0b0000_0010;
const ENTRY_SHIFT_DECREMENT: u8 = 0b0000_0000;
const ENTRY_MODE_SET: u8 = 0b0000_0100;

const RETURN_HOME: u8 = 0b0000_0010;

const SET_DDRAM_ADDR: u8 = 0b1000_0000;

const INITIALIZE_4BIT: u8 = 0x33;

#[derive(Default)]
pub struct Lcd<I2cType, DelayType> {
    i2c: I2cType,
    address: u8,
    delay: DelayType,

    display_function: u8,
    display_control: u8,
    display_mode: u8,
    cols: u8,
    rows: u8,
    char_size: u8,
    back_light_val: u8,
}

impl<I2cType, DelayType> Lcd<I2cType, DelayType> 
where
    I2cType: Write,
    DelayType: DelayMs<u16>
{
    pub fn new(
        i2c: I2cType, 
        address: u8, 
        delay: DelayType,
        cols: u8,
        rows: u8,
        char_size: u8
    ) -> Self {
        Lcd {
            i2c,
            address,
            delay,
            display_function: 0,
            display_control: 0,
            display_mode: 0,
            cols,
            rows,
            char_size,
            back_light_val: BACKLIGHT,
        }
    }

    fn expander_write(&mut self, nibble: u8, data: bool) {
        let rs = match data {
            false => 0u8,
            true => REGISTER_SELECT
        };

        let byte = nibble | rs | self.back_light_val;

        let _ = self.i2c.write(self.address, &[byte, byte | ENABLE]);
        self.delay.delay_ms(2u16);
        let _ = self.i2c.write(self. address, &[byte]);
    }

    fn write(&mut self, byte: u8, data: bool) -> Result<(), ()> {
        let upper_nibble = byte & 0xF0;
        self.expander_write(upper_nibble, data);

        let lower_nibble = (byte & 0x0F) << 4;
        self.expander_write(lower_nibble, data);

        Ok(())
    }

    fn write_byte(&mut self, byte: u8) -> Result<(), ()> {
        self.write(byte, true)?;

        self.delay.delay_ms(1_u16);

        Ok(())
    }

    fn command(&mut self, cmd: u8) -> Result<(), ()> {
        self.write(cmd, false)?;

        self.delay.delay_ms(1_u16);

        Ok(())
    }

    pub fn init(&mut self) -> Result<(), ()> {
        self.display_function = FOUR_BIT_MODE | ONE_LINE | FIVE_X8_DOTS;

        if self.rows > 1 {
            self.display_function |= TWO_LINE;
        }

        if self.char_size != 0 && self.rows == 1 {
            self.display_function |= FIVE_X10_DOTS;
        }

        self.delay.delay_ms(15_u16);

        self.write(INITIALIZE_4BIT, false)?;
        self.delay.delay_ms(5u16);

        self.command(0x32)?;

        self.command(0x28)?;

        // Clear display

        self.command(0x0E)?;

        // Move the cursor to beginning of first line

        self.command(0x01)?;

        self.command(0x80)?;

        self.command(FUNCTION_SET | self.display_function)?;

        self.display_control = DISPLAY_ON | CURSOR_OFF | BLINK_OFF;
        self.display()?;

        self.clear()?;

        self.display_mode = ENTRY_LEFT | ENTRY_SHIFT_DECREMENT;
        self.command(ENTRY_MODE_SET | self.display_mode)?;

        self.home()?;

        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), ()> {
        self.command(0b0000_0001)?;

        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), ()> {
        self.command(0b0000_0010)?;

        Ok(())
    }

    pub fn backlight(&mut self) -> Result<(), ()> {
        self.back_light_val = BACKLIGHT;
        self.display()?;

        Ok(())
    }

    pub fn no_backlight(&mut self) -> Result<(), ()> {
        self.back_light_val = NO_BACKLIGHT;
        self.display()?;

        Ok(())
    }

    pub fn display(&mut self) -> Result<(), ()> {
        self.display_control |= (1 << 2);
        self.command(DISPLAY_CONTROL | self.display_control)?;

        Ok(())
    }

    pub fn no_display(&mut self) -> Result<(), ()> {
        self.display_control &= !(1 << 2);
        self.command(DISPLAY_CONTROL | self.display_control)?;

        Ok(())
    }

    pub fn home(&mut self) -> Result<(), ()> {
        self.command(RETURN_HOME)?;

        Ok(())
    }

    pub fn set_cursor(&mut self, col: u8, mut row: u8) -> Result<(), ()> {
        const ROW_OFFSETS: [u8; 4] = [0x00_u8, 0x40_u8, 0x14_u8, 0x54_u8];

        if row >= self.rows { row = self.rows - 1; }

        self.command(SET_DDRAM_ADDR | (col + ROW_OFFSETS[row as usize]))?;

        Ok(())
    }

    pub fn write_char(&mut self, data: char) -> Result<(), ()> {
        self.write_byte(data as u8)?;

        Ok(())
    }

    pub fn write_bytes(&mut self, data: &[u8]) -> Result<(), ()> {
        for &b in data {
            self.write_byte(b)?;
        }

        Ok(())
    }

    pub fn write_str(&mut self, data: &str) -> Result<(), ()> {
        self.write_bytes(data.as_bytes())?;

        Ok(())
    }
}
