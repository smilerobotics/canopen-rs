use crate::error::{Error, Result};
use crate::frame::{CanOpenFrame, ToSocketCanFrame};
use crate::id::{CommunicationObject, NodeId};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmergencyFrame {
    pub node_id: NodeId,
    pub error_code: u16,
    pub error_register: u8,
}

impl EmergencyFrame {
    const FRAME_DATA_SIZE: usize = 8;

    pub fn new(node_id: NodeId, error_code: u16, error_register: u8) -> Self {
        Self {
            node_id,
            error_code,
            error_register,
        }
    }

    pub(super) fn from_node_id_bytes(node_id: NodeId, bytes: &[u8]) -> Result<Self> {
        if bytes.len() != Self::FRAME_DATA_SIZE {
            return Err(Error::InvalidDataLength {
                length: bytes.len(),
                data_type: "EmergencyFrame".to_owned(),
            });
        }
        Ok(Self::new(
            node_id,
            u16::from_le_bytes(bytes[0..2].try_into().unwrap()),
            bytes[2],
        ))
    }
}

impl From<EmergencyFrame> for CanOpenFrame {
    fn from(frame: EmergencyFrame) -> Self {
        CanOpenFrame::EmergencyFrame(frame)
    }
}

impl ToSocketCanFrame for EmergencyFrame {
    fn communication_object(&self) -> CommunicationObject {
        CommunicationObject::Emergency(self.node_id)
    }

    fn set_data(&self, data: &mut [u8]) -> usize {
        assert!(data.len() >= Self::FRAME_DATA_SIZE);
        data[0..2].copy_from_slice(&self.error_code.to_le_bytes());
        data[2] = self.error_register;
        data[3..].fill(0x00);

        Self::FRAME_DATA_SIZE
    }
}

#[cfg(test)]
mod tests {
    use socketcan::{EmbeddedFrame, Frame};

    use super::*;

    #[test]
    fn test_from_node_id_bytes() {
        assert_eq!(
            EmergencyFrame::from_node_id_bytes(
                1.try_into().unwrap(),
                &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
            ),
            Ok(EmergencyFrame {
                node_id: 1.try_into().unwrap(),
                error_code: 0x0000,
                error_register: 0x00
            })
        );
        assert_eq!(
            EmergencyFrame::from_node_id_bytes(
                2.try_into().unwrap(),
                &[0x00, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00]
            ),
            Ok(EmergencyFrame {
                node_id: 2.try_into().unwrap(),
                error_code: 0x1000,
                error_register: 0x01
            })
        );
        assert_eq!(
            EmergencyFrame::from_node_id_bytes(
                127.try_into().unwrap(),
                &[0x34, 0x12, 0x56, 0x00, 0x00, 0x00, 0x00, 0x00]
            ),
            Ok(EmergencyFrame {
                node_id: 127.try_into().unwrap(),
                error_code: 0x1234,
                error_register: 0x56
            })
        );
        assert!(
            EmergencyFrame::from_node_id_bytes(1.try_into().unwrap(), &[0x00, 0x00, 0x00]).is_err()
        );
    }

    #[test]
    fn test_communication_object() {
        assert_eq!(
            EmergencyFrame::new(1.try_into().unwrap(), 0x0000, 0x00).communication_object(),
            CommunicationObject::Emergency(1.try_into().unwrap())
        );
        assert_eq!(
            EmergencyFrame::new(2.try_into().unwrap(), 0x1000, 0x01).communication_object(),
            CommunicationObject::Emergency(2.try_into().unwrap())
        );
        assert_eq!(
            EmergencyFrame::new(127.try_into().unwrap(), 0x1234, 0x56).communication_object(),
            CommunicationObject::Emergency(127.try_into().unwrap())
        );
    }

    #[test]
    fn test_set_data() {
        let mut buf = [0u8; 8];

        let frame_data_size =
            EmergencyFrame::new(1.try_into().unwrap(), 0x0000, 0x00).set_data(&mut buf);
        assert_eq!(frame_data_size, 8);
        assert_eq!(&buf[..], &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

        let frame_data_size =
            EmergencyFrame::new(2.try_into().unwrap(), 0x1000, 0x01).set_data(&mut buf);
        assert_eq!(frame_data_size, 8);
        assert_eq!(&buf[..], &[0x00, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00]);

        let frame_data_size =
            EmergencyFrame::new(127.try_into().unwrap(), 0x1234, 0x56).set_data(&mut buf);
        assert_eq!(frame_data_size, 8);
        assert_eq!(&buf[..], &[0x34, 0x12, 0x56, 0x00, 0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_nmt_node_monitoring_frame_to_socketcan_frame() {
        let frame = EmergencyFrame::new(1.try_into().unwrap(), 0x0000, 0x00).to_socketcan_frame();
        assert_eq!(frame.raw_id(), 0x081);
        assert_eq!(
            frame.data(),
            &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        );

        let frame = EmergencyFrame::new(2.try_into().unwrap(), 0x1000, 0x01).to_socketcan_frame();
        assert_eq!(frame.raw_id(), 0x082);
        assert_eq!(
            frame.data(),
            &[0x00, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00]
        );

        let frame = EmergencyFrame::new(127.try_into().unwrap(), 0x1234, 0x56).to_socketcan_frame();
        assert_eq!(frame.raw_id(), 0x0FF);
        assert_eq!(
            frame.data(),
            &[0x34, 0x12, 0x56, 0x00, 0x00, 0x00, 0x00, 0x00]
        );
    }
}
