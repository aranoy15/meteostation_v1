#![no_std]
#![no_main]

use panic_halt as _;

use app::peripherals::usb as app_usb;
use app::system::clocks;
use app::system::systick;
use cortex_m_rt::entry;
use nb::block;
use stm32f1xx_hal::{i2c, prelude::*, stm32, usb};

#[entry]
fn main() -> ! {
    if let (Some(cp), Some(dp)) = (cortex_m::Peripherals::take(), stm32::Peripherals::take()) {
        systick::init(cp.SYST);

        let mut flash = dp.FLASH.constrain();
        let mut rcc = dp.RCC.constrain();
        let mut afio = dp.AFIO.constrain(&mut rcc.apb2);
        let cfgr = rcc.cfgr;

        let clocks = clocks::init(cfgr, &mut flash.acr);

        assert!(clocks.usbclk_valid());

        let gpioa = dp.GPIOA.split(&mut rcc.apb2);
        let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
        let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);

        let scl = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
        let sda = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);

        let i2c = i2c::BlockingI2c::i2c1(
            dp.I2C1,
            (scl, sda),
            &mut afio.mapr,
            i2c::Mode::Standard {
                frequency: 100_000.hz(),
            },
            clocks,
            &mut rcc.apb1,
            100_000,
            1,
            100_000,
            100_000,
        );

        //let mut local_delay = app::system::delay::Delay::new();

        let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

        let usb_peripheral = usb::Peripheral {
            usb: dp.USB,
            pin_dm: gpioa.pa11,
            pin_dp: gpioa.pa12,
        };

        app_usb::init(usb_peripheral);

        let led_delay = app::system::delay::Delay::new();
        let mut lcd = app::drivers::lcd::Lcd::new(
            i2c, 
            0x27, 
            led_delay, 
        )
        .columns(20)
        .rows(4)
        .char_size(1)
        .build();

        lcd.init().unwrap_or_default();
        lcd.reset().unwrap_or_default();
        lcd.clear().unwrap_or_default();

        lcd.no_backlight().unwrap_or_default();

        lcd.set_cursor(0, 0).unwrap_or_default();
        lcd.write_str("Hello, world!").unwrap_or_default();

        lcd.set_cursor(0, 1).unwrap_or_default();
        lcd.write_str("Broken").unwrap_or_default();

        lcd.set_cursor(0, 2).unwrap_or_default();
        lcd.write_str("Canceling").unwrap_or_default();

        lcd.set_cursor(0, 3).unwrap_or_default();
        lcd.write_bytes(&['1' as u8, '2' as u8, '3' as u8]).unwrap_or_default();

        let mut timer = app::system::timer::Timer::new();
        timer.start(100_u32);

        loop {
            match block!(timer.wait()) {
                Ok(()) => {
                    led.toggle().unwrap();
                    timer.reset();
                }
                Err(_) => {}
            }
        }
    }

    panic!();
}
