# Flashing Guide for STM32 Projects

This document provides comprehensive instructions for flashing your embedded Rust project to STM32 boards using various methods and tools.

**Supported Targets:**
- STM32 NUCLEO-F446RE (default)
- STM32F401CCU6 "Black Pill"

## Table of Contents
- [Method 1: probe-rs (Recommended)](#method-1-probe-rs-recommended)
- [Method 2: ST-Link Tools](#method-2-st-link-tools)
- [Method 3: DFU (Device Firmware Update)](#method-3-dfu-device-firmware-update)
- [Method 4: OpenOCD + GDB](#method-4-openocd--gdb)
- [Method 5: STM32CubeProgrammer](#method-5-stm32cubeprogrammer)
- [Troubleshooting](#troubleshooting)

## Prerequisites

Before flashing, ensure your project builds successfully:
```powershell
cargo build
```

## Method 1: probe-rs (Recommended)

**probe-rs** is the modern, Rust-native tool for embedded development. It's fast, reliable, and well-integrated with the Rust ecosystem.

### Installation
```powershell
# Install probe-rs
cargo install probe-rs-tools

# Verify installation
probe-rs --version
```

### Setup
The project is already configured for probe-rs via `Embed.toml` and `.cargo/config.toml`.

### Flashing Commands

#### Option A: Using cargo run (Recommended)

**For STM32F446RE Nucleo (default):**
```powershell
# Build and flash in one command
cargo run

# With release optimization
cargo run --release
```

**For STM32F401CCU6 Black Pill:**
```powershell
# Build and flash with F401 configuration
cargo run --config f401

# With release optimization
cargo run --config f401 --release
```

#### Option B: Using probe-rs directly

**For STM32F446RE Nucleo:**
```powershell
# Build first
cargo build

# Flash the binary
probe-rs run --chip STM32F446RETx target/thumbv7em-none-eabihf/debug/marv

# Start RTT session
probe-rs rtt --chip STM32F446RETx
```

**For STM32F401CCU6 Black Pill:**
```powershell
# Build with F401 feature
cargo build --features stm32f401

# Flash the binary
probe-rs run --chip STM32F401CCUx target/thumbv7em-none-eabihf/debug/marv

# Start RTT session
probe-rs rtt --chip STM32F401CCUx
```

# Flash the binary
probe-rs run --chip STM32F446RETx target/thumbv7em-none-eabihf/debug/marv

# For release build
probe-rs run --chip STM32F446RETx target/thumbv7em-none-eabihf/release/marv
```

#### Option C: Flash without running
```powershell
# Just flash (don't start execution)
probe-rs download --chip STM32F446RETx target/thumbv7em-none-eabihf/debug/marv
```

### Debugging with probe-rs
```powershell
# Start GDB server
probe-rs gdb --chip STM32F446RETx target/thumbv7em-none-eabihf/debug/marv

# In another terminal, connect with GDB
arm-none-eabi-gdb target/thumbv7em-none-eabihf/debug/marv
(gdb) target remote :1337
(gdb) load
(gdb) continue
```

### RTT Logging
```powershell
# View RTT output in real-time
probe-rs rtt --chip STM32F446RETx
```

---

## Method 2: ST-Link Tools

The NUCLEO-F446RE has an integrated ST-LINK/V2-1 debugger/programmer.

### Installation
Download and install **STM32 ST-LINK Utility** or **STM32CubeProgrammer** from STMicroelectronics website.

### Using ST-LINK Utility (Windows)

#### Command Line Interface
```powershell
# Build the project first
cargo build

# Convert ELF to HEX (if needed)
arm-none-eabi-objcopy -O ihex target/thumbv7em-none-eabihf/debug/marv target/thumbv7em-none-eabihf/debug/marv.hex

# Flash using ST-LINK CLI
ST-LINK_CLI.exe -c SWD -P target/thumbv7em-none-eabihf/debug/marv.hex -Rst
```

#### GUI Method
1. Open **STM32 ST-LINK Utility**
2. Connect to the board: **Target** → **Connect**
3. Load the hex file: **File** → **Open file** → Select `marv.hex`
4. Flash: **Target** → **Program & Verify**
5. Reset and run: **Target** → **Reset** → **Software system reset**

### Using st-flash (Linux/Cross-platform)

#### Installation
```bash
# Ubuntu/Debian
sudo apt-get install stlink-tools

# macOS with Homebrew
brew install stlink

# Windows with MSYS2
pacman -S mingw-w64-x86_64-stlink
```

#### Flashing Commands
```powershell
# Build first
cargo build

# Flash the binary directly
st-flash write target/thumbv7em-none-eabihf/debug/marv 0x08000000

# Reset the board
st-flash reset
```

---

## Method 3: DFU (Device Firmware Update)

DFU allows flashing over USB without an external debugger. You need to put the STM32 into DFU mode.

### Prerequisites
- **dfu-util** installed
- USB cable
- STM32 in DFU bootloader mode

### Installation of dfu-util

#### Windows
```powershell
# Using chocolatey
choco install dfu-util

# Or download from: http://dfu-util.sourceforge.net/releases/
```

#### Linux
```bash
# Ubuntu/Debian
sudo apt-get install dfu-util

# Fedora
sudo dnf install dfu-util
```

### Entering DFU Mode

#### Method A: Hardware (Boot pins)
1. Disconnect USB cable
2. Set **BOOT0** jumper to 1 (3.3V) - may require soldering on NUCLEO boards
3. Reset the board or reconnect USB
4. Board should appear as DFU device

#### Method B: Software (if supported)
```powershell
# Some STM32 boards support software DFU entry
# This varies by board and bootloader
```

#### Method C: Button combination (board-specific)
Some boards enter DFU mode with specific button combinations during power-on.

### Flashing with DFU

```powershell
# Build the project
cargo build

# Convert ELF to DFU format
arm-none-eabi-objcopy -O binary target/thumbv7em-none-eabihf/debug/marv target/thumbv7em-none-eabihf/debug/marv.bin

# List DFU devices
dfu-util -l

# Flash the binary (adjust alt setting if needed)
dfu-util -a 0 -s 0x08000000:leave -D target/thumbv7em-none-eabihf/debug/marv.bin

# Alternative with specific device
dfu-util -d 0483:df11 -a 0 -s 0x08000000:leave -D target/thumbv7em-none-eabihf/debug/marv.bin
```

### Exiting DFU Mode
1. Set **BOOT0** back to 0 (GND)
2. Reset the board
3. Your application should start running

---

## Method 4: OpenOCD + GDB

OpenOCD provides a flexible debugging and flashing solution.

### Installation

#### Windows
Download OpenOCD from: https://github.com/xpack-dev-tools/openocd-xpack/releases

#### Linux
```bash
# Ubuntu/Debian
sudo apt-get install openocd

# Fedora
sudo dnf install openocd
```

### Configuration

Create `openocd.cfg` in your project root:
```cfg
# OpenOCD configuration for STM32F446RE via ST-Link
source [find interface/stlink.cfg]
source [find target/stm32f4x.cfg]

# Reset configuration
reset_config srst_only
```

### Flashing Commands

#### Terminal 1: Start OpenOCD server
```powershell
# Start OpenOCD server
openocd -f openocd.cfg
```

#### Terminal 2: Connect with GDB and flash
```powershell
# Build first
cargo build

# Start GDB
arm-none-eabi-gdb target/thumbv7em-none-eabihf/debug/marv

# In GDB prompt:
(gdb) target remote :3333
(gdb) monitor reset halt
(gdb) load
(gdb) monitor reset run
(gdb) quit
```

#### Alternative: Direct OpenOCD flashing
```powershell
# Convert to hex first
arm-none-eabi-objcopy -O ihex target/thumbv7em-none-eabihf/debug/marv target/thumbv7em-none-eabihf/debug/marv.hex

# Flash with OpenOCD
openocd -f openocd.cfg -c "program target/thumbv7em-none-eabihf/debug/marv.hex verify reset exit"
```

---

## Method 5: STM32CubeProgrammer

STM32CubeProgrammer is ST's official, modern programming tool.

### Installation
Download from STMicroelectronics website: https://www.st.com/en/development-tools/stm32cubeprog.html

### GUI Method
1. Open **STM32CubeProgrammer**
2. Select connection method: **ST-LINK**
3. Click **Connect**
4. Load file: **Open file** → Select your `.hex` or `.bin` file
5. Set start address: `0x08000000` (for .bin files)
6. Click **Download**
7. Click **Disconnect**

### Command Line Interface
```powershell
# Build and convert first
cargo build
arm-none-eabi-objcopy -O ihex target/thumbv7em-none-eabihf/debug/marv target/thumbv7em-none-eabihf/debug/marv.hex

# Flash using STM32CubeProgrammer CLI
STM32_Programmer_CLI.exe -c port=SWD -d target/thumbv7em-none-eabihf/debug/marv.hex -s

# For binary files
STM32_Programmer_CLI.exe -c port=SWD -d target/thumbv7em-none-eabihf/debug/marv.bin 0x08000000 -s
```

---

## Troubleshooting

### Common Issues and Solutions

#### Issue: "No device found" or "Cannot connect"
**Solutions:**
1. Check USB cable connection
2. Verify board power (LED indicators)
3. Try different USB port
4. Install/update ST-Link drivers
5. Reset the board
6. Check if another debugger is connected

#### Issue: "Permission denied" (Linux)
**Solution:**
```bash
# Add user to dialout group
sudo usermod -a -G dialout $USER

# Create udev rules for ST-Link
sudo nano /etc/udev/rules.d/99-stlink.rules

# Add these lines:
SUBSYSTEM=="usb", ATTRS{idVendor}=="0483", ATTRS{idProduct}=="3748", MODE="0666"
SUBSYSTEM=="usb", ATTRS{idVendor}=="0483", ATTRS{idProduct}=="374b", MODE="0666"

# Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger
```

#### Issue: "Target voltage too low" or "No target detected"
**Solutions:**
1. Check board power supply
2. Verify all jumpers are correctly positioned
3. Try connecting under reset (hold reset while connecting)
4. Check SWD connections if using external debugger

#### Issue: DFU device not detected
**Solutions:**
1. Verify BOOT0 pin is correctly set to VDD
2. Install DFU drivers (Windows)
3. Try different USB cable/port
4. Check if board supports DFU mode
5. Use ST's DfuSe Demonstration software for driver installation

#### Issue: "Flash programming failed"
**Solutions:**
1. Verify target memory addresses
2. Check if flash is write-protected
3. Erase flash before programming
4. Verify binary size doesn't exceed flash capacity

#### Issue: Program doesn't start after flashing
**Solutions:**
1. Verify BOOT0 is set to GND for normal operation
2. Check if reset vector is correctly programmed
3. Verify clock configuration
4. Reset the board manually

### Verification Commands

After flashing, you can verify the programming:

```powershell
# Using probe-rs
probe-rs info --chip STM32F446RETx

# Using st-flash
st-flash --version

# Check if RTT is working (if your program uses it)
probe-rs rtt --chip STM32F446RETx
```

### Driver Installation (Windows)

If you encounter driver issues on Windows:

1. **ST-Link Drivers**: Download from ST website or use ST-Link Utility installer
2. **DFU Drivers**: Use STM32CubeProgrammer installer or Zadig tool
3. **WinUSB Drivers**: For probe-rs, may need WinUSB drivers via Zadig

### Recommended Workflow

For daily development, the recommended workflow is:

1. **Development**: Use `cargo run` with probe-rs for fast iteration
2. **Debugging**: Use `probe-rs gdb` or integrated IDE debugging
3. **Production**: Use STM32CubeProgrammer for final programming
4. **Field updates**: Use DFU if supported by your application

This covers all major flashing methods for the STM32 NUCLEO-F446RE. Choose the method that best fits your development environment and requirements.
