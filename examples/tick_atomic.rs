#![no_main]
#![no_std]

use panic_halt as _;
use hal::{pac, prelude::*};
use cortex_m_rt::{entry, exception};
use cortex_m;

use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::Peripherals;

use core::sync::atomic::{AtomicU32, Ordering};

static SYS_TICK: AtomicU32 = AtomicU32::new(0);

fn systick_init(syst: &mut cortex_m::peripheral::SYST, rcc: &mut hal::rcc::Rcc) {
    syst.set_reload(rcc.clocks.sysclk().0 / 1000 - 1);//1ms
    syst.clear_current();
    syst.set_clock_source(SystClkSource::Core);
    syst.enable_counter();
    syst.enable_interrupt();
}

fn systick_elapse(last_tick: u32) -> u32
{
	let mut elapse;
    let curr_tick = SYS_TICK.load(Ordering::Relaxed);

    if curr_tick < last_tick {
        elapse = u32::MAX - last_tick + 1;
        elapse += curr_tick;
    } else {
        elapse = curr_tick - last_tick;
    }

    return elapse;
}


fn msdelay( ms: u32)
{
    let start = SYS_TICK.load(Ordering::Relaxed);

    while systick_elapse(start) < ms { }
}

#[entry]
fn main() -> ! {
    if let (Some(mut p), Some(cp)) = (pac::Peripherals::take(), Peripherals::take()) {
        let mut rcc = p.RCC.configure().hsi().freeze(&mut p.FLASH);
        let gpioc = p.GPIOC.split(&mut rcc);

        let mut syst = cp.SYST;
        /* (Re-)configure PA1 as output */

        let mut led1 = cortex_m::interrupt::free(|cs| gpioc.pc3.into_push_pull_output(cs));
        let mut led5 = cortex_m::interrupt::free(|cs| gpioc.pc7.into_push_pull_output(cs));

        led5.set_high().unwrap();

        systick_init(&mut syst, &mut rcc);
        loop {
            led1.toggle().unwrap();
            msdelay(100);
        }
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
    let sys_tick = SYS_TICK.load(Ordering::Relaxed);
    SYS_TICK.store(sys_tick.wrapping_add(1), Ordering::Relaxed);
}
