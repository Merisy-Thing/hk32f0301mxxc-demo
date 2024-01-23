#![no_main]
#![no_std]

use panic_halt as _;
use hal::{pac, prelude::*};
use cortex_m_rt::{entry, exception};
use cortex_m;

use hal::gpio::*;
use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::Peripherals;

use core::cell::RefCell;
use core::ops::DerefMut;

static GPIO: Mutex<RefCell<Option<gpioc::PC3<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));

fn systick_init(syst: &mut cortex_m::peripheral::SYST, rcc: &mut hal::rcc::Rcc) {
    syst.set_reload(rcc.clocks.sysclk().0 / 5 - 1);//100ms
    syst.clear_current();
    syst.set_clock_source(SystClkSource::Core);
    syst.enable_counter();
    syst.enable_interrupt();
}

#[entry]
fn main() -> ! {
    if let (Some(mut p), Some(cp)) = (pac::Peripherals::take(), Peripherals::take()) {
        let mut rcc = p.RCC.configure().hsi().freeze(&mut p.FLASH);
        let gpioc = p.GPIOC.split(&mut rcc);

        let mut syst = cp.SYST;
        /* (Re-)configure PA1 as output */

        let led1 = cortex_m::interrupt::free(|cs| gpioc.pc3.into_push_pull_output(cs));
        let mut led5 = cortex_m::interrupt::free(|cs| gpioc.pc7.into_push_pull_output(cs));

        led5.set_high().unwrap();

        cortex_m::interrupt::free(move |cs| {
            *GPIO.borrow(cs).borrow_mut() = Some(led1);
        });

        systick_init(&mut syst, &mut rcc);
    }

    loop {
        continue;
    }
}

/* Define an exception, i.e. function to call when exception occurs. Here if our SysTick timer
 * trips the flash function will be called and the specified stated passed in via argument */
//, flash, state: u8 = 1);
#[exception]
fn SysTick() {
    /* Enter critical section */
    cortex_m::interrupt::free(|cs| {
        if let Some(ref mut led) = *GPIO.borrow(cs).borrow_mut().deref_mut() {
            led.toggle().unwrap();
        }
    });
}
