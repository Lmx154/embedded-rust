#![allow(dead_code)]

use rtt_target::rprintln;

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
pub struct GpsData {
    pub valid: bool,
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub nano: i32,           // Nanoseconds
    pub latitude: i32,       // Latitude in 1e-7 degrees
    pub longitude: i32,      // Longitude in 1e-7 degrees
    pub height_msl: i32,     // Height above mean sea level in mm
    pub horizontal_accuracy: u32, // Horizontal accuracy in mm
    pub vertical_accuracy: u32,   // Vertical accuracy in mm
    pub ground_speed: i32,   // Ground speed in mm/s
    pub satellites: u8,      // Number of satellites
}

impl GpsData {
    pub fn new() -> Self {
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

    pub fn print_position(&self) {
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

    /// Get latitude in degrees as f64
    pub fn latitude_degrees(&self) -> f64 {
        self.latitude as f64 / 1e7
    }

    /// Get longitude in degrees as f64
    pub fn longitude_degrees(&self) -> f64 {
        self.longitude as f64 / 1e7
    }

    /// Get altitude in meters as f64
    pub fn altitude_meters(&self) -> f64 {
        self.height_msl as f64 / 1000.0
    }

    /// Get ground speed in meters per second as f64
    pub fn speed_ms(&self) -> f64 {
        self.ground_speed as f64 / 1000.0
    }

    /// Get horizontal accuracy in meters as f64
    pub fn horizontal_accuracy_meters(&self) -> f64 {
        self.horizontal_accuracy as f64 / 1000.0
    }

    /// Get vertical accuracy in meters as f64
    pub fn vertical_accuracy_meters(&self) -> f64 {
        self.vertical_accuracy as f64 / 1000.0
    }
}

// UBX Parser
pub struct UbxParser {
    state: UbxParserState,
    message: UbxMessage,
    payload_index: usize,
    calculated_checksum_a: u8,
    calculated_checksum_b: u8,
}

impl UbxParser {
    pub fn new() -> Self {
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

    pub fn parse_byte(&mut self, byte: u8) -> Option<GpsData> {
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
        let _valid = payload[11]; // Validity flags
        
        let nano = i32::from_le_bytes([payload[16], payload[17], payload[18], payload[19]]);
        let fix_type = payload[20];
        let flags = payload[21];
        let num_sv = payload[23]; // Number of satellites
        
        let longitude = i32::from_le_bytes([payload[24], payload[25], payload[26], payload[27]]);
        let latitude = i32::from_le_bytes([payload[28], payload[29], payload[30], payload[31]]);
        let _height = i32::from_le_bytes([payload[32], payload[33], payload[34], payload[35]]);
        let h_msl = i32::from_le_bytes([payload[36], payload[37], payload[38], payload[39]]);
        let h_acc = u32::from_le_bytes([payload[40], payload[41], payload[42], payload[43]]);
        let v_acc = u32::from_le_bytes([payload[44], payload[45], payload[46], payload[47]]);
        
        let _vel_n = i32::from_le_bytes([payload[48], payload[49], payload[50], payload[51]]);
        let _vel_e = i32::from_le_bytes([payload[52], payload[53], payload[54], payload[55]]);
        let _vel_d = i32::from_le_bytes([payload[56], payload[57], payload[58], payload[59]]);
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

// UBX Configuration Commands
pub struct UbxConfig;

impl UbxConfig {
    /// Get UBX command to configure port for UBX-only output (disables NMEA)
    pub fn get_port_config_ubx_only() -> [u8; 28] {
        [
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
        ]
    }

    /// Get UBX command to enable NAV-PVT messages
    pub fn get_enable_nav_pvt() -> [u8; 11] {
        [
            0xB5, 0x62,  // UBX sync chars
            0x06, 0x01,  // Class CFG, ID MSG
            0x03, 0x00,  // Length (3 bytes)
            0x01, 0x07,  // Message Class/ID (NAV-PVT)
            0x01,        // Rate (1 = output every solution)
            0x13, 0x51   // Checksum
        ]
    }
}
