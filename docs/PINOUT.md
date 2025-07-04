# STM32F446RE Nucleo Board Pinout Configuration

This document describes the pin assignments for the embedded Rust project on the STM32F446RE Nucleo board.

## GPIO Pin Assignments

### Status Indicators
| Pin | Function | Description |
|-----|----------|-------------|
| PA5 | LED | Built-in green LED on Nucleo board for status indication |

### GPS Module (u-blox NEO-M9N)
| STM32 Pin | Function | GPS Module Pin | Alternate Function | Description |
|-----------|----------|----------------|-------------------|-------------|
| PA9 | USART1_TX | GPS RX | AF7 | Data transmission from STM32 to GPS |
| PA10 | USART1_RX | GPS TX | AF7 | Data reception from GPS to STM32 |

### Communication Configuration
- **UART Peripheral**: USART1
- **Baud Rate**: 38400 bps (**default for NEO-M9N-00B and GNSS 7 Click**)
- **Data Format**: 8N1 (8 data bits, no parity, 1 stop bit)
- **Default Protocol**: UBX binary protocol (factory default)
- **NMEA Output**: Enabled by firmware at 38400 baud by sending a UBX configuration command at startup

## Wiring Connections

### GPS Module Connections
```
u-blox NEO-M9N    →    STM32F446RE Nucleo
─────────────────      ─────────────────────
VCC (3.3V)       →    3.3V
GND               →    GND  
TX                →    PA10 (USART1_RX)
RX                →    PA9  (USART1_TX)
```

## Notes

- **Voltage Level**: The GPS module operates at 3.3V logic levels, which is compatible with the STM32F446RE
- **Power Supply**: Ensure GPS module is powered from 3.3V rail, not 5V
- **Default Protocol**: u-blox NEO-M9N-00B and GNSS 7 Click default to UBX binary protocol at 38400 baud
- **NMEA Output**: The firmware sends a UBX configuration command at startup to enable NMEA 0183 output at 38400 baud
- **Alternate Function**: Both UART pins use Alternate Function 7 (AF7) for USART1

## Pin Availability

The following pins are currently **occupied** and should not be used for other functions:
- PA5 (LED)
- PA9 (GPS UART TX)
- PA10 (GPS UART RX)

All other GPIO pins remain available for additional peripherals and sensors.
