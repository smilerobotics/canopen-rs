use libc::CAN_MAX_DLEN;
use socketcan::EmbeddedFrame;

use crate::id::{CommunicationObject, NodeID};
use crate::{Error, Result};

trait ToSocketCANFrame {
    fn communication_object(&self) -> CommunicationObject;
    fn set_data(&self, data: &mut [u8]) -> usize;

    fn to_socketcan_frame(&self) -> socketcan::CanFrame {
        let mut data = [0u8; CAN_MAX_DLEN];
        let data_size = self.set_data(&mut data);
        socketcan::CanFrame::new(self.communication_object(), &data[0..data_size]).unwrap()
        // The `new` method could return `None` if the data length is too long.
        // But its length is same as the limit.
    }
}

mod nmt_node_control;
pub use nmt_node_control::{NMTCommand, NMTNodeControlAddress, NMTNodeControlFrame};

mod sdo;
pub use sdo::SDOFrame;

#[derive(Debug, PartialEq)]
pub enum CANOpenFrame {
    NMTNodeControlFrame(NMTNodeControlFrame),
    SDOFrame(SDOFrame),
}

impl CANOpenFrame {
    pub fn new_nmt_node_control_frame(command: NMTCommand, address: NMTNodeControlAddress) -> Self {
        Self::NMTNodeControlFrame(NMTNodeControlFrame::new(command, address))
    }

    pub fn new_sdo_read_frame(node_id: NodeID, index: u16, sub_index: u8) -> Self {
        Self::SDOFrame(SDOFrame::new_sdo_read_frame(node_id, index, sub_index))
    }

    pub fn new_sdo_write_frame(node_id: NodeID, index: u16, sub_index: u8, data: &[u8]) -> Self {
        Self::SDOFrame(SDOFrame::new_sdo_write_frame(
            node_id, index, sub_index, data,
        ))
    }
}

impl From<CANOpenFrame> for socketcan::CanFrame {
    fn from(frame: CANOpenFrame) -> Self {
        match frame {
            CANOpenFrame::NMTNodeControlFrame(frame) => frame.to_socketcan_frame(),
            CANOpenFrame::SDOFrame(frame) => frame.to_socketcan_frame(),
        }
    }
}

impl TryFrom<socketcan::CanFrame> for CANOpenFrame {
    type Error = Error;
    fn try_from(frame: socketcan::CanFrame) -> Result<Self> {
        match frame {
            socketcan::CanFrame::Data(frame) => {
                let cob: CommunicationObject = frame.id().try_into()?;
                match cob {
                    CommunicationObject::NMTNodeControl => {
                        Ok(NMTNodeControlFrame::from_bytes(frame.data())?.into())
                    }
                    _ => Err(Error::NotImplemented),
                }
            }
            socketcan::CanFrame::Remote(_) => Err(Error::NotImplemented),
            socketcan::CanFrame::Error(_) => Err(Error::NotImplemented),
        }
    }
}

#[cfg(test)]
mod tests {
    use socketcan::EmbeddedFrame;

    use super::*;

    #[test]
    fn test_socketcan_frame_to_nmt_node_control_frame() {
        let frame: Result<CANOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x000).unwrap(), &[0x01, 0x00])
                .unwrap()
                .try_into();
        assert_eq!(
            frame,
            Ok(CANOpenFrame::NMTNodeControlFrame(NMTNodeControlFrame::new(
                NMTCommand::Operational,
                NMTNodeControlAddress::AllNodes
            )))
        );
        let frame: Result<CANOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x000).unwrap(), &[0x02, 0x01])
                .unwrap()
                .try_into();
        assert_eq!(
            frame,
            Ok(CANOpenFrame::NMTNodeControlFrame(NMTNodeControlFrame::new(
                NMTCommand::Stopped,
                NMTNodeControlAddress::Node(1.try_into().unwrap())
            )))
        );
        let frame: Result<CANOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x000).unwrap(), &[0x80, 0x02])
                .unwrap()
                .try_into();
        assert_eq!(
            frame,
            Ok(CANOpenFrame::NMTNodeControlFrame(NMTNodeControlFrame::new(
                NMTCommand::PreOperational,
                NMTNodeControlAddress::Node(2.try_into().unwrap())
            )))
        );
        let frame: Result<CANOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x000).unwrap(), &[0x81, 0x03])
                .unwrap()
                .try_into();
        assert_eq!(
            frame,
            Ok(CANOpenFrame::NMTNodeControlFrame(NMTNodeControlFrame::new(
                NMTCommand::ResetNode,
                NMTNodeControlAddress::Node(3.try_into().unwrap())
            )))
        );
        let frame: Result<CANOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x000).unwrap(), &[0x82, 0x7F])
                .unwrap()
                .try_into();
        assert_eq!(
            frame,
            Ok(CANOpenFrame::NMTNodeControlFrame(NMTNodeControlFrame::new(
                NMTCommand::ResetCommunication,
                NMTNodeControlAddress::Node(127.try_into().unwrap())
            )))
        );
        let frame: Result<CANOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x000).unwrap(), &[0x00, 0x00])
                .unwrap()
                .try_into();
        assert_eq!(frame, Err(Error::InvalidNMTCommand(0)));
        let frame: Result<CANOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x000).unwrap(), &[0x03, 0x00])
                .unwrap()
                .try_into();
        assert_eq!(frame, Err(Error::InvalidNMTCommand(3)));
        let frame: Result<CANOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x000).unwrap(), &[0xFF, 0x00])
                .unwrap()
                .try_into();
        assert_eq!(frame, Err(Error::InvalidNMTCommand(255)));
        let frame: Result<CANOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x000).unwrap(), &[0x01, 0x80])
                .unwrap()
                .try_into();
        assert_eq!(frame, Err(Error::InvalidNodeId(128)));
        let frame: Result<CANOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x000).unwrap(), &[0x01, 0xFF])
                .unwrap()
                .try_into();
        assert_eq!(frame, Err(Error::InvalidNodeId(255)));
    }
}
