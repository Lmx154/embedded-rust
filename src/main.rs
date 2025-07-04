#![no_std]
#![no_main]

use cortex_m::asm::nop;
use cortex_m_rt::entry;
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::prelude::*;
use nb;

mod hardware;
mod sensors;

use hardware::HardwareConfig;
use sensors::GpsManager;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello from STM32F446RE - GPS UBX Reader!");

    // Initialize hardware (pins, UART, etc.)
    let mut hardware = HardwareConfig::new();
    rprintln!("Hardware initialized");

    // Initialize GPS manager
    let mut gps = GpsManager::new();
    
    // Get GPS configuration commands and send them
    let (port_config, pvt_config) = gps.get_config_commands();
    
    for &byte in &port_config {
        nb::block!(hardware.uart_tx.write(byte)).ok();
    }
    
    for &byte in &pvt_config {
        nb::block!(hardware.uart_tx.write(byte)).ok();
    }
    
    rprintln!("UBX configuration sent, waiting for GPS data...");

    let mut led_toggle_counter = 0u32;
    let mut last_print_time = 0u32;

    loop {
        // Toggle LED every 500,000 iterations to show we're alive
        if led_toggle_counter % 500_000 == 0 {
            hardware.led.toggle();
        }
        led_toggle_counter += 1;

        // Check if we have received data from GPS
        match hardware.uart_rx.read() {
            Ok(byte) => {
                // Process GPS byte
                if let Some(gps_data) = gps.process_byte(byte) {
                    // Print GPS data immediately when received
                    gps_data.print_position();
                    last_print_time = led_toggle_counter;
                }
            }
            Err(nb::Error::WouldBlock) => {
                // No data available, continue
                nop();
                
                // Print status every 5 seconds (approximately)
                if led_toggle_counter.wrapping_sub(last_print_time) > 2_500_000 {
                    if gps.has_fix() {
                        rprintln!("GPS Status: Active fix with {} satellites", gps.satellite_count());
                    } else {
                        rprintln!("GPS Status: Searching for satellites...");
                    }
                    last_print_time = led_toggle_counter;
                }
            }
            Err(nb::Error::Other(e)) => {
                // Only print errors occasionally to avoid spam
                if led_toggle_counter % 1_000_000 == 0 {
                    rprintln!("UART error: {:?}", e);
                }
            }
        }
    }
}
