#![no_main]
#![no_std]

use cortex_m_rt::entry;
use cortex_m;
use hal::delay::Delay;
use panic_halt as _;
use hal::prelude::*;
//use nb::block;
use hal::serial::Serial;

use serial_log as _;
use defmt;

#[entry]
fn main() -> ! {
    if let Some(mut p) = hal::pac::Peripherals::take() {
        let mut rcc = p.RCC.configure().hsi().freeze(&mut p.FLASH);
        let gpiod = p.GPIOD.split(&mut rcc);
        let cm_p = cortex_m::peripheral::Peripherals::take().unwrap();
        let mut delay = Delay::new(cm_p.SYST, rcc.clocks);

        cortex_m::interrupt::free(
            |cs|  gpiod.pd2.into_alternate_af1(cs)
        );

        let _serial = Serial::uart1(p.UART1, 115_200.bps(), rcc.clocks);

        loop {
            defmt::println!("Hello ");
            defmt::println!("World!");

            for i in 0..10 {
                delay.delay_ms(100u32);
                defmt::println!("{} ", i);
            }
            
            delay.delay_ms(1000u32);
        }
    }

    loop {
        continue;
    }
}
