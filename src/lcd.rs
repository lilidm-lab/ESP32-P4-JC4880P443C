use embedded_graphics::pixelcolor::{IntoStorage, Rgb565};
use embedded_graphics::prelude::{DrawTarget, OriginDimensions, Size};
use embedded_graphics::Pixel;

use esp_idf_svc::sys;
use esp_idf_svc::sys::EspError;

const DSI_BUS_ID: i32 = 0;
const DSI_DATA_LANES: u8 = 2;
const DSI_LANE_MBPS: u32 = 500;
const VIRTUAL_CHANNEL: u8 = 0;
const CMD_BITS: i32 = 8;
const PARAM_BITS: i32 = 8;

const H_RES: u16 = 480;
const V_RES: u16 = 800;
const DPI_CLOCK_MHZ: u32 = 34;
const NUM_FBS: u8 = 1;
const HS_PULSE: u32 = 12;
const HS_BACK_PORCH: u32 = 42;
const HS_FRONT_PORCH: u32 = 42;
const VS_PULSE: u32 = 2;
const VS_BACK_PORCH: u32 = 8;
const VS_FRONT_PORCH: u32 = 166;

const PHY_LDO_CHAN: i32 = 3;
const PHY_LDO_MV: i32 = 2500;
const PANEL_RESET_PIN: i32 = 5;
const BACKLIGHT_PIN: i32 = 23;

pub const COLOR_BG: u16 = 0x0000;

/// Guition's sequence for this panel, from the vendored esp_lcd_st7701 in
/// the JC4880P443C_I_W archive. (cmd, data, delay_ms after)
const INIT_CMDS: &[(u8, &[u8], u32)] = &[
    (0xFF, &[0x77, 0x01, 0x00, 0x00, 0x13], 0),
    (0xEF, &[0x08], 0),
    (0xFF, &[0x77, 0x01, 0x00, 0x00, 0x10], 0),
    (0xC0, &[0x63, 0x00], 0),
    (0xC1, &[0x0D, 0x02], 0),
    (0xC2, &[0x10, 0x08], 0),
    (0xCC, &[0x10], 0),
    (
        0xB0,
        &[
            0x80, 0x09, 0x53, 0x0C, 0xD0, 0x07, 0x0C, 0x09, 0x09, 0x28, 0x06, 0xD4, 0x13, 0x69,
            0x2B, 0x71,
        ],
        0,
    ),
    (
        0xB1,
        &[
            0x80, 0x94, 0x5A, 0x10, 0xD3, 0x06, 0x0A, 0x08, 0x08, 0x25, 0x03, 0xD3, 0x12, 0x66,
            0x6A, 0x0D,
        ],
        0,
    ),
    (0xFF, &[0x77, 0x01, 0x00, 0x00, 0x11], 0),
    (0xB0, &[0x5D], 0),
    (0xB1, &[0x58], 0),
    (0xB2, &[0x87], 0),
    (0xB3, &[0x80], 0),
    (0xB5, &[0x4E], 0),
    (0xB7, &[0x85], 0),
    (0xB8, &[0x21], 0),
    (0xB9, &[0x10, 0x1F], 0),
    (0xBB, &[0x03], 0),
    (0xBC, &[0x00], 0),
    (0xC1, &[0x78], 0),
    (0xC2, &[0x78], 0),
    (0xD0, &[0x88], 0),
    (0xE0, &[0x00, 0x3A, 0x02], 0),
    (
        0xE1,
        &[
            0x04, 0xA0, 0x00, 0xA0, 0x05, 0xA0, 0x00, 0xA0, 0x00, 0x40, 0x40,
        ],
        0,
    ),
    (
        0xE2,
        &[
            0x30, 0x00, 0x40, 0x40, 0x32, 0xA0, 0x00, 0xA0, 0x00, 0xA0, 0x00, 0xA0, 0x00,
        ],
        0,
    ),
    (0xE3, &[0x00, 0x00, 0x33, 0x33], 0),
    (0xE4, &[0x44, 0x44], 0),
    (
        0xE5,
        &[
            0x09, 0x2E, 0xA0, 0xA0, 0x0B, 0x30, 0xA0, 0xA0, 0x05, 0x2A, 0xA0, 0xA0, 0x07, 0x2C,
            0xA0, 0xA0,
        ],
        0,
    ),
    (0xE6, &[0x00, 0x00, 0x33, 0x33], 0),
    (0xE7, &[0x44, 0x44], 0),
    (
        0xE8,
        &[
            0x08, 0x2D, 0xA0, 0xA0, 0x0A, 0x2F, 0xA0, 0xA0, 0x04, 0x29, 0xA0, 0xA0, 0x06, 0x2B,
            0xA0, 0xA0,
        ],
        0,
    ),
    (0xEB, &[0x00, 0x00, 0x4E, 0x4E, 0x00, 0x00, 0x00], 0),
    (0xEC, &[0x08, 0x01], 0),
    (
        0xED,
        &[
            0xB0, 0x2B, 0x98, 0xA4, 0x56, 0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xF7, 0x65, 0x4A, 0x89,
            0xB2, 0x0B,
        ],
        0,
    ),
    (0xEF, &[0x08, 0x08, 0x08, 0x45, 0x3F, 0x54], 0),
    (0xFF, &[0x77, 0x01, 0x00, 0x00, 0x00], 0),
    (0x11, &[0x00], 120),
    (0x29, &[0x00], 20),
];

pub struct Lcd {
    panel: sys::esp_lcd_panel_handle_t,
}

pub fn init() -> Result<Lcd, EspError> {
    unsafe { Lcd::new() }
}

impl Lcd {
    unsafe fn new() -> Result<Self, EspError> {
        Self::phy_ldo_power_up()?;
        Self::panel_gpio_reset();

        let bus_config = sys::esp_lcd_dsi_bus_config_t {
            bus_id: DSI_BUS_ID,
            num_data_lanes: DSI_DATA_LANES,
            // 0 = let driver pick legacy PLL_F20M source (required on P4 rev <3.0)
            phy_clk_src: 0,
            lane_bit_rate_mbps: DSI_LANE_MBPS as f32,
            ..Default::default()
        };
        let mut bus: sys::esp_lcd_dsi_bus_handle_t = std::ptr::null_mut();
        sys::esp_lcd_new_dsi_bus(&bus_config, &mut bus).esp()?;
        let bus = bus;

        let io_config = sys::esp_lcd_dbi_io_config_t {
            virtual_channel: VIRTUAL_CHANNEL,
            lcd_cmd_bits: CMD_BITS,
            lcd_param_bits: PARAM_BITS,
        };
        let mut io: sys::esp_lcd_panel_io_handle_t = std::ptr::null_mut();
        sys::esp_lcd_new_panel_io_dbi(bus, &io_config, &mut io).esp()?;
        let io = io;

        Self::vendor_init(io)?;

        let dpi_config = sys::esp_lcd_dpi_panel_config_t {
            virtual_channel: VIRTUAL_CHANNEL,
            dpi_clk_src: sys::soc_periph_mipi_dsi_dpi_clk_src_t_MIPI_DSI_DPI_CLK_SRC_DEFAULT,
            dpi_clock_freq_mhz: DPI_CLOCK_MHZ as f32,
            in_color_format: sys::lcd_color_format_t_LCD_COLOR_FMT_RGB565,
            out_color_format: sys::lcd_color_format_t_LCD_COLOR_FMT_RGB565,
            num_fbs: NUM_FBS,
            video_timing: sys::esp_lcd_video_timing_t {
                h_size: H_RES as u32,
                v_size: V_RES as u32,
                hsync_pulse_width: HS_PULSE,
                hsync_back_porch: HS_BACK_PORCH,
                hsync_front_porch: HS_FRONT_PORCH,
                vsync_pulse_width: VS_PULSE,
                vsync_back_porch: VS_BACK_PORCH,
                vsync_front_porch: VS_FRONT_PORCH,
            },
            ..Default::default()
        };
        let mut panel: sys::esp_lcd_panel_handle_t = std::ptr::null_mut();
        sys::esp_lcd_new_panel_dpi(bus, &dpi_config, &mut panel).esp()?;
        let panel = panel;

        sys::esp_lcd_panel_init(panel).esp()?;
        log::info!("dsi: panel up");

        Ok(Lcd { panel })
    }

    unsafe fn phy_ldo_power_up() -> Result<(), EspError> {
        let config = sys::esp_ldo_channel_config_t {
            chan_id: PHY_LDO_CHAN,
            voltage_mv: PHY_LDO_MV,
            flags: Default::default(),
        };
        let mut ldo: sys::esp_ldo_channel_handle_t = std::ptr::null_mut();
        sys::esp_ldo_acquire_channel(&config, &mut ldo).esp()
    }

    unsafe fn panel_gpio_reset() {
        let rst = PANEL_RESET_PIN;
        sys::gpio_reset_pin(rst);
        sys::gpio_set_direction(rst, sys::gpio_mode_t_GPIO_MODE_OUTPUT);
        sys::gpio_set_level(rst, 1);
        sys::vTaskDelay(pd_ms(10));
        sys::gpio_set_level(rst, 0);
        sys::vTaskDelay(pd_ms(10));
        sys::gpio_set_level(rst, 1);
        sys::vTaskDelay(pd_ms(120));
    }

    unsafe fn vendor_init(io: sys::esp_lcd_panel_io_handle_t) -> Result<(), EspError> {
        for (cmd, data, delay_ms) in INIT_CMDS {
            sys::esp_lcd_panel_io_tx_param(io, *cmd as i32, data.as_ptr() as *const _, data.len())
                .esp()?;
            if *delay_ms > 0 {
                sys::vTaskDelay(pd_ms(*delay_ms));
            }
        }
        Ok(())
    }

    pub fn backlight_on(&self) -> Result<(), EspError> {
        let pin = BACKLIGHT_PIN;
        unsafe {
            sys::gpio_reset_pin(pin);
            sys::gpio_set_direction(pin, sys::gpio_mode_t_GPIO_MODE_OUTPUT);
            sys::gpio_set_level(pin, 1);
        }
        Ok(())
    }

    pub fn blit(&self, x: u16, y: u16, w: u16, h: u16, buf: &[u16]) -> Result<(), EspError> {
        debug_assert_eq!(buf.len(), (w as usize) * (h as usize));
        unsafe {
            sys::esp_lcd_panel_draw_bitmap(
                self.panel,
                x as i32,
                y as i32,
                (x + w) as i32,
                (y + h) as i32,
                buf.as_ptr() as *const _,
            )
            .esp()?;
        }
        Ok(())
    }
}

pub struct Screen {
    buf: Vec<u16>,
}

impl Screen {
    pub fn new() -> Self {
        Screen {
            buf: vec![COLOR_BG; (H_RES as usize) * (V_RES as usize)],
        }
    }

    pub fn flush(&self, lcd: &Lcd) -> Result<(), EspError> {
        lcd.blit(0, 0, H_RES, V_RES, &self.buf)
    }
}

impl Default for Screen {
    fn default() -> Self {
        Self::new()
    }
}

impl OriginDimensions for Screen {
    fn size(&self) -> Size {
        Size::new(H_RES as u32, V_RES as u32)
    }
}

impl DrawTarget for Screen {
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels {
            let (x, y) = (coord.x as usize, coord.y as usize);
            if x < H_RES as usize && y < V_RES as usize {
                self.buf[y * H_RES as usize + x] = color.into_storage();
            }
        }
        Ok(())
    }
}

fn pd_ms(ms: u32) -> u32 {
    ms * sys::configTICK_RATE_HZ / 1000
}

trait EspCheck {
    fn esp(self) -> Result<(), EspError>;
}

impl EspCheck for i32 {
    fn esp(self) -> Result<(), EspError> {
        EspError::from(self).map_or(Ok(()), Err)
    }
}
