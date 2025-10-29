#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

// modules
mod robot;

use core::panic::PanicInfo;
use robot::Robot;

// Baudrate of the robot serial connection
const BAUDRATE: u32 = 115200;

// Frequency of the robot processing
const PROCESS_INTERVAL_US: u32 = 0;

#[arduino_hal::entry]
// Load peripherals of arduino
fn main() -> ! {
    let peripherals = match arduino_hal::Peripherals::take() {
        Some(p) => p,
        None => panic!("Fail to load peripherals"),
    };

    // Initialize Robot with the peripherals, baurate of serial, and process interval
    let mut robot = Robot::new(peripherals, BAUDRATE, PROCESS_INTERVAL_US);
    robot.start();
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
