# GT911 Touch — JC4880P443C_I_W (ESP32-P4)

Debug summary: 2026-10-16 — **SOLVED** (touch is accurate across the entire display, UI fully usable)

## Why touch coordinates were wrong — root causes

### 1. Reading the point register at the wrong offset (root cause of "skewed touch")

The original code read coordinates from register **0x8150**, but the GT911 stores
point data starting at **0x814F**. The layout per point is:

```
register:  814F    8150   8151   8152   8153   8154
value:     track   x_lo   x_hi   y_lo   y_hi   size
```

We read from 0x8150 and decoded as `[x_hi, x_lo, y_hi, y_lo]` → every byte
shifted by one position:

| Value the program saw | Wrong formula | Consequence |
|---|---|---|
| "x" | `x_hi*256 + y_lo` | x moved but jumped +256 when x>255, contaminated with Y-axis values |
| "y" | `y_hi*256 + size` | **quantized into 4 bands `k*256+~40`** because y_hi∈{0..3} and size (finger pressure) always fell in the 28-43 range |

The "Y jumps in 256px steps" symptom we chased all day was `(y_hi<<8)|size`, not a real coordinate.

**Fix** (`src/touch.rs`): read from `REG_POINT0 = 0x814F` and decode
`x = buf[2]<<8 | buf[1]`, `y = buf[4]<<8 | buf[3]`
(equivalent to Espressif's `esp_lcd_touch_gt911.c`)

### 2. UI buttons not registering (second bug — found after touch was accurate)

The original `wait_touch` used debouncing of "read successfully twice in a
row", but `Touch::read()` **clears the status register (0x814E) every time** →
an immediate second read returns `None` because the chip hasn't sent a new
frame yet → the loop could never return → button presses had no effect.

**Fix** (`src/ui.rs`): removed the debounce — wait for finger release, then
return the first point read.

## Fixes made along the way (necessary but not the root cause)

- **RST pin: GPIO22 → GPIO3** — per the BSP verified on this board
  (`ultramcu/guition-jc4880p4-bsp`); previously we never actually reset the GT911
- Final pin map: SDA=7, SCL=8, RST=3, INT=21 (latch address 0x5D during reset)

## How the root cause was found (lessons)

1. Hex dump of raw bytes every frame → spotted the pattern `y = k*256 + 40` (2-bit Y)
2. Ruled out the software by uploading someone else's test firmware
   (`ultramcu/guition-jc4880p4-bsp` → `DisplayTouchTest`) — touch was smooth
   across the whole display = hardware is fine, so our code was definitely wrong
3. Diffed the register-reading method against the known-working code → found the 0x814F offset
4. Confirmed with our own data: bytes `00 27 02 23` → wrong decode gives (39,547);
   correct decode is y=0x227=551

## Tools remaining in the project

- `ui::paint_test` — draws points following touch in real time on a 100px grid (EXIT = top-right corner)
- `touch.dump_config()` — hex dump of the 186-byte config at boot
- `touch raw` log every frame (should be removed before release)

## Rules / things to remember

- Never read points from 0x8150 — the track id is always at 0x814F
- Never re-read status without waiting for the chip to update (we clear 0x814E after every read)
- Coordinate byte order: the high byte sits at the address after the low byte
  (per-axis little-endian in registers, not a misplaced cross-byte from_be)
