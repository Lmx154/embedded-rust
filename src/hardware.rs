use stm32f4xx_hal::{
    pac,
    prelude::*,
    gpio::{Pin, Output, PushPull},
    i2c::{I2c, Mode},
};

pub type LedPin = Pin<'A', 5, Output<PushPull>>;
pub type I2cBus = I2c<pac::I2C1>;

pub struct HardwareConfig {
    pub led: LedPin,
}

pub struct Hardware {
    pub config: HardwareConfig,
    pub i2c: I2cBus,
}

impl Hardware {
    pub fn new() -> Self {
        // Get access to the device specific peripherals
        let dp = pac::Peripherals::take().unwrap();

        // Take ownership over the raw flash and rcc devices and convert them into the corresponding
        // HAL structs
        let rcc = dp.RCC.constrain();

        // Freeze the configuration of all the clocks in the system and store the frozen frequencies
        let clocks = rcc.cfgr.freeze();

        // Acquire the GPIO peripherals
        let gpioa = dp.GPIOA.split();
        let gpiob = dp.GPIOB.split();

        // Configure PA5 (built-in LED on Nucleo-F446RE) as a push-pull output
        let led = gpioa.pa5.into_push_pull_output();

        // Configure I2C1 pins
        // PB8 = SCL (I2C1) - AF4
        // PB9 = SDA (I2C1) - AF4
        let scl = gpiob.pb8.into_alternate::<4>().set_open_drain();
        let sda = gpiob.pb9.into_alternate::<4>().set_open_drain();

        // Configure I2C1 with 100 kHz clock (standard mode)
        let i2c = I2c::new(dp.I2C1, (scl, sda), Mode::standard(100.kHz()), &clocks);

        Self {
            config: HardwareConfig { led },
            i2c,
        }
    }
}
