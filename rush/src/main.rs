//! embassy hello world
//!
//! This is an example of running the embassy executor with multiple tasks
//! concurrently.

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(error_in_core)]

extern crate alloc;

use esp32s3_hal::{
    clock::ClockControl, peripherals::Peripherals, prelude::*, soc, timer::TimerGroup, Rtc, IO,
};
use esp_backtrace as _;
use esp_println::println;

mod command_evaluator;
mod command_parser;
mod rush_gpio_manager;

use rush_gpio_manager::GpioManager;

//static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_psram_heap() {
    unsafe {
        ALLOCATOR.init(
            soc::psram::PSRAM_VADDR_START as *mut u8,
            soc::psram::PSRAM_BYTES,
        );
    }
}

#[entry]
fn main() -> ! {
    println!("Init!");
    let peripherals = Peripherals::take();
    soc::psram::init_psram(peripherals.PSRAM);
    init_psram_heap();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt1 = timer_group1.wdt;

    command_evaluator::evaluate_command("read gpio.1");

    // Disable watchdog timers
    rtc.swd.disable();
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let gmgr = GpioManager::new(io.pins);

    let a = &gmgr.rush_input_pins[0];

    loop {
        let b = a.is_low().unwrap();
        println!("b: {}", b);
    }
}
