use core::time::Duration;

pub const DEVICE_NAME: &str = "ESP32-P4-JC4880P443C";
pub const ADV_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug)]
pub enum BleError {
    Esp(esp_idf_svc::sys::EspError),
    Timeout,
}

impl std::fmt::Display for BleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BleError::Esp(e) => write!(f, "ble esp error: {e}"),
            BleError::Timeout => write!(f, "ble timeout"),
        }
    }
}

pub fn init() -> Result<(), BleError> {
    unimplemented!("ble needs NimBLE enabled in sdkconfig + controller (C6 via esp-hosted)")
}
