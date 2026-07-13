# Fuel Intelligence Platform — ESP32 Hardware Bring-up

## Board

LILYGO T-SIM / T-A7670E ESP32 board.

## Confirmed Pin Map

### Modem

| Function               |   GPIO |
| ---------------------- | -----: |
| ESP TX to modem RX     | GPIO26 |
| ESP RX from modem TX   | GPIO27 |
| Modem PWRKEY           |  GPIO4 |
| Modem RESET            |  GPIO5 |
| Board peripheral power | GPIO12 |

### SD Card

| Function |   GPIO |
| -------- | -----: |
| SD MISO  |  GPIO2 |
| SD MOSI  | GPIO15 |
| SD SCLK  | GPIO14 |
| SD CS    | GPIO13 |

### External GPS Header

| Function   |   GPIO |
| ---------- | -----: |
| GPS TX     | GPIO21 |
| GPS RX     | GPIO22 |
| GPS PPS    | GPIO23 |
| GPS WAKEUP | GPIO19 |

## Build Environment

Before building in WSL, load the ESP toolchain:

```bash
source ~/export-esp.sh

Then build:

cd ~/projects/fuel-intelligence-platform/hardware-bringup/board-hello
cargo build --release
Flash Command

Run from PowerShell:

espflash flash --monitor --ignore-app-descriptor --port COM3 "\\wsl.localhost\Ubuntu\home\william\projects\fuel-intelligence-platform\hardware-bringup\board-hello\target\xtensa-esp32-none-elf\release\board-hello"
Monitor Only
espflash monitor --port COM3
Important Flashing Rule

Do not flash firmware while the SD card is inserted.

Use this order:

Remove SD card.
Flash firmware.
Stop monitor.
Insert SD card if needed.
Run monitor only.
Press RST.
Confirmed Hardware Tests
Test	Status
ESP32 boot	Done
ESP32 flash	Done
Serial monitor	Done
SD card SPI bus	Done
SD card detect	Done
SD card read/open volume	Done
SD card write	Done
A7670E AT command	Done
SIM detection	Done
LTE signal	Done
LTE registration	Done
Packet data attach	Done
IP address allocation	Done
HTTP request	Pending
GNSS/GPS	Pending
Fuel sensor	Pending
Vibration sensor	Pending
Verified Modem Results
Basic AT
AT
OK
SIM Status
AT+CPIN?
+CPIN: READY
OK
Signal Strength

Working signal example:

AT+CSQ
+CSQ: 25,99
OK
Operator
AT+COPS?
+COPS: 0,2,"23420",7
OK

23420 is the connected operator code.

7 means LTE.

LTE Registration
AT+CEREG?
+CEREG: 0,1
OK
Packet Data Attached
AT+CGATT?
+CGATT: 1
OK
IP Address
AT+CGPADDR
+CGPADDR: 1,10.20.80.36
OK

This confirms the modem received an IP address.

Current Source Structure
src/
├── main.rs
├── board.rs
└── modem.rs
Current Module Responsibility
board.rs

Owns the board-level control pins:

GPIO12 board peripheral power
GPIO5 modem reset
GPIO4 modem power key
modem.rs

Owns:

UART setup
modem power-on sequence
sending AT commands
reading/printing modem responses
Next Steps
Add HTTP modem test.
Add SD logging module.
Log modem diagnostics to FUEL_LOG.TXT.
Add GNSS/GPS test.
Add fuel sensor test.
Add vibration sensor test.
Build reusable diagnostic firmware for new boards.
Later flash production firmware.
```
