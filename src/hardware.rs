use stm32f4xx_hal::{
    pac,
    prelude::*,
    gpio::{Pin, Output, PushPull},
    serial::{Config, Serial, Tx, Rx},
};

pub type LedPin = Pin<'A', 5, Output<PushPull>>;
pub type UartTx = Tx<pac::USART1>;
pub type UartRx = Rx<pac::USART1>;

pub struct HardwareConfig {
    pub led: LedPin,
    pub uart_tx: UartTx,
    pub uart_rx: UartRx,
}

impl HardwareConfig {
    pub fn new() -> Self {
        // Get access to the device specific peripherals
        let dp = pac::Peripherals::take().unwrap();

        // Take ownership over the raw flash and rcc devices and convert them into the corresponding
        // HAL structs
        let rcc = dp.RCC.constrain();

        // Freeze the configuration of all the clocks in the system and store the frozen frequencies
        let clocks = rcc.cfgr.freeze();

        // Acquire the GPIOA peripheral
        let gpioa = dp.GPIOA.split();

        // Configure PA5 (built-in LED on Nucleo-F446RE) as a push-pull output
        let led = gpioa.pa5.into_push_pull_output();

        // Configure UART pins
        // PA9 = TX (output to GPS RX) - AF7
        // PA10 = RX (input from GPS TX) - AF7
        let tx_pin = gpioa.pa9.into_alternate::<7>();
        let rx_pin = gpioa.pa10.into_alternate::<7>();

        // Configure UART1 (USART1) with 38400 baud rate (default for NEO-M9N-00B)
        let config = Config::default().baudrate(38400.bps());
        let uart = Serial::new(dp.USART1, (tx_pin, rx_pin), config, &clocks).unwrap();
        let (uart_tx, uart_rx) = uart.split();

        Self {
            led,
            uart_tx,
            uart_rx,
        }
    }
}
