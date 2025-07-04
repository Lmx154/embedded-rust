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

// UBX Protocol Constants
const UBX_SYNC_CHAR_1: u8 = 0xB5;
const UBX_SYNC_CHAR_2: u8 = 0x62;

// UBX Message Classes
const UBX_CLASS_NAV: u8 = 0x01;

// UBX NAV Message IDs
const UBX_NAV_PVT: u8 = 0x07;  // Navigation Position Velocity Time Solution

// UBX Parser States
#[derive(Clone, Copy, PartialEq)]
enum UbxParserState {
    WaitingForSync1,
    WaitingForSync2,
    ReadingClass,
    ReadingId,
    ReadingLength1,
    ReadingLength2,
    ReadingPayload,
    ReadingChecksum1,
    ReadingChecksum2,
}

// UBX Message Structure
struct UbxMessage {
    class: u8,
    id: u8,
    length: u16,
    payload: [u8; 256], // Max UBX payload size
    checksum_a: u8,
    checksum_b: u8,
}

impl UbxMessage {
    fn new() -> Self {
        Self {
            class: 0,
            id: 0,
            length: 0,
            payload: [0; 256],
            checksum_a: 0,
            checksum_b: 0,
        }
    }
}

// GPS Position/Velocity/Time data from UBX-NAV-PVT
#[derive(Clone, Copy)]
struct GpsData {
    valid: bool,
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    nano: i32,           // Nanoseconds
    latitude: i32,       // Latitude in 1e-7 degrees
    longitude: i32,      // Longitude in 1e-7 degrees
    height_msl: i32,     // Height above mean sea level in mm
    horizontal_accuracy: u32, // Horizontal accuracy in mm
    vertical_accuracy: u32,   // Vertical accuracy in mm
    ground_speed: i32,   // Ground speed in mm/s
    satellites: u8,      // Number of satellites
}

impl GpsData {
    fn new() -> Self {
        Self {
            valid: false,
            year: 0,
            month: 0,
            day: 0,
            hour: 0,
            minute: 0,
            second: 0,
            nano: 0,
            latitude: 0,
            longitude: 0,
            height_msl: 0,
            horizontal_accuracy: 0,
            vertical_accuracy: 0,
            ground_speed: 0,
            satellites: 0,
        }
    }

    fn print_position(&self) {
        if self.valid {
            // Convert from 1e-7 degrees to degrees with 7 decimal places
            let lat_deg = self.latitude as f64 / 1e7;
            let lon_deg = self.longitude as f64 / 1e7;
            let height_m = self.height_msl as f64 / 1000.0;
            let speed_ms = self.ground_speed as f64 / 1000.0;
            let h_acc_m = self.horizontal_accuracy as f64 / 1000.0;
            
            rprintln!("GPS Fix: {}/{:02}/{:02} {:02}:{:02}:{:02}", 
                     self.year, self.month, self.day, self.hour, self.minute, self.second);
            rprintln!("Position: {:.7}°, {:.7}° (±{:.1}m)", lat_deg, lon_deg, h_acc_m);
            rprintln!("Altitude: {:.1}m, Speed: {:.1}m/s, Sats: {}", 
                     height_m, speed_ms, self.satellites);
        } else {
            rprintln!("GPS: No valid fix");
        }
    }
}

// UBX Parser
struct UbxParser {
    state: UbxParserState,
    message: UbxMessage,
    payload_index: usize,
    calculated_checksum_a: u8,
    calculated_checksum_b: u8,
}

impl UbxParser {
    fn new() -> Self {
        Self {
            state: UbxParserState::WaitingForSync1,
            message: UbxMessage::new(),
            payload_index: 0,
            calculated_checksum_a: 0,
            calculated_checksum_b: 0,
        }
    }

    fn reset(&mut self) {
        self.state = UbxParserState::WaitingForSync1;
        self.payload_index = 0;
        self.calculated_checksum_a = 0;
        self.calculated_checksum_b = 0;
    }

    fn calculate_checksum(&mut self, byte: u8) {
        self.calculated_checksum_a = self.calculated_checksum_a.wrapping_add(byte);
        self.calculated_checksum_b = self.calculated_checksum_b.wrapping_add(self.calculated_checksum_a);
    }

    fn parse_byte(&mut self, byte: u8) -> Option<GpsData> {
        match self.state {
            UbxParserState::WaitingForSync1 => {
                if byte == UBX_SYNC_CHAR_1 {
                    self.state = UbxParserState::WaitingForSync2;
                }
            }
            UbxParserState::WaitingForSync2 => {
                if byte == UBX_SYNC_CHAR_2 {
                    self.state = UbxParserState::ReadingClass;
                    self.calculated_checksum_a = 0;
                    self.calculated_checksum_b = 0;
                } else {
                    self.reset();
                }
            }
            UbxParserState::ReadingClass => {
                self.message.class = byte;
                self.calculate_checksum(byte);
                self.state = UbxParserState::ReadingId;
            }
            UbxParserState::ReadingId => {
                self.message.id = byte;
                self.calculate_checksum(byte);
                self.state = UbxParserState::ReadingLength1;
            }
            UbxParserState::ReadingLength1 => {
                self.message.length = byte as u16;
                self.calculate_checksum(byte);
                self.state = UbxParserState::ReadingLength2;
            }
            UbxParserState::ReadingLength2 => {
                self.message.length |= (byte as u16) << 8;
                self.calculate_checksum(byte);
                self.payload_index = 0;
                if self.message.length == 0 {
                    self.state = UbxParserState::ReadingChecksum1;
                } else if self.message.length <= 256 {
                    self.state = UbxParserState::ReadingPayload;
                } else {
                    // Message too large, reset
                    self.reset();
                }
            }
            UbxParserState::ReadingPayload => {
                if self.payload_index < self.message.length as usize {
                    self.message.payload[self.payload_index] = byte;
                    self.payload_index += 1;
                    self.calculate_checksum(byte);
                    
                    if self.payload_index >= self.message.length as usize {
                        self.state = UbxParserState::ReadingChecksum1;
                    }
                } else {
                    self.reset();
                }
            }
            UbxParserState::ReadingChecksum1 => {
                self.message.checksum_a = byte;
                self.state = UbxParserState::ReadingChecksum2;
            }
            UbxParserState::ReadingChecksum2 => {
                self.message.checksum_b = byte;
                
                // Verify checksum
                if self.calculated_checksum_a == self.message.checksum_a &&
                   self.calculated_checksum_b == self.message.checksum_b {
                    
                    // Process the message
                    let result = self.process_message();
                    self.reset();
                    return result;
                } else {
                    rprintln!("UBX checksum error");
                }
                self.reset();
            }
        }
        None
    }

    fn process_message(&self) -> Option<GpsData> {
        if self.message.class == UBX_CLASS_NAV && self.message.id == UBX_NAV_PVT {
            return self.parse_nav_pvt();
        }
        None
    }

    fn parse_nav_pvt(&self) -> Option<GpsData> {
        if self.message.length < 84 {
            return None;
        }

        let payload = &self.message.payload;
        
        // Extract fields from UBX-NAV-PVT payload
        let year = u16::from_le_bytes([payload[4], payload[5]]);
        let month = payload[6];
        let day = payload[7];
        let hour = payload[8];
        let minute = payload[9];
        let second = payload[10];
        let valid = payload[11]; // Validity flags
        
        let nano = i32::from_le_bytes([payload[16], payload[17], payload[18], payload[19]]);
        let fix_type = payload[20];
        let flags = payload[21];
        let num_sv = payload[23]; // Number of satellites
        
        let longitude = i32::from_le_bytes([payload[24], payload[25], payload[26], payload[27]]);
        let latitude = i32::from_le_bytes([payload[28], payload[29], payload[30], payload[31]]);
        let height = i32::from_le_bytes([payload[32], payload[33], payload[34], payload[35]]);
        let h_msl = i32::from_le_bytes([payload[36], payload[37], payload[38], payload[39]]);
        let h_acc = u32::from_le_bytes([payload[40], payload[41], payload[42], payload[43]]);
        let v_acc = u32::from_le_bytes([payload[44], payload[45], payload[46], payload[47]]);
        
        let vel_n = i32::from_le_bytes([payload[48], payload[49], payload[50], payload[51]]);
        let vel_e = i32::from_le_bytes([payload[52], payload[53], payload[54], payload[55]]);
        let vel_d = i32::from_le_bytes([payload[56], payload[57], payload[58], payload[59]]);
        let g_speed = i32::from_le_bytes([payload[60], payload[61], payload[62], payload[63]]);
        
        // Check if we have a valid 3D fix
        let has_valid_fix = fix_type >= 3 && (flags & 0x01) != 0;
        
        Some(GpsData {
            valid: has_valid_fix,
            year,
            month,
            day,
            hour,
            minute,
            second,
            nano,
            latitude,
            longitude,
            height_msl: h_msl,
            horizontal_accuracy: h_acc,
            vertical_accuracy: v_acc,
            ground_speed: g_speed,
            satellites: num_sv,
        })
    }
}

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
    
    // Send UBX command to disable NMEA and enable UBX-NAV-PVT messages
    // First, disable all NMEA messages on UART1 port
    let ubx_cfg_nmea_off: [u8; 28] = [
        0xB5, 0x62,  // UBX sync chars
        0x06, 0x00,  // Class CFG, ID PRT (Port configuration)
        0x14, 0x00,  // Length (20 bytes)
        0x01,        // Port ID (1 = UART1)
        0x00,        // Reserved
        0x00, 0x00,  // TX Ready pin config
        0x00, 0x23, 0x00, 0x23,  // UART mode (8N1)
        0x00, 0x96, 0x00, 0x00,  // Baud rate (38400)
        0x01, 0x00,  // Input protocols (UBX only)
        0x01, 0x00,  // Output protocols (UBX only)
        0x00, 0x00,  // Flags
        0x00, 0x00,  // Reserved
        0x41, 0x28   // Checksum
    ];
    
    for &byte in &ubx_cfg_nmea_off {
        nb::block!(tx.write(byte)).ok();
    }
    
    // Enable UBX-NAV-PVT message (Navigation Position Velocity Time Solution)
    let ubx_cfg_pvt: [u8; 11] = [
        0xB5, 0x62,  // UBX sync chars
        0x06, 0x01,  // Class CFG, ID MSG
        0x03, 0x00,  // Length (3 bytes)
        0x01, 0x07,  // Message Class/ID (NAV-PVT)
        0x01,        // Rate (1 = output every solution)
        0x13, 0x51   // Checksum
    ];
    
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
