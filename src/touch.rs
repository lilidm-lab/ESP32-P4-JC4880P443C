use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{InputPin, Output, OutputPin, PinDriver};
use esp_idf_svc::hal::i2c::{I2c, I2cConfig, I2cDriver};
use esp_idf_svc::hal::units::Hertz;
use esp_idf_svc::sys::{EspError, TickType_t};

const GT911_ADDR: u8 = 0x5D;
const GT911_ADDR_ALT: u8 = 0x14;
const REG_STATUS: u16 = 0x814E;
// Point data starts at 0x814F (track id), not 0x8150: layout is
// [track, x_lo, x_hi, y_lo, y_hi, size] — reading from 0x8150 shifted
// every byte by one and quantized Y into k*256+size bands.
const REG_POINT0: u16 = 0x814F;
const REG_CONFIG: u16 = 0x8047;
const REG_CONFIG_FRESH: u16 = 0x8100;
const CFG_BODY_LEN: usize = 184;
const CFG_CHECKSUM_IDX: usize = CFG_BODY_LEN;
const CFG_TOTAL_LEN: usize = CFG_BODY_LEN + 2;
const SCREEN_X_MAX: u16 = 480;
const SCREEN_Y_MAX: u16 = 800;
const CFG_APPLY_MS: u32 = 50;
const I2C_TIMEOUT: TickType_t = 50;
const RESET_HOLD_MS: u32 = 10;
const RESET_SETTLE_MS: u32 = 6;
const STARTUP_MS: u32 = 60;

pub struct Touch<'d> {
    i2c: I2cDriver<'d>,
    addr: u8,
    x: Axis,
    y: Axis,
}

struct Axis {
    ref_raw: i32,
    ref_scr: i32,
    num: i32,
    den: i32,
    scr_max: i32,
}

impl Axis {
    fn identity(scr_max: i32) -> Self {
        Axis {
            ref_raw: 0,
            ref_scr: 0,
            num: 1,
            den: 1,
            scr_max,
        }
    }

    fn map(&self, raw: u16) -> u16 {
        let v = (raw as i32 - self.ref_raw) * self.num / self.den + self.ref_scr;
        v.clamp(0, self.scr_max) as u16
    }
}

impl<'d> Touch<'d> {
    pub fn new<I, SDA, SCL, RST, INT>(
        i2c: I,
        sda: SDA,
        scl: SCL,
        rst: RST,
        int: INT,
    ) -> Result<Self, EspError>
    where
        I: I2c + 'd,
        SDA: InputPin + OutputPin + 'd,
        SCL: InputPin + OutputPin + 'd,
        RST: OutputPin + 'd,
        INT: InputPin + OutputPin + 'd,
    {
        // INT low through the reset tail selects the 0x5D address; after
        // disabling the driver the pin floats as an input, ready for IRQ use.
        let mut int_out: PinDriver<Output> = PinDriver::output(int)?;
        int_out.set_low()?;
        let mut rst_drv: PinDriver<Output> = PinDriver::output(rst)?;
        rst_drv.set_low()?;
        FreeRtos::delay_ms(RESET_HOLD_MS);
        rst_drv.set_high()?;
        FreeRtos::delay_ms(RESET_SETTLE_MS);
        drop(int_out);
        FreeRtos::delay_ms(STARTUP_MS);

        let config = I2cConfig::new().baudrate(Hertz(400_000));
        let mut i2c = I2cDriver::new(i2c, sda, scl, &config)?;
        // GT911 answers at 0x5D or 0x14 depending on the INT level at reset.
        let addr = [GT911_ADDR, GT911_ADDR_ALT].into_iter().find(|a| {
            i2c.write_read(*a, &REG_STATUS.to_be_bytes(), &mut [0u8; 1], I2C_TIMEOUT)
                .is_ok()
        });
        let Some(addr) = addr else {
            return Err(EspError::from(esp_idf_svc::sys::ESP_ERR_NOT_FOUND).unwrap());
        };
        log::info!("gt911 at 0x{addr:02X}");
        let mut touch = Touch {
            i2c,
            addr,
            x: Axis::identity(SCREEN_X_MAX as i32 - 1),
            y: Axis::identity(SCREEN_Y_MAX as i32 - 1),
        };
        touch.dump_config()?;
        touch.program_resolution()?;
        Ok(touch)
    }

    /// Rewrite X/Y output max in the GT911 config so the sensor reports
    /// Hex-dump the full 186-byte config for diagnosing sensor mapping.
    fn dump_config(&mut self) -> Result<(), EspError> {
        let mut cfg = [0u8; CFG_TOTAL_LEN];
        self.i2c
            .write_read(self.addr, &REG_CONFIG.to_be_bytes(), &mut cfg, I2C_TIMEOUT)?;
        for (i, row) in cfg.chunks(16).enumerate() {
            let hex: String = row.iter().map(|b| format!("{b:02x} ")).collect();
            log::info!(
                "gt911 cfg@{:04x}: {}",
                0x8047 + (i * 16) as u16,
                hex.trim_end()
            );
        }
        Ok(())
    }

    /// Rewrite X/Y output max in the GT911 config so the sensor reports
    /// coordinates directly in 480x800 screen space, like the vendor driver.
    fn program_resolution(&mut self) -> Result<(), EspError> {
        let mut cfg = [0u8; CFG_TOTAL_LEN];
        self.i2c
            .write_read(self.addr, &REG_CONFIG.to_be_bytes(), &mut cfg, I2C_TIMEOUT)?;
        log::info!(
            "gt911 cfg v{} x_max {} y_max {}",
            cfg[0],
            u16::from_le_bytes([cfg[1], cfg[2]]),
            u16::from_le_bytes([cfg[3], cfg[4]])
        );
        cfg[1..3].copy_from_slice(&SCREEN_X_MAX.to_le_bytes());
        cfg[3..5].copy_from_slice(&SCREEN_Y_MAX.to_le_bytes());
        let sum = cfg[..CFG_BODY_LEN].iter().map(|&b| b as u32).sum::<u32>();
        cfg[CFG_CHECKSUM_IDX] = (!(sum & 0xFF) + 1) as u8;
        let mut pkt = [0u8; CFG_TOTAL_LEN + 2];
        pkt[..2].copy_from_slice(&REG_CONFIG.to_be_bytes());
        pkt[2..].copy_from_slice(&cfg);
        self.i2c.write(self.addr, &pkt, I2C_TIMEOUT)?;
        self.write_reg(REG_CONFIG_FRESH, 1)?;
        FreeRtos::delay_ms(CFG_APPLY_MS);
        let mut check = [0u8; 5];
        self.i2c.write_read(
            self.addr,
            &REG_CONFIG.to_be_bytes(),
            &mut check,
            I2C_TIMEOUT,
        )?;
        log::info!(
            "gt911 cfg now x_max {} y_max {}",
            u16::from_le_bytes([check[1], check[2]]),
            u16::from_le_bytes([check[3], check[4]])
        );
        Ok(())
    }

    /// Returns the first touch point in screen coords (x, y), or None when untouched.
    pub fn read(&mut self) -> Option<(u16, u16)> {
        let status = self.read_reg(REG_STATUS).ok()?;
        if status & 0x80 == 0 {
            return None;
        }
        let points = (status & 0x0F) as usize;
        let point = if points > 0 {
            let mut buf = [0u8; 8];
            let _ =
                self.i2c
                    .write_read(self.addr, &REG_POINT0.to_be_bytes(), &mut buf, I2C_TIMEOUT);
            let raw_x = ((buf[2] as u16) << 8) | buf[1] as u16;
            let raw_y = ((buf[4] as u16) << 8) | buf[3] as u16;
            log::info!(
                "touch raw ({raw_x},{raw_y}) pts {points} bytes {:02x} {:02x} {:02x} {:02x}",
                buf[1],
                buf[2],
                buf[3],
                buf[4]
            );
            Some((self.x.map(raw_x), self.y.map(raw_y)))
        } else {
            None
        };
        let _ = self.write_reg(REG_STATUS, 0x00);
        point
    }

    fn read_reg(&mut self, reg: u16) -> Result<u8, EspError> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(self.addr, &reg.to_be_bytes(), &mut buf, I2C_TIMEOUT)?;
        Ok(buf[0])
    }

    fn write_reg(&mut self, reg: u16, val: u8) -> Result<(), EspError> {
        let mut pkt = [0u8; 3];
        pkt[..2].copy_from_slice(&reg.to_be_bytes());
        pkt[2] = val;
        self.i2c.write(self.addr, &pkt, I2C_TIMEOUT)
    }
}
