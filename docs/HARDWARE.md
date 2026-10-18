# JC4880P443C / JC4880P433 — Hardware Specification

GUITION 4.3-inch capacitive touch display board built around the GUITION
JC-ESP32P4-M3-C6 module (ESP32-P4 + ESP32-C6). Reference article: cnx-software,
"4.3-inch touch display board features single ESP32-P4 + ESP32-C6 module"
(Aug 12, 2025).

## Module — GUITION JC-ESP32P4-M3-C6

### ESP32-P4 (Primary MCU, Host)

| Block     | Detail                                            |
|-----------|---------------------------------------------------|
| CPU       | Dual-core 32-bit RISC-V HP @ up to 400 MHz, AI extension, single-precision FPU |
| LP core   | Single RISC-V LP MCU @ up to 40 MHz               |
| HP RAM    | 768 KB HP L2MEM (dual-core), 32 KB LP SRAM, 8 KB LP TCM |
| PSRAM     | 32 MB (AP hex-mode part; runs 200 MHz on this project) |
| ROM       | 128 KB HP ROM, 16 KB LP ROM                       |
| GPU       | 2D Pixel Processing Accelerator (PPA)             |
| VPU       | H.264 + JPEG codecs                               |
| Display   | MIPI-DSI controller (480x800 panel on this board) |
| Camera    | MIPI CSI                                          |
| WiFi/BLE  | none — uses the C6 co-processor (ESP-Hosted)      |

Board bring-up fact (this project): chip revision **v1.3**. IDF v5.5+ builds
default to rev 3.01+ and crash on this silicon; the project pins IDF v5.4.2
(see `.cargo/config.toml` and `sdkconfig.defaults`).

### ESP32-C6 (WiFi co-processor, Slave)

WiFi 6 (802.11ax), Bluetooth 5 (LE), 802.15.4 (Zigbee/Thread). Runs the
ESP-Hosted slave firmware; reached from the P4 over SDIO.

### WiFi link (ESP-Hosted over SDIO)

| Function | GPIO | Notes          |
|----------|------|----------------|
| CMD      | 19   | SDIO command   |
| CLK      | 18   | SDIO clock     |
| D0–D3    | 14–17| 4-bit bus @ 40 MHz |
| RESET    | 54   | C6_CHIP_PU     |

RMII pins in the schematic (GPIO 49–35, 30–28, 52, 51) are NOT used for WiFi —
reserved for future Ethernet PHY variants.

## Board

| Block        | Detail                                          |
|--------------|-------------------------------------------------|
| Display      | 4.3" 480x800 IPS capacitive touch, MIPI-DSI     |
| Camera       | MIPI CSI connector (external module)            |
| Storage      | 16 MB NOR flash, microSD slot                   |
| USB          | 2x USB 2.0 Type-C                               |
| Serial       | 4-pin UART0/UART1 header, 4-pin RS-485 header   |
| Expansion    | 26-pin 2.54 mm header: I2C, SPI, PWM, ADC, RMT, MCPWM, TWAI, LEDC; separate I2C connector |
| Audio        | Microphone connector, speaker connector         |
| Antenna      | IPEX connector + ceramic antenna on dev board   |
| Security     | Secure boot, flash encryption, HW crypto, TRNG, key management, digital signature |
| Power        | 5 V via USB-C, lithium battery interface, multiple 3.3 V / 5 V pins |
| Misc         | Power LED; jumpers for power, reset, user buttons |
| Dimensions   | 114.4 x 66.8 mm; module 27 x 27 x 3.4 mm        |
| Temperature  | -20 °C ~ +70 °C                                 |

## Project mapping

| Board feature | Project file |
|---------------|--------------|
| DSI panel init/timing | `src/lcd.rs` |
| WiFi STA via ESP-Hosted | `src/wifi.rs`, ESP-Hosted block in `sdkconfig.defaults` |
| IDF / silicon pin | `.cargo/config.toml` (`ESP_IDF_VERSION=v5.4.2`), note block in `sdkconfig.defaults` |
