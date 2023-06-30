use crate::frame::{CanOpenFrame, ToSocketCanFrame};
use crate::id::CommunicationObject;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SyncFrame;

impl SyncFrame {
    const FRAME_DATA_SIZE: usize = 0;

    pub fn new() -> Self {
        Self
    }
}

impl From<SyncFrame> for CanOpenFrame {
    fn from(frame: SyncFrame) -> Self {
        CanOpenFrame::SyncFrame(frame)
    }
}

impl ToSocketCanFrame for SyncFrame {
    fn communication_object(&self) -> CommunicationObject {
        CommunicationObject::Sync
    }

    fn set_data(&self, _data: &mut [u8]) -> usize {
        Self::FRAME_DATA_SIZE
    }
}

#[cfg(test)]
mod tests {
    use socketcan::{EmbeddedFrame, Frame};

    use super::*;

    #[test]
    fn test_communication_object() {
        assert_eq!(SyncFrame.communication_object(), CommunicationObject::Sync);
    }

    #[test]
    fn test_set_data() {
        let mut buf = [0u8; 8];

        let frame_data_size = SyncFrame::new().set_data(&mut buf);
        assert_eq!(frame_data_size, 0);
    }

    #[test]
    fn test_sync_frame_to_socketcan_frame() {
        let frame = SyncFrame::new().to_socketcan_frame();
        assert_eq!(frame.raw_id(), 0x080);
        assert_eq!(frame.data(), &[]);
    }
}
