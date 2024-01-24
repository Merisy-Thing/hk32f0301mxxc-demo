#![no_main]
#![no_std]

use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::Peripherals;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::exception;
use hal::prelude::*;
use panic_halt as _;
use core::cell::Cell;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use hal::serial::Serial;
use serial_log::println;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embassy_time_driver::{AlarmHandle, Driver};

static SYS_TICK: AtomicU32 = AtomicU32::new(0);

fn systick_init(syst: &mut cortex_m::peripheral::SYST, rcc: &mut hal::rcc::Rcc) {
    syst.set_reload(rcc.clocks.sysclk().0 / 1000 - 1); //1ms
    syst.clear_current();
    syst.set_clock_source(SystClkSource::Core);
    syst.enable_counter();
    syst.enable_interrupt();
}

struct AlarmState {
    timestamp: Cell<u64>,
    callback: Cell<Option<(fn(*mut ()), *mut ())>>,
}
unsafe impl Send for AlarmState {}

const DUMMY_ALARM: AlarmState = AlarmState {
    timestamp: Cell::new(0),
    callback: Cell::new(None),
};

struct TimerDriver {
    alarms: Mutex<AlarmState>,
    allocated: AtomicBool,
}

embassy_time_driver::time_driver_impl!(static DRIVER: TimerDriver = TimerDriver{
    alarms:  Mutex::new(DUMMY_ALARM),
    allocated: AtomicBool::new(false),
});

impl TimerDriver {
    fn check_alarm(&self, curr_tick: u64) {
        cortex_m::interrupt::free(|cs| {
            let allocated = self.allocated.load(Ordering::Relaxed);
            if !allocated {
                return;
            }
            let alarm = &self.alarms.borrow(cs);

            let timestamp = alarm.timestamp.get();

            if timestamp <= curr_tick {
                alarm.timestamp.set(u64::MAX);
        
                if let Some((f, ctx)) = alarm.callback.get() {
                    f(ctx);
                }
            }
        });
    }
}

impl Driver for TimerDriver {
    fn now(&self) -> u64 {
        SYS_TICK.load(Ordering::Relaxed) as u64
    }

    unsafe fn allocate_alarm(&self) -> Option<AlarmHandle> {
        let allocated = self.allocated.load(Ordering::Relaxed);
        if allocated {
            return None;
        }

        self.allocated.store(true, Ordering::Relaxed);
        Some(AlarmHandle::new(0))
    }

    fn set_alarm_callback(&self, _alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ()) {
        cortex_m::interrupt::free(|cs| {
            let alarm = &self.alarms.borrow(cs);
            alarm.callback.set(Some((callback, ctx)));
        })
    }

    fn set_alarm(&self, _alarm: AlarmHandle, timestamp: u64) -> bool {
        cortex_m::interrupt::free(|cs| {
            let alarm = &self.alarms.borrow(cs);
            alarm.timestamp.set(timestamp);

            let now = self.now();
            if timestamp <= now {
                alarm.timestamp.set(u64::MAX);
                false
            } else {
                alarm.timestamp.set(timestamp);
                true
            }
        })
    }
}


#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    if let Some(mut p) = hal::pac::Peripherals::take() {
        let mut rcc = p.RCC.configure().hsi().freeze(&mut p.FLASH);
        let gpiod = p.GPIOD.split(&mut rcc);

        let cp = Peripherals::take().unwrap();

        cortex_m::interrupt::free(|cs| gpiod.pd2.into_alternate_af1(cs));

        let _serial = Serial::uart1(p.UART1, 115_200.bps(), rcc.clocks);
        let mut syst = cp.SYST;
        systick_init(&mut syst, &mut rcc);

        spawner.spawn(test()).ok();

        println!("Hello world!");
        loop {
            println!("main...");
            Timer::after(Duration::from_millis(500)).await;
        }
    }

    loop {
        continue;
    }
}

#[embassy_executor::task]
async fn test() {
    let mut i=0;
    loop {
        println!("test... {}", i);
        i+=1;
        Timer::after(Duration::from_millis(1000)).await;
    }
}

#[exception]
fn SysTick() {
    let sys_tick = SYS_TICK.load(Ordering::Relaxed);
    SYS_TICK.store(sys_tick.wrapping_add(1), Ordering::Relaxed);
    DRIVER.check_alarm(sys_tick as u64);
}
