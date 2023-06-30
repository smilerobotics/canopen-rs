use crate::error::{Error, Result};
use crate::frame::{CANOpenFrame, ToSocketCANFrame};
use crate::id::{CommunicationObject, NodeID};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NMTCommand {
    Operational = 0x01,
    Stopped = 0x02,
    PreOperational = 0x80,
    ResetNode = 0x81,
    ResetCommunication = 0x82,
}

impl NMTCommand {
    fn as_byte(&self) -> u8 {
        self.to_owned() as u8
    }

    fn from_byte(byte: u8) -> Result<Self> {
        match byte {
            0x01 => Ok(Self::Operational),
            0x02 => Ok(Self::Stopped),
            0x80 => Ok(Self::PreOperational),
            0x81 => Ok(Self::ResetNode),
            0x82 => Ok(Self::ResetCommunication),
            _ => Err(Error::InvalidNMTCommand(byte)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NMTNodeControlAddress {
    AllNodes,
    Node(NodeID),
}

impl NMTNodeControlAddress {
    fn as_byte(&self) -> u8 {
        match self {
            Self::AllNodes => 0x00,
            Self::Node(node_id) => node_id.as_raw(),
        }
    }

    fn from_byte(value: u8) -> Result<Self> {
        match value {
            0x00 => Ok(Self::AllNodes),
            _ => Ok(Self::Node(value.try_into()?)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct NMTNodeControlFrame {
    command: NMTCommand,
    address: NMTNodeControlAddress,
}

impl NMTNodeControlFrame {
    const FRAME_DATA_SIZE: usize = 2;

    pub fn new(command: NMTCommand, address: NMTNodeControlAddress) -> Self {
        Self { command, address }
    }

    pub(super) fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 2 {
            return Err(Error::InvalidDataLength {
                length: bytes.len(),
                data_type: "NMTNodeControlFrame".to_owned(),
            });
        }
        Ok(Self::new(
            NMTCommand::from_byte(bytes[0])?,
            NMTNodeControlAddress::from_byte(bytes[1])?,
        ))
    }
}

impl From<NMTNodeControlFrame> for CANOpenFrame {
    fn from(frame: NMTNodeControlFrame) -> Self {
        CANOpenFrame::NMTNodeControlFrame(frame)
    }
}

impl ToSocketCANFrame for NMTNodeControlFrame {
    fn communication_object(&self) -> CommunicationObject {
        CommunicationObject::NMTNodeControl
    }

    fn set_data(&self, buf: &mut [u8]) -> usize {
        assert!(buf.len() >= Self::FRAME_DATA_SIZE);

        buf[0] = self.command.as_byte();
        buf[1] = self.address.as_byte();
        Self::FRAME_DATA_SIZE
    }
}

#[cfg(test)]
mod tests {
    use socketcan::{EmbeddedFrame, Frame};

    use super::*;

    #[test]
    fn test_nmt_command_to_byte() {
        assert_eq!(NMTCommand::Operational.as_byte(), 0x01);
        assert_eq!(NMTCommand::Stopped.as_byte(), 0x02);
        assert_eq!(NMTCommand::PreOperational.as_byte(), 0x80);
        assert_eq!(NMTCommand::ResetNode.as_byte(), 0x81);
        assert_eq!(NMTCommand::ResetCommunication.as_byte(), 0x82);
    }

    #[test]
    fn test_nmt_command_from_byte() {
        let command = NMTCommand::from_byte(0x01);
        assert_eq!(command, Ok(NMTCommand::Operational));
        let command = NMTCommand::from_byte(0x02);
        assert_eq!(command, Ok(NMTCommand::Stopped));
        let command = NMTCommand::from_byte(0x80);
        assert_eq!(command, Ok(NMTCommand::PreOperational));
        let command = NMTCommand::from_byte(0x81);
        assert_eq!(command, Ok(NMTCommand::ResetNode));
        let command = NMTCommand::from_byte(0x82);
        assert_eq!(command, Ok(NMTCommand::ResetCommunication));
        let command = NMTCommand::from_byte(0x00);
        assert_eq!(command, Err(Error::InvalidNMTCommand(0x00)));
        let command = NMTCommand::from_byte(0x03);
        assert_eq!(command, Err(Error::InvalidNMTCommand(0x03)));
        let command = NMTCommand::from_byte(0xFF);
        assert_eq!(command, Err(Error::InvalidNMTCommand(0xFF)));
    }

    #[test]
    fn test_nmt_node_control_address_to_byte() {
        assert_eq!(NMTNodeControlAddress::AllNodes.as_byte(), 0x00);
        assert_eq!(
            NMTNodeControlAddress::Node(1.try_into().unwrap()).as_byte(),
            0x01
        );
        assert_eq!(
            NMTNodeControlAddress::Node(127.try_into().unwrap()).as_byte(),
            0x7F
        );
    }

    #[test]
    fn test_nmt_node_control_address_from_byte() {
        let address = NMTNodeControlAddress::from_byte(0x00);
        assert_eq!(address, Ok(NMTNodeControlAddress::AllNodes));
        let address = NMTNodeControlAddress::from_byte(0x01);
        assert_eq!(
            address,
            Ok(NMTNodeControlAddress::Node(1.try_into().unwrap()))
        );
        let address = NMTNodeControlAddress::from_byte(0x7F);
        assert_eq!(
            address,
            Ok(NMTNodeControlAddress::Node(127.try_into().unwrap()))
        );
        let address = NMTNodeControlAddress::from_byte(0x80);
        assert_eq!(address, Err(Error::InvalidNodeId(128)));
        let address = NMTNodeControlAddress::from_byte(0xFF);
        assert_eq!(address, Err(Error::InvalidNodeId(255)));
    }

    #[test]
    fn test_from_bytes() {
        let frame = NMTNodeControlFrame::from_bytes(&[0x01, 0x00]);
        assert_eq!(
            frame,
            Ok(NMTNodeControlFrame {
                command: NMTCommand::Operational,
                address: NMTNodeControlAddress::AllNodes
            })
        );
        let frame = NMTNodeControlFrame::from_bytes(&[0x02, 0x01]);
        assert_eq!(
            frame,
            Ok(NMTNodeControlFrame {
                command: NMTCommand::Stopped,
                address: NMTNodeControlAddress::Node(1.try_into().unwrap()),
            })
        );
        let frame = NMTNodeControlFrame::from_bytes(&[0x80, 0x02]);
        assert_eq!(
            frame,
            Ok(NMTNodeControlFrame {
                command: NMTCommand::PreOperational,
                address: NMTNodeControlAddress::Node(2.try_into().unwrap()),
            })
        );
        let frame = NMTNodeControlFrame::from_bytes(&[0x81, 0x03]);
        assert_eq!(
            frame,
            Ok(NMTNodeControlFrame {
                command: NMTCommand::ResetNode,
                address: NMTNodeControlAddress::Node(3.try_into().unwrap()),
            })
        );
        let frame = NMTNodeControlFrame::from_bytes(&[0x82, 0x7F]);
        assert_eq!(
            frame,
            Ok(NMTNodeControlFrame {
                command: NMTCommand::ResetCommunication,
                address: NMTNodeControlAddress::Node(127.try_into().unwrap()),
            })
        );
        let frame = NMTNodeControlFrame::from_bytes(&[0x00, 0x00]);
        assert_eq!(frame, Err(Error::InvalidNMTCommand(0)));
        let frame = NMTNodeControlFrame::from_bytes(&[0x03, 0x00]);
        assert_eq!(frame, Err(Error::InvalidNMTCommand(3)));
        let frame = NMTNodeControlFrame::from_bytes(&[0xFF, 0x00]);
        assert_eq!(frame, Err(Error::InvalidNMTCommand(255)));
        let frame = NMTNodeControlFrame::from_bytes(&[0x01, 0x80]);
        assert_eq!(frame, Err(Error::InvalidNodeId(128)));
        let frame = NMTNodeControlFrame::from_bytes(&[0x01, 0xFF]);
        assert_eq!(frame, Err(Error::InvalidNodeId(255)));
    }

    #[test]
    fn test_communication_object() {
        let frame =
            NMTNodeControlFrame::new(NMTCommand::Operational, NMTNodeControlAddress::AllNodes);
        assert_eq!(
            frame.communication_object(),
            CommunicationObject::NMTNodeControl
        );
        let frame = NMTNodeControlFrame::new(
            NMTCommand::Stopped,
            NMTNodeControlAddress::Node(1.try_into().unwrap()),
        );
        assert_eq!(
            frame.communication_object(),
            CommunicationObject::NMTNodeControl
        );
        let frame = NMTNodeControlFrame::new(
            NMTCommand::PreOperational,
            NMTNodeControlAddress::Node(2.try_into().unwrap()),
        );
        assert_eq!(
            frame.communication_object(),
            CommunicationObject::NMTNodeControl
        );
        let frame = NMTNodeControlFrame::new(
            NMTCommand::ResetNode,
            NMTNodeControlAddress::Node(3.try_into().unwrap()),
        );
        assert_eq!(
            frame.communication_object(),
            CommunicationObject::NMTNodeControl
        );

        let frame = NMTNodeControlFrame::new(
            NMTCommand::ResetCommunication,
            NMTNodeControlAddress::Node(127.try_into().unwrap()),
        );
        assert_eq!(
            frame.communication_object(),
            CommunicationObject::NMTNodeControl
        );
    }

    #[test]
    fn test_set_data() {
        let mut buf = [0u8; 8];

        let frame_data_size =
            NMTNodeControlFrame::new(NMTCommand::Operational, NMTNodeControlAddress::AllNodes)
                .set_data(&mut buf);
        assert_eq!(frame_data_size, 2);
        assert_eq!(&buf[..frame_data_size], &[0x01, 0x00]);

        let frame_data_size = NMTNodeControlFrame::new(
            NMTCommand::Stopped,
            NMTNodeControlAddress::Node(1.try_into().unwrap()),
        )
        .set_data(&mut buf);
        assert_eq!(frame_data_size, 2);
        assert_eq!(&buf[..frame_data_size], &[0x02, 0x01]);

        let frame_data_size = NMTNodeControlFrame::new(
            NMTCommand::PreOperational,
            NMTNodeControlAddress::Node(2.try_into().unwrap()),
        )
        .set_data(&mut buf);
        assert_eq!(frame_data_size, 2);
        assert_eq!(&buf[..frame_data_size], &[0x80, 0x02]);

        let frame_data_size = NMTNodeControlFrame::new(
            NMTCommand::ResetNode,
            NMTNodeControlAddress::Node(3.try_into().unwrap()),
        )
        .set_data(&mut buf);
        assert_eq!(frame_data_size, 2);
        assert_eq!(&buf[..frame_data_size], &[0x81, 0x03]);

        let frame_data_size = NMTNodeControlFrame::new(
            NMTCommand::ResetCommunication,
            NMTNodeControlAddress::Node(127.try_into().unwrap()),
        )
        .set_data(&mut buf);
        assert_eq!(frame_data_size, 2);
        assert_eq!(&buf[..frame_data_size], &[0x82, 0x7F]);
    }

    #[test]
    fn test_nmt_node_control_frame_to_socketcan_frame() {
        let frame =
            NMTNodeControlFrame::new(NMTCommand::Operational, NMTNodeControlAddress::AllNodes)
                .to_socketcan_frame();
        assert_eq!(frame.raw_id(), 0x00);
        assert_eq!(frame.data(), &[0x01, 0x00]);

        let frame = NMTNodeControlFrame::new(
            NMTCommand::Stopped,
            NMTNodeControlAddress::Node(1.try_into().unwrap()),
        )
        .to_socketcan_frame();
        assert_eq!(frame.raw_id(), 0x00);
        assert_eq!(frame.data(), &[0x02, 0x01]);

        let frame = NMTNodeControlFrame::new(
            NMTCommand::PreOperational,
            NMTNodeControlAddress::Node(2.try_into().unwrap()),
        )
        .to_socketcan_frame();
        assert_eq!(frame.raw_id(), 0x00);
        assert_eq!(frame.data(), &[0x80, 0x02]);

        let frame = NMTNodeControlFrame::new(
            NMTCommand::ResetNode,
            NMTNodeControlAddress::Node(3.try_into().unwrap()),
        )
        .to_socketcan_frame();
        assert_eq!(frame.raw_id(), 0x00);
        assert_eq!(frame.data(), &[0x81, 0x03]);

        let frame = NMTNodeControlFrame::new(
            NMTCommand::ResetCommunication,
            NMTNodeControlAddress::Node(127.try_into().unwrap()),
        )
        .to_socketcan_frame();
        assert_eq!(frame.raw_id(), 0x00);
        assert_eq!(frame.data(), &[0x82, 0x7F]);
    }
}
