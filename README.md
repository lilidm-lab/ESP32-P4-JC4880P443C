# ESP32-P4-JC4880P443C

ESP32-P4 (JC4880P443C) `std` firmware in Rust: MIPI-DSI LCD + WiFi via ESP-Hosted
(ESP32-C6 co-processor over SDIO).

## Prerequisites

```sh
cargo install espup ldproxy espflash
espup install
source ~/export-esp.sh
```

## Build & Flash

```sh
cargo build --release
cargo run --release
```

`cargo run` uses `espflash flash --monitor` (see `.cargo/config.toml`).

Credentials come from build-time env vars (do not commit real values):

```sh
WIFI_SSID="MyNet" WIFI_PASS="MyPass" cargo run --release
```

## Toolchain

Pinned versions (IDF 5.4.2, nightly Rust, esp_hosted 2.11) and the reasons
for each pin: see [docs/02-toolchain.md](docs/02-toolchain.md).

## WiFi (ESP-Hosted)

The P4 has no radio. WiFi runs on the onboard ESP32-C6 slave, driven by the
ESP-Hosted component over SDIO (GPIO 14-19, reset on GPIO 54). Configuration
lives in `sdkconfig.defaults` and the managed components in `Cargo.toml`
(`package.metadata.esp-idf-sys`). Application code uses the standard
`esp_idf_svc::wifi` APIs — no custom transport code.

> Note: the spec's versions (esp_hosted ^1.0.0 / esp_wifi_remote ^0.3.0) do not
> build against IDF v5.5.3; this project pins the compatible pairing
> esp_hosted ^2.11 + esp_wifi_remote ^1.6.

If the C6 slave firmware is missing or too old, flash it once from the
[esp-hosted](https://github.com/espressif/esp-hosted) `network_adapter`
example for `esp32c6`.

## LCD

480x800 MIPI-DSI panel on the DSI bus (`lcd.rs`). Vendor init commands are in
`VENDOR_INIT_CMDS`; adjust the table if your panel variant needs a different
sequence (e.g. ILI9881C instead of ST7701).
