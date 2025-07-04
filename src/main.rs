#![no_std]
#![no_main]

use cortex_m::asm::nop;
use cortex_m_rt::entry;
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::{
    pac,
    prelude::*,
    serial::{Config, Serial},
};
use nb;

mod sensors;
use sensors::{GpsData, UbxParser, UbxConfig};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello from STM32F446RE - GPS UBX Reader!");

    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let rcc = dp.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    let clocks = rcc.cfgr.freeze();

    // Acquire the GPIOA peripheral
    let gpioa = dp.GPIOA.split();

    // Configure PA5 (built-in LED on Nucleo-F446RE) as a push-pull output
    let mut led = gpioa.pa5.into_push_pull_output();

    // Configure UART pins
    // PA9 = TX (output to GPS RX) - AF7
    // PA10 = RX (input from GPS TX) - AF7
    let tx_pin = gpioa.pa9.into_alternate::<7>();
    let rx_pin = gpioa.pa10.into_alternate::<7>();

    // Configure UART1 (USART1) with 38400 baud rate (default for NEO-M9N-00B)
    let config = Config::default().baudrate(38400.bps());
    let uart = Serial::new(dp.USART1, (tx_pin, rx_pin), config, &clocks).unwrap();
    let (mut tx, mut rx) = uart.split();

    rprintln!("UART initialized for GPS communication");
    rprintln!("Configuring NEO-M9N for UBX output...");
    
    // Send UBX commands to configure GPS for UBX-only output
    let ubx_cfg_port = UbxConfig::get_port_config_ubx_only();
    for &byte in &ubx_cfg_port {
        nb::block!(tx.write(byte)).ok();
    }
    
    let ubx_cfg_pvt = UbxConfig::get_enable_nav_pvt();
    for &byte in &ubx_cfg_pvt {
        nb::block!(tx.write(byte)).ok();
    }
    
    rprintln!("UBX configuration sent, waiting for GPS data...");

    let mut led_toggle_counter = 0u32;
    let mut ubx_parser = UbxParser::new();
    let mut last_gps_data = GpsData::new();
    let mut last_print_time = 0u32;

    loop {
        // Toggle LED every 500,000 iterations to show we're alive
        if led_toggle_counter % 500_000 == 0 {
            led.toggle();
        }
        led_toggle_counter += 1;

        // Check if we have received data from GPS
        match rx.read() {
            Ok(byte) => {
                // Parse UBX byte
                if let Some(gps_data) = ubx_parser.parse_byte(byte) {
                    last_gps_data = gps_data;
                    
                    // Print GPS data immediately when received
                    last_gps_data.print_position();
                    last_print_time = led_toggle_counter;
                }
            }
            Err(nb::Error::WouldBlock) => {
                // No data available, continue
                nop();
                
                // Print status every 5 seconds (approximately)
                if led_toggle_counter.wrapping_sub(last_print_time) > 2_500_000 {
                    if last_gps_data.valid {
                        rprintln!("GPS Status: Active fix with {} satellites", last_gps_data.satellites);
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
