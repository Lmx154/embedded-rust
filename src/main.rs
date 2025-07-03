#![no_std]
#![no_main]

use cortex_m::asm::nop;
use cortex_m_rt::entry;
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::{
    pac,
    prelude::*,
};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello from STM32F446RE!");

    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let rcc = dp.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    let _clocks = rcc.cfgr.freeze();

    // Acquire the GPIOA peripheral
    let gpioa = dp.GPIOA.split();

    // Configure PA5 (built-in LED on Nucleo-F446RE) as a push-pull output
    let mut led = gpioa.pa5.into_push_pull_output();

    let mut counter = 0u32;

    loop {
        led.set_high();
        rprintln!("LED ON - Count: {}", counter);
        
        // Simple delay
        for _ in 0..1_000_000 {
            nop();
        }

        led.set_low();
        rprintln!("LED OFF - Count: {}", counter);
        
        // Simple delay
        for _ in 0..1_000_000 {
            nop();
        }

        counter += 1;
    }
}
