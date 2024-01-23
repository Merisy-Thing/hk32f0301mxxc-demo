#![no_main]
#![no_std]

use panic_halt as _;
use hal::{pac, prelude::*, delay::Delay};
use cortex_m_rt::entry;
use cortex_m;

#[entry]
fn main() -> ! {
    if let Some(mut p) = pac::Peripherals::take() {
        let mut rcc = p.RCC.configure().hsi().freeze(&mut p.FLASH);
        let cm_p = cortex_m::peripheral::Peripherals::take().unwrap();

        //let mut delay = cortex_m::delay::Delay::new(cm_p.SYST, 48000000);
        let mut delay = Delay::new(cm_p.SYST, rcc.clocks);

        let gpioc = p.GPIOC.split(&mut rcc);
        let mut led1 = cortex_m::interrupt::free(|cs| gpioc.pc3.into_push_pull_output(cs));
        let mut led5 = cortex_m::interrupt::free(|cs| gpioc.pc7.into_push_pull_output(cs));

        led5.set_high().unwrap();
        loop {
            led1.set_high().unwrap();
            delay.delay_ms(800);

            led1.set_low().unwrap();
            delay.delay_ms(200);
        }
    }

    loop {
        continue;
    }
}
