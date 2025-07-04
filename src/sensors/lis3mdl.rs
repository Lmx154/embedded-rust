use embedded_hal::i2c::I2c;
use rtt_target::rprintln;

// LIS3MDL I2C address (when SA1 pin is connected to GND)
pub const LIS3MDL_ADDRESS: u8 = 0x1C;

// LIS3MDL Register addresses
pub const WHO_AM_I: u8 = 0x0F;
pub const CTRL_REG1: u8 = 0x20;
pub const CTRL_REG2: u8 = 0x21;
pub const CTRL_REG3: u8 = 0x22;
pub const CTRL_REG4: u8 = 0x23;
pub const CTRL_REG5: u8 = 0x24;
pub const STATUS_REG: u8 = 0x27;
pub const OUT_X_L: u8 = 0x28;
pub const OUT_X_H: u8 = 0x29;
pub const OUT_Y_L: u8 = 0x2A;
pub const OUT_Y_H: u8 = 0x2B;
pub const OUT_Z_L: u8 = 0x2C;
pub const OUT_Z_H: u8 = 0x2D;
pub const TEMP_OUT_L: u8 = 0x2E;
pub const TEMP_OUT_H: u8 = 0x2F;

// Expected WHO_AM_I value for LIS3MDL
pub const LIS3MDL_WHO_AM_I_VALUE: u8 = 0x3D;

// Performance modes
#[derive(Debug, Clone, Copy)]
pub enum PerformanceMode {
    LowPower,
    Medium,
    High,
    UltraHigh,
}

// Data rates
#[derive(Debug, Clone, Copy)]
pub enum DataRate {
    Hz0_625,  // 0.625 Hz
    Hz1_25,   // 1.25 Hz
    Hz2_5,    // 2.5 Hz
    Hz5,      // 5 Hz
    Hz10,     // 10 Hz
    Hz20,     // 20 Hz
    Hz40,     // 40 Hz
    Hz80,     // 80 Hz
}

// Full scale selection
#[derive(Debug, Clone, Copy)]
pub enum FullScale {
    Gauss4,   // ±4 gauss
    Gauss8,   // ±8 gauss
    Gauss12,  // ±12 gauss
    Gauss16,  // ±16 gauss
}

#[derive(Debug)]
pub struct MagnetometerData {
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub temperature: i16,
}

pub struct Lis3mdl<I2C> {
    i2c: I2C,
    address: u8,
    full_scale: FullScale,
}

impl<I2C, E> Lis3mdl<I2C>
where
    I2C: I2c<Error = E>,
{
    pub fn new(i2c: I2C) -> Self {
        Self {
            i2c,
            address: LIS3MDL_ADDRESS,
            full_scale: FullScale::Gauss4,
        }
    }

    pub fn init(&mut self) -> Result<(), E> {
        // Check WHO_AM_I register
        let who_am_i = self.read_register(WHO_AM_I)?;
        if who_am_i != LIS3MDL_WHO_AM_I_VALUE {
            rprintln!("ERROR: LIS3MDL WHO_AM_I mismatch! Expected 0x{:02X}, got 0x{:02X}", 
                     LIS3MDL_WHO_AM_I_VALUE, who_am_i);
            // For now, we'll continue anyway - some clones might have different WHO_AM_I
        } else {
            rprintln!("LIS3MDL WHO_AM_I check passed: 0x{:02X}", who_am_i);
        }

        // Configure CTRL_REG1: Temperature enabled, High performance XY, 10 Hz, no self-test
        // Bit 7: TEMP_EN = 1 (temperature sensor enabled)
        // Bit 6-5: OM[1:0] = 11 (High performance mode for X and Y axes)
        // Bit 4-2: DO[2:0] = 100 (10 Hz data rate)
        // Bit 1: FAST_ODR = 0
        // Bit 0: ST = 0 (self-test disabled)
        self.write_register(CTRL_REG1, 0b11101000)?;

        // Configure CTRL_REG2: Full scale ±4 gauss, no reset
        // Bit 7: Reserved = 0
        // Bit 6-5: FS[1:0] = 00 (±4 gauss)
        // Bit 4: Reserved = 0
        // Bit 3: REBOOT = 0 (normal mode)
        // Bit 2: SOFT_RST = 0 (normal mode)
        // Bit 1-0: Reserved = 00
        self.write_register(CTRL_REG2, 0b00000000)?;

        // Configure CTRL_REG3: Continuous conversion mode
        // Bit 7-2: Reserved = 000000
        // Bit 1-0: MD[1:0] = 00 (continuous conversion mode)
        self.write_register(CTRL_REG3, 0b00000000)?;

        // Configure CTRL_REG4: High performance Z axis, Little endian
        // Bit 7-4: Reserved = 0000
        // Bit 3-2: OMZ[1:0] = 11 (High performance mode for Z axis)
        // Bit 1: BLE = 0 (little endian)
        // Bit 0: Reserved = 0
        self.write_register(CTRL_REG4, 0b00001100)?;

        // Configure CTRL_REG5: Fast read disabled, continuous update
        // Bit 7: FAST_READ = 0 (fast read disabled)
        // Bit 6: BDU = 0 (continuous update)
        // Bit 5-0: Reserved = 000000
        self.write_register(CTRL_REG5, 0b00000000)?;

        rprintln!("LIS3MDL initialized successfully");
        Ok(())
    }

    pub fn read_magnetometer(&mut self) -> Result<MagnetometerData, E> {
        // Check if data is ready
        let status = self.read_register(STATUS_REG)?;
        if (status & 0x08) == 0 {
            rprintln!("Warning: Data not ready yet (STATUS: 0x{:02X})", status);
        }

        // Read all magnetometer data (6 bytes) in one go
        let mut data = [0u8; 6];
        self.read_registers(OUT_X_L, &mut data)?;

        // Convert to signed 16-bit values (little endian)
        let x = i16::from_le_bytes([data[0], data[1]]);
        let y = i16::from_le_bytes([data[2], data[3]]);
        let z = i16::from_le_bytes([data[4], data[5]]);

        // Read temperature (2 bytes)
        let mut temp_data = [0u8; 2];
        self.read_registers(TEMP_OUT_L, &mut temp_data)?;
        let temperature = i16::from_le_bytes([temp_data[0], temp_data[1]]);

        Ok(MagnetometerData {
            x,
            y,
            z,
            temperature,
        })
    }

    pub fn read_magnetometer_gauss(&mut self) -> Result<(f32, f32, f32), E> {
        let data = self.read_magnetometer()?;
        
        // Convert raw values to gauss based on full scale setting
        let scale_factor = match self.full_scale {
            FullScale::Gauss4 => 4.0 / 32768.0,   // ±4 gauss, 16-bit
            FullScale::Gauss8 => 8.0 / 32768.0,   // ±8 gauss, 16-bit
            FullScale::Gauss12 => 12.0 / 32768.0, // ±12 gauss, 16-bit
            FullScale::Gauss16 => 16.0 / 32768.0, // ±16 gauss, 16-bit
        };

        let x_gauss = data.x as f32 * scale_factor;
        let y_gauss = data.y as f32 * scale_factor;
        let z_gauss = data.z as f32 * scale_factor;

        Ok((x_gauss, y_gauss, z_gauss))
    }

    pub fn read_temperature_celsius(&mut self) -> Result<f32, E> {
        let data = self.read_magnetometer()?;
        
        // Temperature calculation: 25°C + (TEMP_OUT / 256)
        // The offset might need calibration for your specific sensor
        let temp_celsius = 25.0 + (data.temperature as f32 / 256.0);
        
        Ok(temp_celsius)
    }

    fn write_register(&mut self, register: u8, value: u8) -> Result<(), E> {
        self.i2c.write(self.address, &[register, value])
    }

    fn read_register(&mut self, register: u8) -> Result<u8, E> {
        let mut buffer = [0u8; 1];
        self.i2c.write_read(self.address, &[register], &mut buffer)?;
        Ok(buffer[0])
    }

    fn read_registers(&mut self, start_register: u8, buffer: &mut [u8]) -> Result<(), E> {
        self.i2c.write_read(self.address, &[start_register], buffer)
    }
}
