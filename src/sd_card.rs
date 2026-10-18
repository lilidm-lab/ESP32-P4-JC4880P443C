pub const BLOCK_SIZE: u64 = 512;
pub const MOUNT_POINT: &str = "/sdcard";

#[derive(Debug)]
pub enum SdError {
    Esp(esp_idf_svc::sys::EspError),
}

impl std::fmt::Display for SdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SdError::Esp(e) => write!(f, "sd card esp error: {e}"),
        }
    }
}

#[derive(Debug)]
pub struct SdCard {
    _private: (),
}

pub fn mount() -> Result<SdCard, SdError> {
    unimplemented!("sd needs esp-idf-hal sdmmc or SD-SPI driver wiring (SDMMC1 on P4)")
}

impl SdCard {
    pub fn capacity_bytes(&self) -> u64 {
        let _ = BLOCK_SIZE;
        todo!()
    }

    pub fn unmount(self) -> Result<(), SdError> {
        todo!()
    }
}
