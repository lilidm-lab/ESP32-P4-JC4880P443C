//! Standalone touch test: draws a dot wherever the finger reports, over a
//! 100px grid. Flash with `make example NAME=touch_paint`.

use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys::link_patches;

use esp32_p4_jc4880p443c::lcd;
use esp32_p4_jc4880p443c::touch::Touch;
use esp32_p4_jc4880p443c::ui;

fn main() -> anyhow::Result<()> {
    link_patches();
    EspLogger::initialize_default();

    let display = lcd::init()?;
    display.backlight_on()?;

    let _peripherals = Peripherals::take()?;
    let mut touch = unsafe {
        Touch::new(
            esp_idf_svc::hal::i2c::I2C0::steal(),
            esp_idf_svc::hal::gpio::Gpio7::steal(),
            esp_idf_svc::hal::gpio::Gpio8::steal(),
            esp_idf_svc::hal::gpio::Gpio3::steal(),
            esp_idf_svc::hal::gpio::Gpio21::steal(),
        )
    }?;

    let mut screen = lcd::Screen::new();
    loop {
        ui::paint_test(&mut screen, &display, &mut touch);
    }
}
