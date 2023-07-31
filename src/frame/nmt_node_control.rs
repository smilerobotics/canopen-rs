use crate::error::{Error, Result};
use crate::frame::{CanOpenFrame, ConvertibleFrame};
use crate::id::{CommunicationObject, NodeId};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NmtCommand {
    Operational = 0x01,
    Stopped = 0x02,
    PreOperational = 0x80,
    ResetNode = 0x81,
    ResetCommunication = 0x82,
}

impl NmtCommand {
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
            _ => Err(Error::InvalidNmtCommand(byte)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NmtNodeControlAddress {
    AllNodes,
    Node(NodeId),
}

impl NmtNodeControlAddress {
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
pub struct NmtNodeControlFrame {
    pub command: NmtCommand,
    pub address: NmtNodeControlAddress,
}

impl NmtNodeControlFrame {
    const FRAME_DATA_SIZE: usize = 2;

    pub fn new(command: NmtCommand, address: NmtNodeControlAddress) -> Self {
        Self { command, address }
    }

    pub(crate) fn new_with_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != Self::FRAME_DATA_SIZE {
            return Err(Error::InvalidDataLength {
                length: bytes.len(),
                data_type: "NmtNodeControlFrame".to_owned(),
            });
        }
        Ok(Self::new(
            NmtCommand::from_byte(bytes[0])?,
            NmtNodeControlAddress::from_byte(bytes[1])?,
        ))
    }
}

impl From<NmtNodeControlFrame> for CanOpenFrame {
    fn from(frame: NmtNodeControlFrame) -> Self {
        CanOpenFrame::NmtNodeControlFrame(frame)
    }
}

impl ConvertibleFrame for NmtNodeControlFrame {
    fn communication_object(&self) -> CommunicationObject {
        CommunicationObject::NmtNodeControl
    }

    fn frame_data(&self) -> std::vec::Vec<u8> {
        let mut data = std::vec::Vec::with_capacity(Self::FRAME_DATA_SIZE);
        data.push(self.command.as_byte());
        data.push(self.address.as_byte());
        assert_eq!(data.len(), Self::FRAME_DATA_SIZE);
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nmt_command_to_byte() {
        assert_eq!(NmtCommand::Operational.as_byte(), 0x01);
        assert_eq!(NmtCommand::Stopped.as_byte(), 0x02);
        assert_eq!(NmtCommand::PreOperational.as_byte(), 0x80);
        assert_eq!(NmtCommand::ResetNode.as_byte(), 0x81);
        assert_eq!(NmtCommand::ResetCommunication.as_byte(), 0x82);
    }

    #[test]
    fn test_nmt_command_from_byte() {
        let command = NmtCommand::from_byte(0x01);
        assert_eq!(command, Ok(NmtCommand::Operational));
        let command = NmtCommand::from_byte(0x02);
        assert_eq!(command, Ok(NmtCommand::Stopped));
        let command = NmtCommand::from_byte(0x80);
        assert_eq!(command, Ok(NmtCommand::PreOperational));
        let command = NmtCommand::from_byte(0x81);
        assert_eq!(command, Ok(NmtCommand::ResetNode));
        let command = NmtCommand::from_byte(0x82);
        assert_eq!(command, Ok(NmtCommand::ResetCommunication));
        let command = NmtCommand::from_byte(0x00);
        assert_eq!(command, Err(Error::InvalidNmtCommand(0x00)));
        let command = NmtCommand::from_byte(0x03);
        assert_eq!(command, Err(Error::InvalidNmtCommand(0x03)));
        let command = NmtCommand::from_byte(0xFF);
        assert_eq!(command, Err(Error::InvalidNmtCommand(0xFF)));
    }

    #[test]
    fn test_nmt_node_control_address_to_byte() {
        assert_eq!(NmtNodeControlAddress::AllNodes.as_byte(), 0x00);
        assert_eq!(
            NmtNodeControlAddress::Node(1.try_into().unwrap()).as_byte(),
            0x01
        );
        assert_eq!(
            NmtNodeControlAddress::Node(127.try_into().unwrap()).as_byte(),
            0x7F
        );
    }

    #[test]
    fn test_nmt_node_control_address_from_byte() {
        let address = NmtNodeControlAddress::from_byte(0x00);
        assert_eq!(address, Ok(NmtNodeControlAddress::AllNodes));
        let address = NmtNodeControlAddress::from_byte(0x01);
        assert_eq!(
            address,
            Ok(NmtNodeControlAddress::Node(1.try_into().unwrap()))
        );
        let address = NmtNodeControlAddress::from_byte(0x7F);
        assert_eq!(
            address,
            Ok(NmtNodeControlAddress::Node(127.try_into().unwrap()))
        );
        let address = NmtNodeControlAddress::from_byte(0x80);
        assert_eq!(address, Err(Error::InvalidNodeId(128)));
        let address = NmtNodeControlAddress::from_byte(0xFF);
        assert_eq!(address, Err(Error::InvalidNodeId(255)));
    }

    #[test]
    fn test_from_bytes() {
        let frame = NmtNodeControlFrame::new_with_bytes(&[0x01, 0x00]);
        assert_eq!(
            frame,
            Ok(NmtNodeControlFrame {
                command: NmtCommand::Operational,
                address: NmtNodeControlAddress::AllNodes
            })
        );
        let frame = NmtNodeControlFrame::new_with_bytes(&[0x02, 0x01]);
        assert_eq!(
            frame,
            Ok(NmtNodeControlFrame {
                command: NmtCommand::Stopped,
                address: NmtNodeControlAddress::Node(1.try_into().unwrap()),
            })
        );
        let frame = NmtNodeControlFrame::new_with_bytes(&[0x80, 0x02]);
        assert_eq!(
            frame,
            Ok(NmtNodeControlFrame {
                command: NmtCommand::PreOperational,
                address: NmtNodeControlAddress::Node(2.try_into().unwrap()),
            })
        );
        let frame = NmtNodeControlFrame::new_with_bytes(&[0x81, 0x03]);
        assert_eq!(
            frame,
            Ok(NmtNodeControlFrame {
                command: NmtCommand::ResetNode,
                address: NmtNodeControlAddress::Node(3.try_into().unwrap()),
            })
        );
        let frame = NmtNodeControlFrame::new_with_bytes(&[0x82, 0x7F]);
        assert_eq!(
            frame,
            Ok(NmtNodeControlFrame {
                command: NmtCommand::ResetCommunication,
                address: NmtNodeControlAddress::Node(127.try_into().unwrap()),
            })
        );
        let frame = NmtNodeControlFrame::new_with_bytes(&[0x00, 0x00]);
        assert_eq!(frame, Err(Error::InvalidNmtCommand(0)));
        let frame = NmtNodeControlFrame::new_with_bytes(&[0x03, 0x00]);
        assert_eq!(frame, Err(Error::InvalidNmtCommand(3)));
        let frame = NmtNodeControlFrame::new_with_bytes(&[0xFF, 0x00]);
        assert_eq!(frame, Err(Error::InvalidNmtCommand(255)));
        let frame = NmtNodeControlFrame::new_with_bytes(&[0x01, 0x80]);
        assert_eq!(frame, Err(Error::InvalidNodeId(128)));
        let frame = NmtNodeControlFrame::new_with_bytes(&[0x01, 0xFF]);
        assert_eq!(frame, Err(Error::InvalidNodeId(255)));
    }

    #[test]
    fn test_communication_object() {
        let frame =
            NmtNodeControlFrame::new(NmtCommand::Operational, NmtNodeControlAddress::AllNodes);
        assert_eq!(
            frame.communication_object(),
            CommunicationObject::NmtNodeControl
        );
        let frame = NmtNodeControlFrame::new(
            NmtCommand::Stopped,
            NmtNodeControlAddress::Node(1.try_into().unwrap()),
        );
        assert_eq!(
            frame.communication_object(),
            CommunicationObject::NmtNodeControl
        );
        let frame = NmtNodeControlFrame::new(
            NmtCommand::PreOperational,
            NmtNodeControlAddress::Node(2.try_into().unwrap()),
        );
        assert_eq!(
            frame.communication_object(),
            CommunicationObject::NmtNodeControl
        );
        let frame = NmtNodeControlFrame::new(
            NmtCommand::ResetNode,
            NmtNodeControlAddress::Node(3.try_into().unwrap()),
        );
        assert_eq!(
            frame.communication_object(),
            CommunicationObject::NmtNodeControl
        );

        let frame = NmtNodeControlFrame::new(
            NmtCommand::ResetCommunication,
            NmtNodeControlAddress::Node(127.try_into().unwrap()),
        );
        assert_eq!(
            frame.communication_object(),
            CommunicationObject::NmtNodeControl
        );
    }

    #[test]
    fn test_set_data() {
        let mut buf = [0u8; 8];

        let data =
            NmtNodeControlFrame::new(NmtCommand::Operational, NmtNodeControlAddress::AllNodes)
                .frame_data();
        assert_eq!(data.len(), 2);
        assert_eq!(data, &[0x01, 0x00]);

        buf.fill(0x00);
        let data = NmtNodeControlFrame::new(
            NmtCommand::Stopped,
            NmtNodeControlAddress::Node(1.try_into().unwrap()),
        )
        .frame_data();
        assert_eq!(data.len(), 2);
        assert_eq!(data, &[0x02, 0x01]);

        buf.fill(0x00);
        let data = NmtNodeControlFrame::new(
            NmtCommand::PreOperational,
            NmtNodeControlAddress::Node(2.try_into().unwrap()),
        )
        .frame_data();
        assert_eq!(data.len(), 2);
        assert_eq!(data, &[0x80, 0x02]);

        buf.fill(0x00);
        let data = NmtNodeControlFrame::new(
            NmtCommand::ResetNode,
            NmtNodeControlAddress::Node(3.try_into().unwrap()),
        )
        .frame_data();
        assert_eq!(data.len(), 2);
        assert_eq!(data, &[0x81, 0x03]);

        buf.fill(0x00);
        let data = NmtNodeControlFrame::new(
            NmtCommand::ResetCommunication,
            NmtNodeControlAddress::Node(127.try_into().unwrap()),
        )
        .frame_data();
        assert_eq!(data.len(), 2);
        assert_eq!(data, &[0x82, 0x7F]);
    }
}
