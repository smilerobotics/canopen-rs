use crate::id::{CommunicationObject, NodeId};

pub trait ConvertibleFrame {
    fn communication_object(&self) -> CommunicationObject;
    fn set_data<'a>(&self, buf: &'a mut [u8]) -> &'a [u8];
}

mod nmt_node_control;
pub use nmt_node_control::{NmtCommand, NmtNodeControlAddress, NmtNodeControlFrame};

mod sync;
pub use sync::SyncFrame;

mod emergency;
pub use emergency::EmergencyFrame;

pub(crate) mod sdo;
pub use sdo::SdoFrame;

mod nmt_node_monitoring;
pub use nmt_node_monitoring::{NmtNodeMonitoringFrame, NmtState};

#[derive(Debug, PartialEq)]
pub enum CanOpenFrame {
    NmtNodeControlFrame(NmtNodeControlFrame),
    SyncFrame(SyncFrame),
    EmergencyFrame(EmergencyFrame),
    SdoFrame(SdoFrame),
    NmtNodeMonitoringFrame(NmtNodeMonitoringFrame),
}

impl CanOpenFrame {
    pub fn new_nmt_node_control_frame(command: NmtCommand, address: NmtNodeControlAddress) -> Self {
        Self::NmtNodeControlFrame(NmtNodeControlFrame::new(command, address))
    }

    pub fn new_sdo_read_frame(node_id: NodeId, index: u16, sub_index: u8) -> Self {
        Self::SdoFrame(SdoFrame::new_sdo_read_frame(node_id, index, sub_index))
    }

    pub fn new_sdo_write_frame(
        node_id: NodeId,
        index: u16,
        sub_index: u8,
        data: std::vec::Vec<u8>,
    ) -> Self {
        Self::SdoFrame(SdoFrame::new_sdo_write_frame(
            node_id, index, sub_index, data,
        ))
    }
}
