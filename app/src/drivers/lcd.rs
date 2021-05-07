


use embedded_hal::blocking::{i2c::{Read, Write, WriteRead}, delay::DelayMs};

const BACKLIGHT: u8 = 0b0000_1000;
const ENABLE: u8 = 0b0000_0100;
// const READ_WRITE: u8 = 0b0000_0010; // Not used as no reading of the `HD44780` is done
const REGISTER_SELECT: u8 = 0b0000_0001;

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
            cols: cols,
            rows: rows,
            char_size: char_size,
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

    pub fn init(&mut self) -> Result<(), ()> {
        self.delay.delay_ms(15_u16);

        self.write(INITIALIZE_4BIT, false)?;

        self.delay.delay_ms(5u16);

        self.write(0x32, false)?;

        self.delay.delay_ms(1u16);

        self.write(0x28, false)?;

        self.delay.delay_ms(1u16);

        self.write(0x0E, false)?;

        self.delay.delay_ms(1u16);

        self.write(0x01, false)?;

        self.delay.delay_ms(1u16);

        Ok(())
    }
}
