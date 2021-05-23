use stm32f1xx_hal as hal;

use hal::{stm32, prelude::*, i2c, gpio::{gpiob::{PB6, PB7}, OpenDrain, Alternate}};
use stm32f1xx_hal::rcc::RccExt;
use stm32f1xx_hal::gpio::GpioExt;

type I2C1SclType = PB6<Alternate<OpenDrain>>;
type I2C1SdaType = PB7<Alternate<OpenDrain>>;
pub type I2C1Type = i2c::BlockingI2c<stm32::I2C1, (I2C1SclType, I2C1SdaType)>;

fn peripherals() -> stm32::Peripherals {
    unsafe {
        stm32::Peripherals::steal()
    }
}

pub fn i2c1(clocks: hal::rcc::Clocks) -> I2C1Type {
    let dp = peripherals();

    let mut rcc = dp.RCC.constrain();
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    let scl = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
    let sda = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);

    i2c::BlockingI2c::i2c1(
        dp.I2C1,
        (scl, sda),
        &mut afio.mapr,
        stm32f1xx_hal::i2c::Mode::Standard {
            frequency: 100_000.hz(),
        },
        clocks,
        &mut rcc.apb1,
        100_000,
        1,
        100_000,
        100_000,
    )
}
