# GT911 Touch — JC4880P443C_I_W (ESP32-P4)

สรุปการดีบัก: 2026-10-16 — **SOLVED** (touch ตรงทั้งจอ, UI ใช้ได้ครบ)

## ทำไม touch ถึงไม่ตรง — สาเหตุหลัก

### 1. อ่าน point register ผิด offset (root cause ของ "touch เพี้ยน")

โค้ดเดิมอ่านพิกัดจาก register **0x8150** แต่ GT911 เก็บ point data เริ่มที่
**0x814F** โดย layout ต่อ 1 จุดคือ:

```
register:  814F    8150   8151   8152   8153   8154
ค่า:       track   x_lo   x_hi   y_lo   y_hi   size
```

เราอ่านจาก 0x8150 แล้วถอดรหัสแบบ `[x_hi, x_lo, y_hi, y_lo]` → ทุก byte
เลื่อนไป 1 ตำแหน่ง:

| ค่าที่โปรแกรมเห็น | สูตรผิด | ผลที่ตามมา |
|---|---|---|
| "x" | `x_hi*256 + y_lo` | x เดินได้แต่ jump +256 เมื่อ x>255, ปนค่าแกน Y |
| "y" | `y_hi*256 + size` | **quantize เป็น 4 แถบ `k*256+~40`** เพราะ y_hi∈{0..3} และ size (ความแรงนิ้ว) ติดช่วง 28-43 เสมอ |

อาการ "Y เด้งทีละ 256px" ที่ไล่กันมาทั้งวันคือ `(y_hi<<8)|size` ไม่ใช่พิกัดจริง

**Fix** (`src/touch.rs`): อ่านจาก `REG_POINT0 = 0x814F` และ decode
`x = buf[2]<<8 | buf[1]`, `y = buf[4]<<8 | buf[3]`
(เทียบเท่ากับ `esp_lcd_touch_gt911.c` ของ Espressif)

### 2. ปุ่ม UI กดไม่ติด (bug ตัวที่สอง — พบหลังจาก touch ตรงแล้ว)

`wait_touch` เดิมใช้ debounce "อ่านได้ 2 ครั้งติดกัน" แต่ `Touch::read()`
**เคลียร์ status register (0x814E) ทุกครั้ง** → การอ่านครั้งที่สองทันทีจะได้
`None` เพราะ chip ยังไม่ส่ง frame ใหม่ → loop ไม่มีวัน return → กดปุ่มไม่มีผล

**Fix** (`src/ui.rs`): ถอด debounce — รอปล่อยนิ้ว แล้ว return จุดแรกที่อ่านได้

## สิ่งที่แก้ระหว่างทาง (จำเป็นแต่ไม่ใช่ root cause)

- **RST pin: GPIO22 → GPIO3** — ตาม BSP ที่ verified บนบอร์ดนี้
  (`ultramcu/guition-jc4880p4-bsp`) เดิมเราไม่เคย reset GT911 จริงเลย
- Pin map สุดท้าย: SDA=7, SCL=8, RST=3, INT=21 (ใช้ latch address 0x5D ตอน reset)

## วิธีหา root cause (บทเรียน)

1. hex dump ค่า byte ดิบทุกเฟรม → เห็น pattern `y = k*256 + 40` (2-bit Y)
2. ตัด software ออกด้วยการอัปโหลด firmware ทดสอบของคนอื่น
   (`ultramcu/guition-jc4880p4-bsp` → `DisplayTouchTest`) — touch ลื่นเต็มจอ
   = ฮาร์ดแวร์ปกติ แน่นอนว่าโค้ดเราผิด
3. diff วิธีอ่าน register กับโค้ดที่ทำงานจริง → เจอ offset 0x814F
4. ยืนยันด้วยข้อมูลเราเอง: bytes `00 27 02 23` → decode ผิดได้ (39,547)
   decode ถูกคือ y=0x227=551

## เครื่องมือที่เหลืออยู่ในโปรเจกต์

- `ui::paint_test` — วาดจุดตาม touch แบบ real-time บน grid 100px (EXIT = มุมขวาบน)
- `touch.dump_config()` — hex dump config 186 bytes ตอน boot
- log `touch raw` ทุกเฟรม (ควรตัดก่อนปล่อยรุ่น)

## ข้อห้าม / ข้อควรจำ

- อย่าอ่าน point จาก 0x8150 — track id อยู่ที่ 0x814F เสมอ
- อย่าอ่านสถานะซ้ำโดยไม่รอ chip update (เรา clear 0x814E หลังอ่านทุกครั้ง)
- byte order ของพิกัด: high byte อยู่ที่ address ถัดจาก low byte
  (per-axis little-endian ใน register, ไม่ใช่ from_be ข้าม byte ที่ผิดตำแหน่ง)
