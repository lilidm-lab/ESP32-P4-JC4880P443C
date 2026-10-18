use core::time::Duration;
use std::fmt;
use std::net::Ipv4Addr;

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::modem::WifiModemPeripheral;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs};
use esp_idf_svc::wifi::{
    AccessPointInfo, AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi,
};

fn sleep(d: core::time::Duration) {
    FreeRtos::delay_ms(d.as_millis() as u32);
}

pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

const NVS_NAMESPACE: &str = "wifi";
const NVS_KEY_SSID: &str = "ssid";
const NVS_KEY_PASS: &str = "pass";

pub type Sta = BlockingWifi<EspWifi<'static>>;

#[derive(Debug)]
pub enum WifiError {
    Esp(esp_idf_svc::sys::EspError),
    Timeout,
}

impl fmt::Display for WifiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WifiError::Esp(e) => write!(f, "wifi esp error: {e}"),
            WifiError::Timeout => write!(f, "connect timeout / wrong password"),
        }
    }
}

impl std::error::Error for WifiError {}

pub fn open_with(
    modem: impl WifiModemPeripheral + 'static,
    sysloop: EspSystemEventLoop,
) -> Result<Sta, WifiError> {
    let wifi = EspWifi::new(modem, sysloop.clone(), None).map_err(WifiError::Esp)?;
    BlockingWifi::wrap(wifi, sysloop).map_err(WifiError::Esp)
}

pub fn client_config(ssid: &str, pass: &str) -> Result<ClientConfiguration, WifiError> {
    let auth = if pass.is_empty() {
        AuthMethod::None
    } else {
        AuthMethod::WPA2Personal
    };
    Ok(ClientConfiguration {
        ssid: ssid.try_into().map_err(|_| WifiError::Timeout)?,
        password: pass.try_into().map_err(|_| WifiError::Timeout)?,
        auth_method: auth,
        ..Default::default()
    })
}

const SCAN_RETRIES: u32 = 5;
const RETRY_DELAY: core::time::Duration = core::time::Duration::from_millis(1500);

pub fn scan(sta: &mut Sta) -> Result<std::vec::Vec<AccessPointInfo>, WifiError> {
    if !sta.is_started().map_err(WifiError::Esp)? {
        sta.start().map_err(WifiError::Esp)?;
    }
    // C6 slave auto-connects from its own stored credentials; while it does,
    // scan results are dropped over RPC. Disconnect first, retry until clean.
    let _ = sta.disconnect();
    for _ in 0..SCAN_RETRIES {
        sleep(RETRY_DELAY);
        match sta.scan() {
            Ok(aps) if !aps.is_empty() => return Ok(aps),
            Ok(aps) => {
                log::info!("scan: {} aps, retrying", aps.len());
            }
            Err(e) => log::info!("scan err {e}, retrying"),
        }
    }
    sta.scan().map_err(WifiError::Esp)
}

pub fn join(sta: &mut Sta, ssid: &str, pass: &str) -> Result<Ipv4Addr, WifiError> {
    sta.set_configuration(&Configuration::Client(client_config(ssid, pass)?))
        .map_err(WifiError::Esp)?;
    sta.connect().map_err(WifiError::Esp)?;
    let res = sta.ip_wait_while(|| sta.is_up().map(|up| !up), Some(CONNECT_TIMEOUT));
    match res {
        Ok(()) => {
            let info = sta
                .wifi()
                .sta_netif()
                .get_ip_info()
                .map_err(WifiError::Esp)?;
            Ok(info.ip)
        }
        Err(_) => {
            let _ = sta.disconnect();
            Err(WifiError::Timeout)
        }
    }
}

pub fn load_creds(nvs: &EspDefaultNvsPartition) -> Option<(String, String)> {
    let store = EspNvs::new(nvs.clone(), NVS_NAMESPACE, false).ok()?;
    let mut ssid_buf = [0u8; 32];
    let mut pass_buf = [0u8; 64];
    let ssid = store.get_str(NVS_KEY_SSID, &mut ssid_buf).ok()??;
    let pass = store.get_str(NVS_KEY_PASS, &mut pass_buf).ok()??;
    Some((ssid.into(), pass.into()))
}

pub fn store_creds(
    nvs: &EspDefaultNvsPartition,
    ssid: &str,
    pass: &str,
) -> Result<(), esp_idf_svc::sys::EspError> {
    let store = EspNvs::new(nvs.clone(), NVS_NAMESPACE, true)?;
    store.set_str(NVS_KEY_SSID, ssid)?;
    store.set_str(NVS_KEY_PASS, pass)
}
