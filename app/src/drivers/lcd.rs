
const CLEAR_DISPLAY: u8 = 0x01;
const RETURN_HOME: u8 = 0x02;
const ENTRY_MODE_SET: u8 = 0x04;
const DISPLAY_CONTROL: u8 = 0x08;
const CURSOR_SHIFT: u8 = 0x10;
const FUNCTION_SET: u8 = 0x20;
const SET_CGRAM_ADDR: u8 = 0x40;
const SET_DDRAM_ADDR: u8 = 0x80;

const ENTRY_RIGHT: u8 = 0x00;
const ENTRY_LEFT: u8 = 0x02;
const ENTRY_SHIFT_INCREMENT: u8 = 0x01;
const ENTRY_SHIFT_DECREMENT: u8 = 0x00;

const DISPLAY_ON: u8 = 0x04;
const DISPLAY_OFF: u8 = 0x00;
const CURSOR_ON: u8 = 0x02;
const CURSOR_OFF: u8 = 0x00;
const BLINK_ON: u8 = 0x01;
const BLINK_OFF: u8 = 0x00;

const DISPLAY_MOVE: u8 = 0x08;
const CURSOR_MOVE: u8 = 0x00;
const MOVE_RIGHT: u8 = 0x04;
const MOVE_LEFT: u8 = 0x00;


const EIGHT_BIT_MODE: u8 = 0x10;
const FOUR_BIT_MODE: u8 = 0x00;
const TWO_LINE: u8 = 0x08;
const ONE_LINE: u8 = 0x00;
const FIVE_x10_DOTS: u8 = 0x04;
const FIVE_x8_DOTS: u8 = 0x00;

const BACKLIGHT: u8 = 0x08;
const NO_BACKLIGHT: u8 = 0x00;

const EN: u8 = 1 << 2;
const RW: u8 = 1 << 1;
const RS: u8 = 1 << 0;

use embedded_hal::blocking::{i2c::{Read, Write, WriteRead}, delay::DelayMs};

#[derive(Default)]
struct Lcd<I2cType, DelayType> {
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
    I2cType: Read + Write + WriteRead,
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
            cols: cols,
            rows: rows,
            char_size: char_size,
            back_light_val: BACKLIGHT,
        }
    }

    fn expander_write(&mut self, data: u8) -> Result<(), ()> {
        match self.i2c.write(self.address, &[data]) {
            Ok(_) => { return Ok(());},
            Err(_) => { return Err(());}
        }
    }

    fn pulse_enable(&mut self, data: u8) -> Result<(), ()> {
        self.expander_write(data)?;
        self.delay.delay_ms(1);

        self.expander_write(data & (!EN))?;
        self.delay.delay_ms(1);

        Ok(())
    }

    fn write_4bits(&mut self, data: u8) -> Result<(), ()> {
        self.expander_write(data)?;
        self.pulse_enable(data)?;

        Ok(())
    }

    fn send(&mut self, value: u8, mode: u8) -> Result<(), ()> {
        let high = value & 0xF0;
        let low = (value << 4) & 0xF0;

        self.write_4bits(high | mode)?;
        self.write_4bits(low | mode)?;

        Ok(())
    }

    fn command(&mut self, value: u8) -> Result<(), ()> {
        self.send(value, 0)
    }
}
