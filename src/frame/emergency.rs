use crate::error::{Error, Result};
use crate::frame::{CanOpenFrame, ConvertibleFrame};
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

    pub(crate) fn new_with_bytes(node_id: NodeId, bytes: &[u8]) -> Result<Self> {
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

impl ConvertibleFrame for EmergencyFrame {
    fn communication_object(&self) -> CommunicationObject {
        CommunicationObject::Emergency(self.node_id)
    }

    fn frame_data(&self) -> std::vec::Vec<u8> {
        let mut data = std::vec::Vec::with_capacity(Self::FRAME_DATA_SIZE);
        data.extend_from_slice(&self.error_code.to_le_bytes());
        data.push(self.error_register);
        data.resize(Self::FRAME_DATA_SIZE, 0x00);
        assert_eq!(data.len(), Self::FRAME_DATA_SIZE);
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_node_id_bytes() {
        let result = EmergencyFrame::new_with_bytes(
            1.try_into().unwrap(),
            &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        );
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            EmergencyFrame {
                node_id: 1.try_into().unwrap(),
                error_code: 0x0000,
                error_register: 0x00
            }
        );

        let result = EmergencyFrame::new_with_bytes(
            2.try_into().unwrap(),
            &[0x00, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00],
        );
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            EmergencyFrame {
                node_id: 2.try_into().unwrap(),
                error_code: 0x1000,
                error_register: 0x01
            }
        );

        let result = EmergencyFrame::new_with_bytes(
            127.try_into().unwrap(),
            &[0x34, 0x12, 0x56, 0x00, 0x00, 0x00, 0x00, 0x00],
        );
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            EmergencyFrame {
                node_id: 127.try_into().unwrap(),
                error_code: 0x1234,
                error_register: 0x56
            }
        );

        let result = EmergencyFrame::new_with_bytes(1.try_into().unwrap(), &[0x00, 0x00, 0x00]);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::InvalidDataLength {
                length: _,
                data_type: _,
            } => (),
            _ => panic!("Error kind mismatch"),
        }
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
    fn test_data() {
        let mut buf = [0u8; 8];

        let data = EmergencyFrame::new(1.try_into().unwrap(), 0x0000, 0x00).frame_data();
        assert_eq!(data.len(), 8);
        assert_eq!(data, &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

        buf.fill(0x00);
        let data = EmergencyFrame::new(2.try_into().unwrap(), 0x1000, 0x01).frame_data();
        assert_eq!(data.len(), 8);
        assert_eq!(data, &[0x00, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00]);

        buf.fill(0x00);
        let data = EmergencyFrame::new(127.try_into().unwrap(), 0x1234, 0x56).frame_data();
        assert_eq!(data.len(), 8);
        assert_eq!(data, &[0x34, 0x12, 0x56, 0x00, 0x00, 0x00, 0x00, 0x00]);
    }
}
