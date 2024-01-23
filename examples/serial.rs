#![no_main]
#![no_std]

use cortex_m_rt::entry;
use cortex_m;
use panic_halt as _;
use hal::prelude::*;
use nb::block;
use hal::serial::Serial;
use embedded_hal_nb::serial::Write;
use embedded_hal_nb::serial::Read;

#[entry]
fn main() -> ! {
    if let Some(mut p) = hal::pac::Peripherals::take() {
        let mut rcc = p.RCC.configure().hsi().freeze(&mut p.FLASH);
        let gpioa = p.GPIOA.split(&mut rcc);

        let (_, _) = cortex_m::interrupt::free(
            |cs|  ( gpioa.pa2.into_alternate_af1(cs), gpioa.pa3.into_alternate_af1(cs) )
        );

        let mut serial = Serial::uart1(p.UART1, 115_200.bps(), rcc.clocks);

        loop {
            let received = block!(serial.read()).unwrap();
            block!(serial.write(received)).ok();
        }
    }

    loop {
        continue;
    }
}
