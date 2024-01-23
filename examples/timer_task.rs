#![no_main]
#![no_std]

use cortex_m_rt::{entry, exception};
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::Peripherals;
use panic_halt as _;
use hal::prelude::*;
//use nb::block;
use hal::serial::Serial;
use serial_log::{ println };
use core::{sync::atomic::{AtomicU32, Ordering},};

static SYS_TICK: AtomicU32 = AtomicU32::new(0);
#[derive(Debug, Clone, Copy)]
struct Timer<F: FnMut()> {
    pub period: u32,
    pub cb: F,
}

const TIMER_LIST_SIZE: usize = 2;

static mut TIMER_TICK: [u32; TIMER_LIST_SIZE] = [0, 0];
const TIMER_LIST: [Timer<fn()>; TIMER_LIST_SIZE] = [
    Timer {
        period: 500,
        cb: time0_cb,
    },
    Timer {
        period: 1000,
        cb: time1_cb,
    },
];

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

fn time0_cb() {
    println!("time0_cb {}", SYS_TICK.load(Ordering::Relaxed));
}

fn time1_cb() {
    println!("time1_cb {}", SYS_TICK.load(Ordering::Relaxed));
}

fn msdelay( ms: u32)
{
    let start = SYS_TICK.load(Ordering::Relaxed);

    while systick_elapse(start) < ms { }
}

#[entry]
fn main() -> ! {
    if let Some(mut p) = hal::pac::Peripherals::take() {
        let mut rcc = p.RCC.configure().hsi().freeze(&mut p.FLASH);
        let gpiod = p.GPIOD.split(&mut rcc);
        //let cm_p = cortex_m::peripheral::Peripherals::take().unwrap();
        //let mut delay = Delay::new(cm_p.SYST, rcc.clocks);

        let cp = Peripherals::take().unwrap();

        cortex_m::interrupt::free(
            |cs|  gpiod.pd2.into_alternate_af1(cs)
        );

        let _serial = Serial::uart1(p.UART1, 115_200.bps(), rcc.clocks);
        let mut syst = cp.SYST;
        systick_init(&mut syst, &mut rcc);

        loop {
            for i in 0..TIMER_LIST_SIZE {
                if systick_elapse(unsafe { TIMER_TICK[i] }) >= TIMER_LIST[i].period {
                    unsafe { TIMER_TICK[i] = SYS_TICK.load(Ordering::Relaxed) };
                    (TIMER_LIST[i].cb)();
                }
            }
            msdelay(100_u32);
        }
    }

    loop {
        continue;
    }
}

#[exception]
fn SysTick() {
    let sys_tick = SYS_TICK.load(Ordering::Relaxed);
    SYS_TICK.store(sys_tick.wrapping_add(1), Ordering::Relaxed);
}
