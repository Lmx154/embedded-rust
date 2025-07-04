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

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello from STM32F446RE - GPS NMEA Reader!");

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
    let (mut _tx, mut rx) = uart.split();

    rprintln!("UART initialized for GPS communication");
    rprintln!("Configuring NEO-M9N to output NMEA...");
    
    // Send UBX command to enable NMEA GGA messages on UART1
    // UBX-CFG-MSG: Configure message rate for GGA
    let ubx_cfg_gga: [u8; 11] = [
        0xB5, 0x62,  // UBX sync chars
        0x06, 0x01,  // Class CFG, ID MSG
        0x03, 0x00,  // Length (3 bytes)
        0xF0, 0x00,  // Message Class/ID (NMEA GGA)
        0x01,        // Rate (1 = output every solution)
        0xFB, 0x11   // Checksum
    ];
    
    for &byte in &ubx_cfg_gga {
        nb::block!(_tx.write(byte)).ok();
    }
    
    rprintln!("Configuration sent, waiting for NMEA data...");

    let mut led_toggle_counter = 0u32;
    let mut line_buffer = [0u8; 128];
    let mut buffer_index = 0usize;

    loop {
        // Toggle LED every 500,000 iterations to show we're alive
        if led_toggle_counter % 500_000 == 0 {
            led.toggle();
        }
        led_toggle_counter += 1;

        // Check if we have received data from GPS
        match rx.read() {
            Ok(byte) => {
                // Debug: print received bytes to see what we're getting
                if led_toggle_counter % 100_000 == 0 {
                    rprintln!("Received byte: 0x{:02X} ({})", byte, byte as char);
                }
                
                // Add byte to buffer
                if buffer_index < line_buffer.len() - 1 {
                    line_buffer[buffer_index] = byte;
                    buffer_index += 1;
                    
                    // Check if we received a complete line (ending with \n)
                    if byte == b'\n' {
                        // Convert buffer to string and print
                        line_buffer[buffer_index] = 0; // Null terminate
                        
                        // Find the actual length (excluding null terminator and potential \r)
                        let mut line_len = buffer_index;
                        if line_len > 0 && line_buffer[line_len - 1] == b'\n' {
                            line_len -= 1;
                        }
                        if line_len > 0 && line_buffer[line_len - 1] == b'\r' {
                            line_len -= 1;
                        }
                        
                        // Try to convert to string slice and print
                        if let Ok(line_str) = core::str::from_utf8(&line_buffer[0..line_len]) {
                            rprintln!("NMEA: {}", line_str);
                        } else {
                            rprintln!("Non-UTF8 data received");
                        }
                        
                        // Reset buffer
                        buffer_index = 0;
                    }
                } else {
                    // Buffer overflow, reset
                    rprintln!("Buffer overflow, resetting");
                    buffer_index = 0;
                }
            }
            Err(nb::Error::WouldBlock) => {
                // No data available, continue
                nop();
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
