//! Minimal test for example.
//! Hardware: Raspberry Pi Pico / Pico 2
//! Connections:
//!   - dummy : PINs 4,5

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let _p4 = p.PIN_4;
    let _p5 = p.PIN_5;

    // Print system clock for verification
    info!("System clock: {} Hz", embassy_rp::clocks::clk_sys_freq());
    info!("Starting imu-sensors rp i2c test");

    info!("looping indefinitely");
    let mut count: u32 = 0;
    loop {
        Timer::after(Duration::from_millis(1)).await;
        count = count.wrapping_add(1);
        if count.is_multiple_of(1000) {
            info!("Loop {} ", count);
        }
    }
}
