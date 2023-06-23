use libc::CAN_MAX_DLEN;
use socketcan::EmbeddedFrame;

use crate::id::{CommunicationObject, NodeID};

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
