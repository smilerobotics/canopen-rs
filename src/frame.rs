use socketcan::embedded_can::Frame;

use crate::id::{CommunicationObject, NodeID};

trait ToSocketCANFrame {
    fn communication_object(&self) -> CommunicationObject;
    fn set_data(&self, buf: &mut [u8]) -> usize;

    fn to_socketcan_frame(&self) -> socketcan::CanFrame {
        let mut buf = [0u8; 8];
        let data_size = self.set_data(&mut buf);
        socketcan::CanFrame::new(self.communication_object(), &buf[0..data_size]).unwrap()
        // TODO: define original error type and return `Result` type
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
