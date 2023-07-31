use crate::frame::{CanOpenFrame, ConvertibleFrame};
use crate::id::CommunicationObject;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SyncFrame;

impl SyncFrame {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SyncFrame {
    fn default() -> Self {
        Self::new()
    }
}

impl From<SyncFrame> for CanOpenFrame {
    fn from(frame: SyncFrame) -> Self {
        CanOpenFrame::SyncFrame(frame)
    }
}

impl ConvertibleFrame for SyncFrame {
    fn communication_object(&self) -> CommunicationObject {
        CommunicationObject::Sync
    }

    fn frame_data(&self) -> std::vec::Vec<u8> {
        std::vec::Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_communication_object() {
        assert_eq!(SyncFrame.communication_object(), CommunicationObject::Sync);
    }

    #[test]
    fn test_set_data() {
        let data = SyncFrame::new().frame_data();
        assert_eq!(data, &[]);
    }
}
