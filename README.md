# Embedded Rust Configuration for Microbit v2 (nRF52833)

This project is configured for the **BBC micro:bit v2** which uses the **nRF52833** microcontroller. The configuration is specified across several important files that define the target architecture, memory layout, build settings, and debugging configuration.

## Important Configuration Files

### 1. **Embed.toml** - probe-rs Configuration
```toml
[default.general]
chip = "nRF52833_xxAA"

[default.rtt]
enabled = true
```

**Purpose**: Configuration file for `probe-rs`, the modern debugging and flashing tool for embedded Rust.

**Key Settings**:
- `chip`: Specifies the exact microcontroller variant (nRF52833_xxAA for micro:bit v2)
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
    FLASH : ORIGIN = 0x00000000, LENGTH = 512K
    RAM : ORIGIN = 0x20000000, LENGTH = 128K
}
```

**Purpose**: Defines the memory layout for the linker. This is crucial for proper code placement and memory management.

**nRF52833 Memory Layout**:
- **FLASH**: 512KB starting at address 0x00000000
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
- STM32 flash starts at `0x08000000` (not 0x00000000)
- STM32F401CCU6 has 256KB flash (vs 512KB on nRF52833)
- STM32F401CCU6 has 64KB RAM (vs 128KB on nRF52833)

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

**For STM32 Blackpill**: The configuration remains the same since STM32F401 is also Cortex-M4F with FPU.

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

Both nRF52833 and STM32F401 use Cortex-M4F cores, so they share the same target triple.

## Hardware-Specific Dependencies

### Current Dependencies (Cargo.toml):
```toml
[dependencies]
cortex-m = { version = "0.7.7", features = ["critical-section-single-core"] }
cortex-m-rt = "0.7.5"
panic-halt = "1.0.0"
rtt-target = "0.6.1"
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

**Additional dependency**: `stm32f4xx-hal` provides hardware abstraction layer for STM32F4 family.

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
probe-rs chip list | findstr STM32F401

# Verify the chip is supported
probe-rs info --chip STM32F401CCUx
```

## Key Differences: nRF52833 vs STM32F401

| Aspect | nRF52833 (micro:bit v2) | STM32F401 (Blackpill) |
|--------|-------------------------|------------------------|
| **Core** | ARM Cortex-M4F @ 64MHz | ARM Cortex-M4F @ 84MHz |
| **Flash** | 512KB @ 0x00000000 | 256KB @ 0x08000000 |
| **RAM** | 128KB @ 0x20000000 | 64KB @ 0x20000000 |
| **Chip ID** | nRF52833_xxAA | STM32F401CCUx |
| **Main Features** | Bluetooth LE, 2.4GHz radio | USB OTG, more timers/peripherals |
| **HAL Crate** | `nrf52833-hal` | `stm32f4xx-hal` |

## Migration Checklist: nRF52833 → STM32F401

1. **Update Embed.toml**: Change chip from "nRF52833_xxAA" to "STM32F401CCUx"
2. **Update memory.x**: Change FLASH origin from 0x00000000 to 0x08000000, and update sizes (256KB flash, 64KB RAM)
3. **Update Cargo.toml**: Replace nRF HAL with `stm32f4xx-hal` using "stm32f401" feature
4. **Update source code**: Replace nRF-specific peripheral code with STM32 equivalents
5. **Test probe-rs**: Verify the chip is detected with `probe-rs info --chip STM32F401CCUx`

The target triple, cargo config, and VS Code settings remain the same since both chips use the same ARM Cortex-M4F architecture.