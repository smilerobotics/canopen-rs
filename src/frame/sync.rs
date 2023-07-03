use crate::frame::{CanOpenFrame, ConvertibleFrame};
use crate::id::CommunicationObject;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SyncFrame;

impl SyncFrame {
    pub fn new() -> Self {
        Self
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

    fn set_data<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        &buf[..0]
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
        let mut buf = [0u8; 8];

        let data = SyncFrame::new().set_data(&mut buf);
        assert_eq!(data, &[]);
    }
}
