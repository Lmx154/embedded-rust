# Serial Output and RTT Debugging Guide

This document provides comprehensive instructions for viewing debug output from your STM32 NUCLEO-F446RE embedded Rust project using various methods including RTT (Real Time Transfer) and traditional UART serial communication.

## Table of Contents
- [Method 1: RTT with probe-rs (Recommended)](#method-1-rtt-with-probe-rs-recommended)
- [Method 2: Traditional UART Serial](#method-2-traditional-uart-serial)
- [Method 3: Combined RTT + Serial](#method-3-combined-rtt--serial)
- [Troubleshooting](#troubleshooting)

---

## Method 1: RTT with probe-rs (Recommended)

**Real Time Transfer (RTT)** is a modern debugging technique that provides fast, bidirectional communication between your embedded device and host computer without requiring additional hardware like UART pins.

### What is RTT?

RTT works by:
- Using memory buffers in RAM for communication
- Debugger reads/writes these buffers via SWD/JTAG
- No additional pins required (uses existing debug interface)
- Much faster than traditional UART
- Non-blocking (won't halt your program)

### Prerequisites

Your project is already configured for RTT with:
- `rtt-target` crate in `Cargo.toml`
- RTT enabled in `Embed.toml`
- RTT macros in your code (`rtt_init_print!()`, `rprintln!()`)

### Basic RTT Commands

#### Starting RTT Session
```powershell
# Connect to target and start RTT logging
probe-rs rtt --chip STM32F446RETx

# Alternative: specify the binary (if multiple targets)
probe-rs rtt --chip STM32F446RETx target/thumbv7em-none-eabihf/debug/marv
```

#### RTT with Automatic Target Detection
```powershell
# Let probe-rs auto-detect the target (if only one connected)
probe-rs rtt
```

#### RTT Session Controls

Once in RTT session:
- **View output**: Debug messages appear in real-time
- **Send input**: Type and press Enter to send data to target
- **Exit**: Press `Ctrl+C` to exit RTT session
- **Clear screen**: `Ctrl+L` (Linux/macOS) or `cls` then Enter (Windows)

### Advanced RTT Commands

#### RTT with Custom Configuration
```powershell
# Specify RTT control block address (if needed)
probe-rs rtt --chip STM32F446RETx --control-block-address 0x20000000

# Connect to specific RTT channel
probe-rs rtt --chip STM32F446RETx --up-channel 0 --down-channel 0
```

#### RTT Log to File
```powershell
# Save RTT output to file
probe-rs rtt --chip STM32F446RETx > debug_output.log

# Append to existing file
probe-rs rtt --chip STM32F446RETx >> debug_output.log
```

#### RTT with Timestamp
```powershell
# Add timestamps to RTT output (PowerShell)
probe-rs rtt --chip STM32F446RETx | ForEach-Object { "$(Get-Date -Format 'HH:mm:ss.fff'): $_" }
```

### RTT in Different Scenarios

#### During Development (Flash and Monitor)
```powershell
# Terminal 1: Flash and run
cargo run

# Terminal 2: Monitor RTT output
probe-rs rtt --chip STM32F446RETx
```

#### RTT with Running Program
```powershell
# If program is already running on target
probe-rs rtt --chip STM32F446RETx
```

#### RTT with Reset
```powershell
# Reset target and immediately start RTT
probe-rs rtt --chip STM32F446RETx --reset
```

---

## Method 2: Traditional UART Serial

For traditional serial communication over UART pins. This method requires configuring UART in your Rust code and connecting to the appropriate pins.

### Hardware Setup

#### NUCLEO-F446RE UART Pins
- **USART2** (default, connected to ST-Link virtual COM port):
  - TX: PA2 (Arduino D1)
  - RX: PA3 (Arduino D0)
- **USART1**:
  - TX: PA9
  - RX: PA10
- **USART3**:
  - TX: PB10
  - RX: PB11

### Code Configuration

First, update your `Cargo.toml` to include UART support:
```toml
[dependencies]
# ...existing dependencies...
nb = "1.0"
```

Example UART code (add to your `src/main.rs`):
```rust
use stm32f4xx_hal::{
    pac,
    prelude::*,
    serial::{config::Config, Serial},
};
use nb::block;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();

    // Configure UART pins
    let gpioa = dp.GPIOA.split();
    let tx_pin = gpioa.pa2.into_alternate();
    let rx_pin = gpioa.pa3.into_alternate();

    // Configure UART
    let mut serial = Serial::new(
        dp.USART2,
        (tx_pin, rx_pin),
        Config::default().baudrate(115200.bps()),
        &clocks,
    ).unwrap();

    loop {
        // Send data
        for byte in b"Hello from UART!\r\n" {
            block!(serial.write(*byte)).ok();
        }

        // Simple delay
        for _ in 0..1_000_000 {
            cortex_m::asm::nop();
        }
    }
}
```

### Connecting to Serial Port

#### Windows - Using Built-in Tools

**Find COM Port:**
```powershell
# List all COM ports
Get-WmiObject -query "SELECT * FROM Win32_PnPEntity" | Where-Object {$_.Name -like "*COM*"} | Select-Object Name, DeviceID

# Or use Device Manager
devmgmt.msc
```

**Connect with PowerShell:**
```powershell
# Using .NET SerialPort (basic)
$port = New-Object System.IO.Ports.SerialPort("COM3", 115200)
$port.Open()
$port.ReadLine()  # Read one line
$port.Close()
```

#### Windows - Using PuTTY

1. Download PuTTY from: https://www.putty.org/
2. Run PuTTY
3. Select "Serial" connection type
4. Set serial line: `COM3` (replace with your COM port)
5. Set speed: `115200`
6. Click "Open"

**PuTTY Command Line:**
```powershell
putty -serial COM3 -sercfg 115200,8,n,1,N
```

#### Windows - Using Windows Terminal

```powershell
# Install Windows Terminal if not available
winget install Microsoft.WindowsTerminal

# Connect to serial port (requires additional tools)
# Recommended: Use PuTTY or dedicated serial terminal
```

#### Cross-Platform - Using screen (Linux/macOS)

```bash
# List serial devices
ls /dev/tty* | grep -E "(USB|ACM)"

# Connect with screen
screen /dev/ttyACM0 115200

# Exit screen: Ctrl+A, then K, then Y
```

#### Cross-Platform - Using minicom (Linux)

```bash
# Install minicom
sudo apt-get install minicom  # Ubuntu/Debian
sudo dnf install minicom      # Fedora

# Configure minicom
sudo minicom -s

# Connect
minicom -D /dev/ttyACM0 -b 115200

# Exit: Ctrl+A, then X
```

#### Cross-Platform - Using picocom (Linux)

```bash
# Install picocom
sudo apt-get install picocom  # Ubuntu/Debian

# Connect
picocom -b 115200 /dev/ttyACM0

# Exit: Ctrl+A, then Ctrl+X
```

### Serial Terminal Settings

**Standard Settings:**
- **Baud Rate**: 115200 (most common)
- **Data Bits**: 8
- **Parity**: None
- **Stop Bits**: 1
- **Flow Control**: None (RTS/CTS disabled)

**Alternative Baud Rates:**
- 9600 (slow, very reliable)
- 38400 (moderate)
- 57600 (faster)
- 230400 (very fast)
- 460800 (maximum for many systems)

---

## Method 3: Combined RTT + Serial

You can use both RTT and UART simultaneously for different purposes.

### Example Use Cases

- **RTT**: Fast debug messages, performance data
- **UART**: User interface, data logging, communication with other devices

### Code Example

```rust
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::serial::{config::Config, Serial};
use nb::block;

#[entry]
fn main() -> ! {
    // Initialize RTT
    rtt_init_print!();
    rprintln!("RTT initialized");

    // Initialize hardware
    let dp = pac::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();

    // Setup UART
    let gpioa = dp.GPIOA.split();
    let tx_pin = gpioa.pa2.into_alternate();
    let rx_pin = gpioa.pa3.into_alternate();
    
    let mut serial = Serial::new(
        dp.USART2,
        (tx_pin, rx_pin),
        Config::default().baudrate(115200.bps()),
        &clocks,
    ).unwrap();

    let mut counter = 0u32;

    loop {
        // Fast debug info via RTT
        rprintln!("Debug: Loop iteration {}", counter);

        // Slower user data via UART
        let uart_msg = format!("Counter: {}\r\n", counter);
        for byte in uart_msg.bytes() {
            block!(serial.write(byte)).ok();
        }

        counter += 1;

        // Delay
        for _ in 0..1_000_000 {
            cortex_m::asm::nop();
        }
    }
}
```

### Monitoring Both Outputs

**Terminal 1 - RTT:**
```powershell
probe-rs rtt --chip STM32F446RETx
```

**Terminal 2 - Serial:**
```powershell
putty -serial COM3 -sercfg 115200,8,n,1,N
```

---

## Troubleshooting

### RTT Issues

#### Issue: "No RTT control block found"
**Solutions:**
1. Ensure `rtt_init_print!()` is called in your code
2. Verify target is running (not halted)
3. Try specifying control block address manually
4. Check if RTT buffers are being optimized out

```powershell
# Try with explicit control block search
probe-rs rtt --chip STM32F446RETx --control-block-address 0x20000000
```

#### Issue: RTT output garbled or missing
**Solutions:**
1. Verify target clock configuration
2. Check if RTT buffers are large enough
3. Ensure target isn't resetting continuously
4. Try lower RTT buffer sizes in code

#### Issue: "Target not found"
**Solutions:**
1. Check USB connection
2. Verify board is powered
3. Try different USB port
4. Check if another debugger is connected

### Serial Port Issues

#### Issue: "Access denied" or "Port in use"
**Solutions:**
1. Close other applications using the port
2. Unplug and reconnect USB cable
3. Check if ST-Link virtual COM port drivers are installed
4. Try different COM port

#### Issue: No output on serial terminal
**Solutions:**
1. Verify baud rate matches (115200)
2. Check UART pin configuration
3. Verify UART is properly initialized in code
4. Test with known working serial device

#### Issue: Garbled output
**Solutions:**
1. Check baud rate settings
2. Verify clock configuration
3. Check for electrical noise
4. Try lower baud rate (9600)

### General Debug Tips

#### Verify Hardware
```powershell
# Check if ST-Link is detected
probe-rs info --chip STM32F446RETx

# List available probe-rs targets
probe-rs chip list | findstr STM32F446
```

#### Check Code
1. Ensure RTT is initialized: `rtt_init_print!()`
2. Use correct print macros: `rprintln!()`, not `println!()`
3. Verify UART pin configuration matches hardware
4. Check clock configuration

#### Monitor Resource Usage
```rust
// Add to your code for debugging
rprintln!("Free RAM: {} bytes", free_ram_size());
rprintln!("Stack usage: {} bytes", stack_usage());
```

### Performance Comparison

| Method | Speed | CPU Usage | Pins Required | Reliability |
|--------|-------|-----------|---------------|-------------|
| **RTT** | Very Fast | Very Low | 0 (uses SWD) | Excellent |
| **UART 115200** | Moderate | Low | 2 (TX/RX) | Good |
| **UART 9600** | Slow | Very Low | 2 (TX/RX) | Excellent |

### Recommended Workflow

1. **Development**: Use RTT for fast debug output
2. **Testing**: Use both RTT and UART to verify functionality
3. **Production**: Remove RTT, keep UART for user communication
4. **Field Debug**: Add RTT back temporarily for debugging

This guide covers all major methods for viewing output from your STM32 embedded Rust project. RTT is recommended for development due to its speed and convenience, while UART is useful for production communication needs.
