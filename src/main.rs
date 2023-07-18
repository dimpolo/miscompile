#![no_std]
#![no_main]

use ahrs::{Ahrs, Mahony};
use esp32s3_hal::clock::{ClockControl, CpuClock};
use esp32s3_hal::peripherals::Peripherals;
use esp32s3_hal::prelude::*;
use esp32s3_hal::{entry, timer, Rtc};
use esp_backtrace as _;
use esp_println::println;
use nalgebra::Vector3;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock80MHz).freeze();

    let timer_group0 = timer::TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut timer0 = timer_group0.timer0;
    let mut wdt = timer_group0.wdt;
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);

    // Disable MWDT and RWDT (Watchdog) flash boot protection
    wdt.disable();
    rtc.rwdt.disable();

    let mut ahrs = Mahony::<f32>::new(0.1, 10.0, 0.06);
    let accelerometer = Vector3::new(0.1, 0.0, 10.0);
    let gyroscope = Vector3::new(0.0, 0.0, 0.0);

    timer0.start(100u64.millis());
    loop {
        nb::block!(timer0.wait()).unwrap();

        let quat = ahrs.update_imu(&gyroscope, &accelerometer).unwrap();

        let (_, pitch, _) = quat.euler_angles();

        println!("{}", pitch.to_degrees())
    }
}
