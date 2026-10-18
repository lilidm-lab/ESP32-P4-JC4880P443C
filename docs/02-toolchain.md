# 02 — Toolchain & Build (working setup)

Verified working state: `cargo check`, `cargo clippy`, `cargo run --release`
(flash + monitor) all pass on this machine.

## Version pins (why each exists)

| Component | Pinned value | Where | Reason |
|---|---|---|---|
| ESP-IDF | `v5.4.2` | `.cargo/config.toml` → `ESP_IDF_VERSION` | Board is chip rev **v1.3**; IDF 5.5.x defaults target rev ≥ 3.01 and crashes (`Illegal instruction`) |
| Rust nightly | `nightly-2026-07-10` (1.99) | `rust-toolchain.toml` | Latest nightly (1.101, 2026-10-14) has an ICE in `compare_impl_item.rs` compiling `embedded_svc` |
| esp_hosted | `^2.11.0` | `Cargo.toml` metadata | Spec's `^1.0.0` does not link against modern IDF |
| esp_wifi_remote | `^1.6.0` | `Cargo.toml` metadata | wifi_remote 1.6.x requires hosted ≥ 2.11 |
| espflash | 4.4.0 | `~/.cargo/bin/espflash` | `--bootloader` flag available for explicit flash |

## Updating tools

```sh
rustup update          # Rust compiler (RISC-V/std — what this project uses)
espup update           # Xtensa toolchain + LLVM (not needed for P4, harmless)
```

Neither of these touches ESP-IDF. IDF is managed by `embuild` inside
`esp-idf-sys`: it clones and builds IDF per `ESP_IDF_VERSION` automatically.
There is no single "update IDF" command — change `ESP_IDF_VERSION` and
rebuild (first build after a change takes ~10-20 min).

EIM (ESP-IDF Installation Manager) is *not* integrated with embuild 0.33.5;
if you ever install IDF via EIM, point the build at it with `IDF_PATH` env var
(embuild then uses it as `EspIdfOrigin::Custom` instead of cloning its own).

## Bootloader facts (verified on device)

- `espflash flash` writes the bootloader built by this project's IDF build
  (from `target/<target>/release/bootloader.bin`). The stale-v5.5.3 episode
  was a leftover build cache; after `rm -rf target/*/build/esp-idf-sys-*`
  the written bootloader matches v5.4.2 (verified by reading 0x2000 back and
  byte-comparing: only the image header + SHA differ, which espflash patches).
- Belt-and-braces: pass it explicitly

  ```sh
  espflash flash --bootloader target/riscv32imafc-esp-espidf/release/bootloader.bin \
      target/riscv32imafc-esp-espidf/release/ESP32-P4-JC4880P443C
  ```
- `espflash` prints "segment 0x2000 unchanged, skipping write" when the
  bootloader already matches — that is healthy, not a bug.

## Known pitfalls

1. **sdkconfig changes are invisible until the esp-idf-sys build dir is
   removed**: `rm -rf target/riscv32imafc-esp-espidf/*/build/esp-idf-sys-*`.
   (Bit us twice: SPIRAM speed, flash size.)
2. **Never bump `ESP_IDF_VERSION` to 5.5+ without also adding to
   `sdkconfig.defaults`:**
   ```
   CONFIG_ESP32P4_SELECTS_REV_LESS_V3=y
   CONFIG_ESP32P4_REV_MIN_100=y
   ```
   and expect DSI/hosted bring-up to need re-verification.
3. **Don't float `channel = "nightly"`** in `rust-toolchain.toml` — pin a
   date; build-std + embedded-svc breakage shows up as compiler ICEs, not
   compile errors in our code.
4. Serial port (`/dev/cu.usbmodem2101`) disappears for ~5 s after flashing;
   wait before reopening the monitor.
