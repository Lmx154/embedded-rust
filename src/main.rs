#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};

mod hardware;

use hardware::Hardware;

// LIS3MDL I2C address
const LIS3MDL_ADDR: u8 = 0x1C;

// Register addresses
const CTRL_REG1: u8 = 0x20;
const CTRL_REG2: u8 = 0x21;
const CTRL_REG3: u8 = 0x22;
const CTRL_REG4: u8 = 0x23;
const OUT_X_L: u8 = 0x28;

// Simple calibration structure
#[derive(Clone, Copy)]
struct MagCalibration {
    offset_x: f32,
    offset_y: f32,
    offset_z: f32,
    is_calibrated: bool,
}

impl MagCalibration {
    fn new() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 0.0,
            offset_z: 0.0,
            is_calibrated: false,
        }
    }
    
    fn calculate_heading(&self, x: i16, y: i16) -> f32 {
        // Apply calibration if available
        let x_cal = if self.is_calibrated {
            x as f32 - self.offset_x
        } else {
            x as f32
        };
        
        let y_cal = if self.is_calibrated {
            y as f32 - self.offset_y
        } else {
            y as f32
        };
        
        // Calculate heading using atan2
        let heading_rad = libm::atan2f(y_cal, x_cal);
        let mut heading_deg = heading_rad * 180.0 / core::f32::consts::PI;
        
        // Normalize to 0-360 degrees
        if heading_deg < 0.0 {
            heading_deg += 360.0;
        }
        
        heading_deg
    }
}

fn calibrate_magnetometer(hardware: &mut Hardware) -> MagCalibration {
    rprintln!("Starting magnetometer calibration...");
    rprintln!("Slowly rotate the device in all directions for 15 seconds");
    
    let mut min_x = i16::MAX;
    let mut max_x = i16::MIN;
    let mut min_y = i16::MAX;
    let mut max_y = i16::MIN;
    let mut min_z = i16::MAX;
    let mut max_z = i16::MIN;
    
    // Calibration duration (approximately 15 seconds at ~2Hz)
    for i in 0..30 {
        let mut buffer = [0u8; 6];
        if let Ok(_) = hardware.i2c.write_read(LIS3MDL_ADDR, &[OUT_X_L | 0x80], &mut buffer) {
            let x = i16::from_le_bytes([buffer[0], buffer[1]]);
            let y = i16::from_le_bytes([buffer[2], buffer[3]]);
            let z = i16::from_le_bytes([buffer[4], buffer[5]]);
            
            // Track min/max values
            if x < min_x { min_x = x; }
            if x > max_x { max_x = x; }
            if y < min_y { min_y = y; }
            if y > max_y { max_y = y; }
            if z < min_z { min_z = z; }
            if z > max_z { max_z = z; }
            
            // Progress indicator
            if i % 5 == 0 {
                rprintln!("Calibration: {}%", i * 100 / 30);
            }
        }
        
        // Delay between readings (~0.5 seconds)
        cortex_m::asm::delay(8_000_000);
    }
    
    // Calculate offsets (center point of min/max range)
    let offset_x = ((max_x + min_x) as f32) / 2.0;
    let offset_y = ((max_y + min_y) as f32) / 2.0;
    let offset_z = ((max_z + min_z) as f32) / 2.0;
    
    rprintln!("Calibration complete!");
    rprintln!("X offset: {:.1} (range: {} to {})", offset_x, min_x, max_x);
    rprintln!("Y offset: {:.1} (range: {} to {})", offset_y, min_y, max_y);
    rprintln!("Z offset: {:.1} (range: {} to {})", offset_z, min_z, max_z);
    
    MagCalibration {
        offset_x,
        offset_y,
        offset_z,
        is_calibrated: true,
    }
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Starting LIS3MDL magnetometer with heading calculation...");

    // Initialize hardware
    let mut hardware = Hardware::new();
    rprintln!("Hardware initialized");

    // Simple LIS3MDL initialization
    let _ = hardware.i2c.write(LIS3MDL_ADDR, &[CTRL_REG1, 0x70]); // Enable X,Y axes, 80Hz
    let _ = hardware.i2c.write(LIS3MDL_ADDR, &[CTRL_REG2, 0x00]); // ±4 gauss
    let _ = hardware.i2c.write(LIS3MDL_ADDR, &[CTRL_REG3, 0x00]); // Continuous mode
    let _ = hardware.i2c.write(LIS3MDL_ADDR, &[CTRL_REG4, 0x0C]); // Enable Z axis
    
    rprintln!("LIS3MDL initialized");

    // Calibrate the magnetometer
    let calibration = calibrate_magnetometer(&mut hardware);
    
    rprintln!("Starting heading measurements...");

    loop {
        // Read raw magnetometer data
        let mut buffer = [0u8; 6];
        match hardware.i2c.write_read(LIS3MDL_ADDR, &[OUT_X_L | 0x80], &mut buffer) {
            Ok(_) => {
                // Combine bytes (little endian)
                let x = i16::from_le_bytes([buffer[0], buffer[1]]);
                let y = i16::from_le_bytes([buffer[2], buffer[3]]);
                let z = i16::from_le_bytes([buffer[4], buffer[5]]);
                
                // Calculate calibrated heading
                let heading = calibration.calculate_heading(x, y);
                
                rprintln!("Raw: X={}, Y={}, Z={} | Heading: {:.1}°", x, y, z, heading);
            }
            Err(_) => {
                rprintln!("Failed to read magnetometer data");
            }
        }

        // Toggle LED and delay
        hardware.config.led.toggle();
        cortex_m::asm::delay(16_000_000); // ~1 second
    }
}
