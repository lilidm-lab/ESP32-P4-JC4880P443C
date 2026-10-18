pub const FRAME_WIDTH: usize = 640;
pub const FRAME_HEIGHT: usize = 480;

const FRAME_LEN: usize = FRAME_WIDTH * FRAME_HEIGHT * 2;

#[derive(Debug)]
pub struct Camera {
    _private: (),
}

#[derive(Debug)]
pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub data: std::vec::Vec<u8>,
}

impl Frame {
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

pub fn init() -> anyhow::Result<Camera> {
    unimplemented!("camera needs esp_cam / esp_video managed components (MIPI-CSI on P4)")
}

impl Camera {
    pub fn capture(&mut self) -> anyhow::Result<Frame> {
        let _ = FRAME_LEN;
        todo!()
    }
}
