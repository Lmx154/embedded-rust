# Embedded Rust Project for STM32 NUCLEO-F446RE

This project is configured for the **STM32 NUCLEO-F446RE** development board which uses the **STM32F446RET6** microcontroller. The project demonstrates basic embedded Rust functionality including LED blinking and RTT debugging output.

## Quick Start

1. **Build the project:**
   ```
   cargo build
   ```

2. **Flash and run (with probe-rs):**
   ```
   cargo run
   ```

3. **Monitor RTT output:**
   The program outputs debug messages via RTT which can be viewed with `probe-rs` or other RTT viewers.

## Hardware Configuration

This project is configured for the NUCLEO-F446RE board with the following features:
- Built-in LED on PA5 (will blink)
- RTT debugging output
- Proper memory layout for STM32F446RE

## Project Configuration Files

### 1. **Embed.toml** - probe-rs Configuration
```toml
[default.general]
chip = "STM32F446RETx"

[default.rtt]
enabled = true
```

**Purpose**: Configuration file for `probe-rs`, the modern debugging and flashing tool for embedded Rust.

**Key Settings**:
- `chip`: Specifies the exact microcontroller variant (STM32F446RETx for NUCLEO-F446RE)
- `rtt.enabled`: Enables Real Time Transfer (RTT) for debugging output without a UART

**For STM32 Blackpill (STM32F401CCU6)**:
```toml
[default.general]
chip = "STM32F401CCUx"

[default.rtt]
enabled = true
```

### 2. **memory.x** - Memory Layout Definition
```
MEMORY
{
    FLASH : ORIGIN = 0x08000000, LENGTH = 512K
    RAM : ORIGIN = 0x20000000, LENGTH = 128K
}
```

**Purpose**: Defines the memory layout for the linker. This is crucial for proper code placement and memory management.

**STM32F446RET6 Memory Layout**:
- **FLASH**: 512KB starting at address 0x08000000
- **RAM**: 128KB starting at address 0x20000000

**For STM32 Blackpill (STM32F401CCU6)**:
```
MEMORY
{
    FLASH : ORIGIN = 0x08000000, LENGTH = 256K
    RAM : ORIGIN = 0x20000000, LENGTH = 64K
}
```
**Key Differences**:
- STM32F401CCU6 has 256KB flash (vs 512KB on STM32F446)
- STM32F401CCU6 has 64KB RAM (vs 128KB on STM32F446)

### 3. **.cargo/config.toml** - Cargo Build Configuration
```toml
[build]
target = "thumbv7em-none-eabihf"

[target.thumbv7em-none-eabihf]
rustflags = ["-C", "link-arg=-Tlink.x"]
```

**Purpose**: Configures Cargo's build behavior for embedded targets.

**Key Settings**:
- `target`: Specifies the Rust target triple for ARM Cortex-M4F with hardware floating point
- `rustflags`: Tells the linker to use the `link.x` script provided by `cortex-m-rt`

**For STM32 Blackpill**: The configuration remains the same since STM32F401 and STM32F446 are both Cortex-M4F with FPU.

### 4. **.vscode/settings.json** - VS Code Integration
```json
{
    "rust-analyzer.check.allTargets": false,
    "rust-analyzer.cargo.target": "thumbv7em-none-eabihf"
}
```

**Purpose**: Configures rust-analyzer in VS Code for embedded development.

**Key Settings**:
- `check.allTargets`: Disabled to avoid checking host targets
- `cargo.target`: Forces rust-analyzer to use the embedded target

**For STM32 Blackpill**: Configuration remains the same.

## Target Architecture Explanation

### thumbv7em-none-eabihf Breakdown:
- **thumbv7e**: ARM Thumb-2 instruction set, ARMv7E-M architecture
- **m**: Cortex-M profile (microcontroller)
- **none**: No operating system (bare metal)
- **eabi**: Embedded Application Binary Interface
- **hf**: Hardware floating point support

Both STM32F446 and STM32F401 use Cortex-M4F cores, so they share the same target triple.

## Hardware-Specific Dependencies

### Current Dependencies (Cargo.toml):
```toml
[dependencies]
cortex-m = { version = "0.7.7", features = ["critical-section-single-core"] }
cortex-m-rt = "0.7.5"
panic-halt = "1.0.0"
rtt-target = "0.6.1"
stm32f4xx-hal = { version = "0.21", features = ["stm32f446"] }
```

### For STM32 Blackpill, you would need:
```toml
[dependencies]
cortex-m = { version = "0.7.7", features = ["critical-section-single-core"] }
cortex-m-rt = "0.7.5"
panic-halt = "1.0.0"
rtt-target = "0.6.1"
stm32f4xx-hal = { version = "0.21", features = ["stm32f401"] }
```

**HAL dependency**: `stm32f4xx-hal` provides hardware abstraction layer for STM32F4 family with specific feature flags for different variants.

## Setup Commands

### Initial Setup on New Machine:

```powershell
# Install the ARM Cortex-M target
rustup target add thumbv7em-none-eabihf

# Install the project (if it contains binary crates)
cargo install --path .

# Install probe-rs tools for debugging and flashing
cargo install probe-rs-tools
```

### Additional Setup for STM32 Development:
```powershell
# Install probe-rs with STM32 support
probe-rs chip list | findstr STM32F446

# Verify the chip is supported
probe-rs info --chip STM32F446RETx

# For STM32F401 Blackpill
probe-rs chip list | findstr STM32F401
probe-rs info --chip STM32F401CCUx
```

## Key Differences: STM32F446 vs STM32F401

| Aspect | STM32F446 (NUCLEO-F446RE) | STM32F401 (Blackpill) |
|--------|----------------------------|------------------------|
| **Core** | ARM Cortex-M4F @ 180MHz | ARM Cortex-M4F @ 84MHz |
| **Flash** | 512KB @ 0x08000000 | 256KB @ 0x08000000 |
| **RAM** | 128KB @ 0x20000000 | 64KB @ 0x20000000 |
| **Chip ID** | STM32F446RETx | STM32F401CCUx |
| **Main Features** | USB OTG, CAN, SAI, advanced timers | USB OTG, basic peripherals |
| **HAL Feature** | `stm32f446` | `stm32f401` |

## Migration Checklist: STM32F446 → STM32F401

1. **Update Embed.toml**: Change chip from "STM32F446RETx" to "STM32F401CCUx"
2. **Update memory.x**: Update sizes (256KB flash, 64KB RAM for F401)
3. **Update Cargo.toml**: Change HAL feature from "stm32f446" to "stm32f401"
4. **Update source code**: Adjust clock speeds and peripheral configurations for F401 limitations
5. **Test probe-rs**: Verify the chip is detected with `probe-rs info --chip STM32F401CCUx`

The target triple, cargo config, and VS Code settings remain the same since both chips use the same ARM Cortex-M4F architecture.

## Hardware-Specific Features

### STM32F446RET6 (NUCLEO-F446RE) Features:
- **Built-in LED**: PA5 (User LED LD2)
- **User Button**: PC13 (Blue push-button B1)
- **ST-LINK/V2-1**: Integrated debugger/programmer
- **Arduino Uno R3 compatibility**: Pin layout compatible
- **Morpho connectors**: Extended pin access
- **Clock**: Up to 180MHz with PLL
- **Advanced peripherals**: CAN, SAI, QSPI, Camera interface

### Example Code Features:
The included example demonstrates:
- System clock configuration to 180MHz
- GPIO control for the built-in LED (PA5)
- RTT debugging output
- Basic delay loop using cortex-m nop instructions