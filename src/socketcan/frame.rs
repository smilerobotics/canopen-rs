use libc::CAN_MAX_DLEN;
use socketcan::EmbeddedFrame;

use crate::error::{Error, Result};
use crate::frame::sdo::Direction;
use crate::frame::ConvertibleFrame;
use crate::frame::{
    CanOpenFrame, EmergencyFrame, NmtNodeControlFrame, NmtNodeMonitoringFrame, SdoFrame, SyncFrame,
};
use crate::id::CommunicationObject;

pub fn to_socketcan_frame<T: ConvertibleFrame>(frame: T) -> socketcan::CanFrame {
    let mut buf = [0u8; CAN_MAX_DLEN];
    let data = frame.set_data(&mut buf);
    socketcan::CanFrame::new(frame.communication_object(), data)
        .expect("Should have failed only when the data length exceeded `CAN_MAX_DLEN`")
}

impl From<CanOpenFrame> for socketcan::CanFrame {
    fn from(frame: CanOpenFrame) -> Self {
        match frame {
            CanOpenFrame::NmtNodeControlFrame(frame) => to_socketcan_frame(frame),
            CanOpenFrame::SyncFrame(frame) => to_socketcan_frame(frame),
            CanOpenFrame::EmergencyFrame(frame) => to_socketcan_frame(frame),
            CanOpenFrame::SdoFrame(frame) => to_socketcan_frame(frame),
            CanOpenFrame::NmtNodeMonitoringFrame(frame) => to_socketcan_frame(frame),
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
                        Ok(NmtNodeControlFrame::new_with_bytes(frame.data())?.into())
                    }
                    CommunicationObject::Sync => Ok(SyncFrame.into()),
                    CommunicationObject::Emergency(node_id) => {
                        Ok(EmergencyFrame::new_with_bytes(node_id, frame.data())?.into())
                    }
                    CommunicationObject::TxSdo(node_id) => {
                        Ok(SdoFrame::new_with_bytes(Direction::Tx, node_id, frame.data())?.into())
                    }
                    CommunicationObject::RxSdo(node_id) => {
                        Ok(SdoFrame::new_with_bytes(Direction::Rx, node_id, frame.data())?.into())
                    }
                    CommunicationObject::NmtNodeMonitoring(node_id) => {
                        Ok(NmtNodeMonitoringFrame::new_with_bytes(node_id, frame.data())?.into())
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
    use socketcan::{EmbeddedFrame, Frame};

    use super::*;

    use crate::frame::sdo::ClientCommandSpecifier;
    use crate::frame::{NmtCommand, NmtNodeControlAddress, NmtState};

    #[test]
    fn test_nmt_node_control_frame_to_socketcan_frame() {
        let frame = to_socketcan_frame(NmtNodeControlFrame::new(
            NmtCommand::Operational,
            NmtNodeControlAddress::AllNodes,
        ));
        assert_eq!(frame.raw_id(), 0x00);
        assert_eq!(frame.data(), &[0x01, 0x00]);

        let frame = to_socketcan_frame(NmtNodeControlFrame::new(
            NmtCommand::Stopped,
            NmtNodeControlAddress::Node(1.try_into().unwrap()),
        ));
        assert_eq!(frame.raw_id(), 0x00);
        assert_eq!(frame.data(), &[0x02, 0x01]);

        let frame = to_socketcan_frame(NmtNodeControlFrame::new(
            NmtCommand::PreOperational,
            NmtNodeControlAddress::Node(2.try_into().unwrap()),
        ));
        assert_eq!(frame.raw_id(), 0x00);
        assert_eq!(frame.data(), &[0x80, 0x02]);

        let frame = to_socketcan_frame(NmtNodeControlFrame::new(
            NmtCommand::ResetNode,
            NmtNodeControlAddress::Node(3.try_into().unwrap()),
        ));
        assert_eq!(frame.raw_id(), 0x00);
        assert_eq!(frame.data(), &[0x81, 0x03]);

        let frame = to_socketcan_frame(NmtNodeControlFrame::new(
            NmtCommand::ResetCommunication,
            NmtNodeControlAddress::Node(127.try_into().unwrap()),
        ));
        assert_eq!(frame.raw_id(), 0x00);
        assert_eq!(frame.data(), &[0x82, 0x7F]);
    }

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
    fn test_sync_frame_to_socketcan_frame() {
        let frame = to_socketcan_frame(SyncFrame::new());
        assert_eq!(frame.raw_id(), 0x080);
        assert_eq!(frame.data(), &[]);
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
    fn test_emergency_frame_to_socketcan_frame() {
        let frame = to_socketcan_frame(EmergencyFrame::new(1.try_into().unwrap(), 0x0000, 0x00));
        assert_eq!(frame.raw_id(), 0x081);
        assert_eq!(
            frame.data(),
            &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        );

        let frame = to_socketcan_frame(EmergencyFrame::new(2.try_into().unwrap(), 0x1000, 0x01));
        assert_eq!(frame.raw_id(), 0x082);
        assert_eq!(
            frame.data(),
            &[0x00, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00]
        );

        let frame = to_socketcan_frame(EmergencyFrame::new(127.try_into().unwrap(), 0x1234, 0x56));
        assert_eq!(frame.raw_id(), 0x0FF);
        assert_eq!(
            frame.data(),
            &[0x34, 0x12, 0x56, 0x00, 0x00, 0x00, 0x00, 0x00]
        );
    }

    #[test]
    fn test_socketcan_frame_to_emergency_frame() {
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
    fn test_sdo_frame_to_socketcan_frame() {
        let frame = to_socketcan_frame(SdoFrame::new_sdo_read_frame(
            1.try_into().unwrap(),
            0x1018,
            2,
        )); // Product code
        assert_eq!(frame.raw_id(), 0x601);
        assert_eq!(
            frame.data(),
            &[0x40, 0x18, 0x10, 0x02, 0x00, 0x00, 0x00, 0x00]
        );

        let frame = to_socketcan_frame(SdoFrame::new_sdo_write_frame(
            1.try_into().unwrap(),
            0x1402,
            2,
            vec![255],
        )); // Transmission type RxPDO3
        assert_eq!(frame.raw_id(), 0x601);
        assert_eq!(
            frame.data(),
            &[0x2F, 0x02, 0x14, 0x02, 0xFF, 0x00, 0x00, 0x00]
        );

        let frame = to_socketcan_frame(SdoFrame::new_sdo_write_frame(
            2.try_into().unwrap(),
            0x1017,
            0,
            1000u16.to_le_bytes().into(),
        )); // Producer heartbeat time
        assert_eq!(frame.raw_id(), 0x602);
        assert_eq!(
            frame.data(),
            &[0x2B, 0x17, 0x10, 0x00, 0xE8, 0x03, 0x00, 0x00]
        );

        let frame = to_socketcan_frame(SdoFrame::new_sdo_write_frame(
            3.try_into().unwrap(),
            0x1200,
            1,
            0x060Au32.to_le_bytes().into(),
        )); // COB-ID SDO client to server
        assert_eq!(frame.raw_id(), 0x603);
        assert_eq!(
            frame.data(),
            &[0x23, 0x00, 0x12, 0x01, 0x0A, 0x06, 0x00, 0x00]
        );

        let frame = to_socketcan_frame(SdoFrame {
            direction: Direction::Tx,
            ccs: ClientCommandSpecifier::InitiateUpload,
            node_id: 4.try_into().unwrap(),
            // Device type
            index: 0x1000,
            sub_index: 0,
            size: Some(4),
            expedited: true,
            data: vec![0x92, 0x01, 0x02, 0x00],
        });
        assert_eq!(frame.raw_id(), 0x584);
        assert_eq!(
            frame.data(),
            &[0x43, 0x00, 0x10, 0x00, 0x92, 0x01, 0x02, 0x00]
        );

        let frame = to_socketcan_frame(SdoFrame {
            direction: Direction::Tx,
            ccs: ClientCommandSpecifier::AbortTransfer,
            node_id: 5.try_into().unwrap(),
            // Device type
            index: 0x1000,
            sub_index: 0,
            size: None,
            expedited: false,
            data: vec![0x02, 0x00, 0x01, 0x06], // SDO_ERR_ACCESS_RO
        });
        assert_eq!(frame.raw_id(), 0x585);
        assert_eq!(
            frame.data(),
            &[0x80, 0x00, 0x10, 0x00, 0x02, 0x00, 0x01, 0x06]
        );
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
    fn test_nmt_node_monitoring_frame_to_socketcan_frame() {
        let frame = to_socketcan_frame(NmtNodeMonitoringFrame::new(
            1.try_into().unwrap(),
            NmtState::BootUp,
        ));
        assert_eq!(frame.raw_id(), 0x701);
        assert_eq!(frame.data(), &[0x00]);

        let frame = to_socketcan_frame(NmtNodeMonitoringFrame::new(
            2.try_into().unwrap(),
            NmtState::Stopped,
        ));
        assert_eq!(frame.raw_id(), 0x702);
        assert_eq!(frame.data(), &[0x04]);

        let frame = to_socketcan_frame(NmtNodeMonitoringFrame::new(
            3.try_into().unwrap(),
            NmtState::Operational,
        ));
        assert_eq!(frame.raw_id(), 0x703);
        assert_eq!(frame.data(), &[0x05]);

        let frame = to_socketcan_frame(NmtNodeMonitoringFrame::new(
            4.try_into().unwrap(),
            NmtState::PreOperational,
        ));
        assert_eq!(frame.raw_id(), 0x704);
        assert_eq!(frame.data(), &[0x7F]);
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
