pub const TX_BUF_LEN: usize = 1024;
pub const RX_BUF_LEN: usize = 1024;

#[derive(Debug)]
pub struct DmaError {
    _private: (),
}

pub fn tx_buf(len: usize) -> Result<(), DmaError> {
    let _ = (len, TX_BUF_LEN);
    unimplemented!("gdma bufs need esp-idf-hal >= 0.48 (esp-dma) or raw esp_idf_svc::sys bindings")
}

pub fn rx_buf(len: usize) -> Result<(), DmaError> {
    let _ = (len, RX_BUF_LEN);
    unimplemented!("gdma bufs need esp-idf-hal >= 0.48 (esp-dma) or raw esp_idf_svc::sys bindings")
}
