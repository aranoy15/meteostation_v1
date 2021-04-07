
const APP_ADDRESS: u32 = 0x8004000;

pub fn start_application(scb: &mut cortex_m::peripheral::SCB) -> ! {
    cortex_m::interrupt::disable();

    unsafe  {
        scb.vtor.write(APP_ADDRESS);
        cortex_m::asm::bootload(APP_ADDRESS as *const u32);
    }
}
