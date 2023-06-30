use libc::CAN_MAX_DLEN;
use socketcan::EmbeddedFrame;

use crate::id::{CommunicationObject, NodeId};
use crate::{Error, Result};

trait ToSocketCanFrame {
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
pub use nmt_node_control::{NmtCommand, NmtNodeControlAddress, NmtNodeControlFrame};

mod sync;
pub use sync::SyncFrame;

mod emergency;
pub use emergency::EmergencyFrame;

mod sdo;
pub use sdo::{ClientCommandSpecifier, Direction, SdoFrame};

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

    pub fn new_sdo_write_frame(node_id: NodeId, index: u16, sub_index: u8, data: &[u8]) -> Self {
        Self::SdoFrame(SdoFrame::new_sdo_write_frame(
            node_id, index, sub_index, data,
        ))
    }
}

impl From<CanOpenFrame> for socketcan::CanFrame {
    fn from(frame: CanOpenFrame) -> Self {
        match frame {
            CanOpenFrame::NmtNodeControlFrame(frame) => frame.to_socketcan_frame(),
            CanOpenFrame::SyncFrame(frame) => frame.to_socketcan_frame(),
            CanOpenFrame::EmergencyFrame(frame) => frame.to_socketcan_frame(),
            CanOpenFrame::SdoFrame(frame) => frame.to_socketcan_frame(),
            CanOpenFrame::NmtNodeMonitoringFrame(frame) => frame.to_socketcan_frame(),
        }
    }
}

impl TryFrom<socketcan::CanFrame> for CanOpenFrame {
    type Error = Error;
    fn try_from(frame: socketcan::CanFrame) -> Result<Self> {
        match frame {
            socketcan::CanFrame::Data(frame) => {
                let cob: CommunicationObject = frame.id().try_into()?;
                match cob {
                    CommunicationObject::NmtNodeControl => {
                        Ok(NmtNodeControlFrame::from_bytes(frame.data())?.into())
                    }
                    CommunicationObject::Sync => Ok(SyncFrame.into()),
                    CommunicationObject::Emergency(node_id) => {
                        Ok(EmergencyFrame::from_node_id_bytes(node_id, frame.data())?.into())
                    }
                    CommunicationObject::TxSdo(node_id) => {
                        Ok(SdoFrame::from_direction_node_id_bytes(
                            Direction::Tx,
                            node_id,
                            frame.data(),
                        )?
                        .into())
                    }
                    CommunicationObject::RxSdo(node_id) => {
                        Ok(SdoFrame::from_direction_node_id_bytes(
                            Direction::Rx,
                            node_id,
                            frame.data(),
                        )?
                        .into())
                    }
                    CommunicationObject::NmtNodeMonitoring(node_id) => Ok(
                        NmtNodeMonitoringFrame::from_node_id_bytes(node_id, frame.data())?.into(),
                    ),
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
        let frame: Result<CanOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x000).unwrap(), &[0x01, 0x00])
                .unwrap()
                .try_into();
        assert_eq!(
            frame,
            Ok(CanOpenFrame::NmtNodeControlFrame(NmtNodeControlFrame {
                command: NmtCommand::Operational,
                address: NmtNodeControlAddress::AllNodes
            }))
        );
        let frame: Result<CanOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x000).unwrap(), &[0x02, 0x01])
                .unwrap()
                .try_into();
        assert_eq!(
            frame,
            Ok(CanOpenFrame::NmtNodeControlFrame(NmtNodeControlFrame {
                command: NmtCommand::Stopped,
                address: NmtNodeControlAddress::Node(1.try_into().unwrap())
            }))
        );
        let frame: Result<CanOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x000).unwrap(), &[0x80, 0x02])
                .unwrap()
                .try_into();
        assert_eq!(
            frame,
            Ok(CanOpenFrame::NmtNodeControlFrame(NmtNodeControlFrame {
                command: NmtCommand::PreOperational,
                address: NmtNodeControlAddress::Node(2.try_into().unwrap())
            }))
        );
        let frame: Result<CanOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x000).unwrap(), &[0x81, 0x03])
                .unwrap()
                .try_into();
        assert_eq!(
            frame,
            Ok(CanOpenFrame::NmtNodeControlFrame(NmtNodeControlFrame {
                command: NmtCommand::ResetNode,
                address: NmtNodeControlAddress::Node(3.try_into().unwrap())
            }))
        );
        let frame: Result<CanOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x000).unwrap(), &[0x82, 0x7F])
                .unwrap()
                .try_into();
        assert_eq!(
            frame,
            Ok(CanOpenFrame::NmtNodeControlFrame(NmtNodeControlFrame {
                command: NmtCommand::ResetCommunication,
                address: NmtNodeControlAddress::Node(127.try_into().unwrap())
            }))
        );
        let frame: Result<CanOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x000).unwrap(), &[0x00, 0x00])
                .unwrap()
                .try_into();
        assert_eq!(frame, Err(Error::InvalidNmtCommand(0)));
        let frame: Result<CanOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x000).unwrap(), &[0x03, 0x00])
                .unwrap()
                .try_into();
        assert_eq!(frame, Err(Error::InvalidNmtCommand(3)));
        let frame: Result<CanOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x000).unwrap(), &[0xFF, 0x00])
                .unwrap()
                .try_into();
        assert_eq!(frame, Err(Error::InvalidNmtCommand(255)));
        let frame: Result<CanOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x000).unwrap(), &[0x01, 0x80])
                .unwrap()
                .try_into();
        assert_eq!(frame, Err(Error::InvalidNodeId(128)));
        let frame: Result<CanOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x000).unwrap(), &[0x01, 0xFF])
                .unwrap()
                .try_into();
        assert_eq!(frame, Err(Error::InvalidNodeId(255)));
    }

    #[test]
    fn test_socketcan_frame_to_sync_frame() {
        let frame: Result<CanOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x080).unwrap(), &[])
                .unwrap()
                .try_into();
        assert_eq!(frame, Ok(CanOpenFrame::SyncFrame(SyncFrame)));
    }

    #[test]
    fn test_socketcan_frame_to_emergyncy_frame() {
        let frame: Result<CanOpenFrame> = socketcan::CanFrame::new(
            socketcan::StandardId::new(0x081).unwrap(),
            &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        )
        .unwrap()
        .try_into();
        assert_eq!(
            frame,
            Ok(CanOpenFrame::EmergencyFrame(EmergencyFrame {
                node_id: 1.try_into().unwrap(),
                error_code: 0x0000,
                error_register: 0x00
            }))
        );

        let frame: Result<CanOpenFrame> = socketcan::CanFrame::new(
            socketcan::StandardId::new(0x082).unwrap(),
            &[0x00, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00],
        )
        .unwrap()
        .try_into();
        assert_eq!(
            frame,
            Ok(CanOpenFrame::EmergencyFrame(EmergencyFrame {
                node_id: 2.try_into().unwrap(),
                error_code: 0x1000,
                error_register: 0x01
            }))
        );

        let frame: Result<CanOpenFrame> = socketcan::CanFrame::new(
            socketcan::StandardId::new(0x0FF).unwrap(),
            &[0x34, 0x12, 0x56, 0x00, 0x00, 0x00, 0x00, 0x00],
        )
        .unwrap()
        .try_into();
        assert_eq!(
            frame,
            Ok(CanOpenFrame::EmergencyFrame(EmergencyFrame {
                node_id: 127.try_into().unwrap(),
                error_code: 0x1234,
                error_register: 0x56
            }))
        );

        let frame: Result<CanOpenFrame> = socketcan::CanFrame::new(
            socketcan::StandardId::new(0x081).unwrap(),
            &[0x00, 0x00, 0x00],
        )
        .unwrap()
        .try_into();
        assert!(frame.is_err());
    }

    #[test]
    fn test_socketcan_frame_to_sdo_frame() {
        let frame: Result<CanOpenFrame> = socketcan::CanFrame::new(
            socketcan::StandardId::new(0x601).unwrap(),
            &[0x40, 0x18, 0x10, 0x02, 0x00, 0x00, 0x00, 0x00],
        )
        .unwrap()
        .try_into();
        assert_eq!(
            frame,
            Ok(CanOpenFrame::SdoFrame(SdoFrame {
                direction: Direction::Rx,
                node_id: 1.try_into().unwrap(),
                ccs: ClientCommandSpecifier::InitiateUpload,
                index: 0x1018,
                sub_index: 2,
                size: None,
                expedited: false,
                data: vec![],
            }))
        );
        let frame: Result<CanOpenFrame> = socketcan::CanFrame::new(
            socketcan::StandardId::new(0x601).unwrap(),
            &[0x2F, 0x02, 0x14, 0x02, 0xFF, 0x00, 0x00, 0x00],
        )
        .unwrap()
        .try_into();
        assert_eq!(
            frame,
            Ok(CanOpenFrame::SdoFrame(SdoFrame {
                direction: Direction::Rx,
                node_id: 1.try_into().unwrap(),
                ccs: ClientCommandSpecifier::InitiateDownload,
                index: 0x1402,
                sub_index: 2,
                size: Some(1),
                expedited: true,
                data: vec![0xFF],
            }))
        );
        let frame: Result<CanOpenFrame> = socketcan::CanFrame::new(
            socketcan::StandardId::new(0x602).unwrap(),
            &[0x2B, 0x17, 0x10, 0x00, 0xE8, 0x03, 0x00, 0x00],
        )
        .unwrap()
        .try_into();
        assert_eq!(
            frame,
            Ok(CanOpenFrame::SdoFrame(SdoFrame {
                direction: Direction::Rx,
                node_id: 2.try_into().unwrap(),
                ccs: ClientCommandSpecifier::InitiateDownload,
                index: 0x1017,
                sub_index: 0,
                size: Some(2),
                expedited: true,
                data: vec![0xE8, 0x03],
            }))
        );
        let frame: Result<CanOpenFrame> = socketcan::CanFrame::new(
            socketcan::StandardId::new(0x603).unwrap(),
            &[0x23, 0x00, 0x12, 0x01, 0x0A, 0x06, 0x00, 0x00],
        )
        .unwrap()
        .try_into();
        assert_eq!(
            frame,
            Ok(CanOpenFrame::SdoFrame(SdoFrame {
                direction: Direction::Rx,
                node_id: 3.try_into().unwrap(),
                ccs: ClientCommandSpecifier::InitiateDownload,
                index: 0x1200,
                sub_index: 1,
                size: Some(4),
                expedited: true,
                data: vec![0x0A, 0x06, 0x00, 0x00],
            }))
        );
        let frame: Result<CanOpenFrame> = socketcan::CanFrame::new(
            socketcan::StandardId::new(0x584).unwrap(),
            &[0x43, 0x00, 0x10, 0x00, 0x92, 0x01, 0x02, 0x00],
        )
        .unwrap()
        .try_into();
        assert_eq!(
            frame,
            Ok(CanOpenFrame::SdoFrame(SdoFrame {
                direction: Direction::Tx,
                node_id: 4.try_into().unwrap(),
                ccs: ClientCommandSpecifier::InitiateUpload,
                index: 0x1000,
                sub_index: 0,
                size: Some(4),
                expedited: true,
                data: vec![0x92, 0x01, 0x02, 0x00],
            }))
        );
        let frame: Result<CanOpenFrame> = socketcan::CanFrame::new(
            socketcan::StandardId::new(0x585).unwrap(),
            &[0x80, 0x00, 0x10, 0x00, 0x02, 0x00, 0x01, 0x06],
        )
        .unwrap()
        .try_into();
        assert_eq!(
            frame,
            Ok(CanOpenFrame::SdoFrame(SdoFrame {
                direction: Direction::Tx,
                node_id: 5.try_into().unwrap(),
                ccs: ClientCommandSpecifier::AbortTransfer,
                index: 0x1000,
                sub_index: 0,
                size: None,
                expedited: false,
                data: vec![0x02, 0x00, 0x01, 0x06],
            }))
        );
    }

    #[test]
    fn test_socketcan_frame_to_nmt_node_monitoring_frame() {
        let frame: Result<CanOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x701).unwrap(), &[0x00])
                .unwrap()
                .try_into();
        assert_eq!(
            frame,
            Ok(CanOpenFrame::NmtNodeMonitoringFrame(
                NmtNodeMonitoringFrame {
                    node_id: 1.try_into().unwrap(),
                    state: NmtState::BootUp,
                }
            ))
        );

        let frame: Result<CanOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x702).unwrap(), &[0x04])
                .unwrap()
                .try_into();
        assert_eq!(
            frame,
            Ok(CanOpenFrame::NmtNodeMonitoringFrame(
                NmtNodeMonitoringFrame {
                    node_id: 2.try_into().unwrap(),
                    state: NmtState::Stopped,
                }
            ))
        );

        let frame: Result<CanOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x703).unwrap(), &[0x05])
                .unwrap()
                .try_into();
        assert_eq!(
            frame,
            Ok(CanOpenFrame::NmtNodeMonitoringFrame(
                NmtNodeMonitoringFrame {
                    node_id: 3.try_into().unwrap(),
                    state: NmtState::Operational,
                }
            ))
        );

        let frame: Result<CanOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x704).unwrap(), &[0x7F])
                .unwrap()
                .try_into();
        assert_eq!(
            frame,
            Ok(CanOpenFrame::NmtNodeMonitoringFrame(
                NmtNodeMonitoringFrame {
                    node_id: 4.try_into().unwrap(),
                    state: NmtState::PreOperational,
                }
            ))
        );

        let frame: Result<CanOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x705).unwrap(), &[0x01])
                .unwrap()
                .try_into();
        assert_eq!(frame, Err(Error::InvalidNmtState(0x01)));

        let frame: Result<CanOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x706).unwrap(), &[0x06])
                .unwrap()
                .try_into();
        assert_eq!(frame, Err(Error::InvalidNmtState(0x06)));

        let frame: Result<CanOpenFrame> =
            socketcan::CanFrame::new(socketcan::StandardId::new(0x708).unwrap(), &[0x80])
                .unwrap()
                .try_into();
        assert_eq!(frame, Err(Error::InvalidNmtState(0x80)));
    }
}
