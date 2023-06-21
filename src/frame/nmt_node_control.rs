use crate::frame::CANOpenFrame;
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
    fn to_byte(&self) -> u8 {
        self.to_owned() as u8
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NMTNodeControlAddress {
    AllNodes,
    Node(NodeID),
}

impl NMTNodeControlAddress {
    fn to_byte(&self) -> u8 {
        match self {
            Self::AllNodes => 0x00,
            Self::Node(node_id) => *node_id,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct NMTNodeControlFrame {
    command: NMTCommand,
    address: NMTNodeControlAddress,
}

impl NMTNodeControlFrame {
    pub fn new(command: NMTCommand, address: NMTNodeControlAddress) -> Self {
        Self {
            command: command,
            address: address,
        }
    }
}

impl CANOpenFrame for NMTNodeControlFrame {
    fn communication_object(&self) -> CommunicationObject {
        CommunicationObject::NMTNodeControl
    }

    fn set_data(&self, buf: &mut [u8]) -> usize {
        buf[0] = self.command.to_byte();
        buf[1] = self.address.to_byte();
        2
    }
}

#[cfg(test)]
mod tests {
    use socketcan::embedded_can::Frame as EmbeddedFrame;
    use socketcan::Frame;

    use super::*;

    #[test]
    fn test_nmt_command() {
        assert_eq!(NMTCommand::Operational.to_byte(), 0x01);
        assert_eq!(NMTCommand::Stopped.to_byte(), 0x02);
        assert_eq!(NMTCommand::PreOperational.to_byte(), 0x80);
        assert_eq!(NMTCommand::ResetNode.to_byte(), 0x81);
        assert_eq!(NMTCommand::ResetCommunication.to_byte(), 0x82);
    }

    #[test]
    fn test_nmt_node_control_address() {
        assert_eq!(NMTNodeControlAddress::AllNodes.to_byte(), 0x00);
        assert_eq!(NMTNodeControlAddress::Node(1).to_byte(), 0x01);
        assert_eq!(NMTNodeControlAddress::Node(127).to_byte(), 0x7F);
    }

    #[test]
    fn test_nmt_node_control_frame_to_socketcan_frame() {
        let frame =
            NMTNodeControlFrame::new(NMTCommand::Operational, NMTNodeControlAddress::AllNodes)
                .to_socketcan_frame();
        assert_eq!(frame.raw_id(), 0x00);
        assert_eq!(frame.data(), &[0x01, 0x00]);

        let frame = NMTNodeControlFrame::new(NMTCommand::Stopped, NMTNodeControlAddress::Node(1))
            .to_socketcan_frame();
        assert_eq!(frame.raw_id(), 0x00);
        assert_eq!(frame.data(), &[0x02, 0x01]);

        let frame =
            NMTNodeControlFrame::new(NMTCommand::PreOperational, NMTNodeControlAddress::Node(2))
                .to_socketcan_frame();
        assert_eq!(frame.raw_id(), 0x00);
        assert_eq!(frame.data(), &[0x80, 0x02]);

        let frame = NMTNodeControlFrame::new(NMTCommand::ResetNode, NMTNodeControlAddress::Node(3))
            .to_socketcan_frame();
        assert_eq!(frame.raw_id(), 0x00);
        assert_eq!(frame.data(), &[0x81, 0x03]);

        let frame = NMTNodeControlFrame::new(
            NMTCommand::ResetCommunication,
            NMTNodeControlAddress::Node(127),
        )
        .to_socketcan_frame();
        assert_eq!(frame.raw_id(), 0x00);
        assert_eq!(frame.data(), &[0x82, 0x7F]);
    }
}
