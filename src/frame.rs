use socketcan::embedded_can::Frame;
use socketcan::CanFrame;

use crate::id::CommunicationObject;

pub trait CANOpenFrame {
    fn communication_object(&self) -> CommunicationObject;
    fn set_data(&self, buf: &mut [u8]) -> usize;

    fn to_socketcan_frame(&self) -> CanFrame {
        let mut buf = [0u8; 8];
        let data_size = self.set_data(&mut buf);
        CanFrame::new(self.communication_object(), &buf[0..data_size]).unwrap()
        // TODO: define original error type and return `Result` type
    }
}

mod nmt_node_control;
pub use nmt_node_control::{NMTCommand, NMTNodeControlAddress, NMTNodeControlFrame};
