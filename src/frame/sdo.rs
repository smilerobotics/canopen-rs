use crate::frame::ToSocketCANFrame;
use crate::id::{CommunicationObject, NodeID};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    #[allow(dead_code)]
    Tx,
    Rx,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ClientCommandSpecifier {
    #[allow(dead_code)]
    SegmentDownload = 0,
    InitiateDownload = 1,
    InitiateUpload = 2,
    #[allow(dead_code)]
    SegmentUpload = 3,
    #[allow(dead_code)]
    AbortTransfer = 4,
    #[allow(dead_code)]
    BlockUpload = 5,
    #[allow(dead_code)]
    BlockDownload = 6,
}

#[derive(Debug, PartialEq)]
pub struct SDOFrame {
    direction: Direction,
    node_id: NodeID,
    ccs: ClientCommandSpecifier,
    index: u16,
    sub_index: u8,
    expedited: bool,
    size_specified: bool,
    data: std::vec::Vec<u8>,
}

impl SDOFrame {
    const FRAME_DATA_SIZE: usize = 8;
    const DATA_CONTENT_SIZE: usize = 4;

    pub fn new_sdo_read_frame(node_id: NodeID, index: u16, sub_index: u8) -> Self {
        Self {
            direction: Direction::Rx,
            node_id: node_id,
            ccs: ClientCommandSpecifier::InitiateUpload,
            index: index,
            sub_index: sub_index,
            expedited: false,
            size_specified: false,
            data: std::vec::Vec::new(),
        }
    }

    pub fn new_sdo_write_frame(node_id: NodeID, index: u16, sub_index: u8, data: &[u8]) -> Self {
        Self {
            direction: Direction::Rx,
            node_id: node_id,
            ccs: ClientCommandSpecifier::InitiateDownload,
            index: index,
            sub_index: sub_index,
            expedited: true,
            size_specified: true,
            data: data.into(),
        }
    }
}

impl ToSocketCANFrame for SDOFrame {
    fn communication_object(&self) -> CommunicationObject {
        match self.direction {
            Direction::Tx => CommunicationObject::TxSDO(self.node_id),
            Direction::Rx => CommunicationObject::RxSDO(self.node_id),
        }
    }

    fn set_data(&self, buf: &mut [u8]) -> usize {
        assert!(buf.len() >= Self::FRAME_DATA_SIZE);
        assert!(self.data.len() <= Self::DATA_CONTENT_SIZE);

        buf[0] = ((self.ccs as u8) << 5)
            + match self.size_specified {
                true => (((4 - self.data.len()) as u8) << 2) & 0b1100,
                false => 0,
            }
            + ((self.expedited as u8) << 1)
            + (self.size_specified as u8);
        buf[1..3].copy_from_slice(&self.index.to_le_bytes());
        buf[3] = self.sub_index;
        buf[4..4 + self.data.len()].copy_from_slice(self.data.as_ref());
        buf[4 + self.data.len()..].fill(0x00);

        Self::FRAME_DATA_SIZE
    }
}

#[cfg(test)]
mod tests {
    use socketcan::{EmbeddedFrame, Frame};

    use super::*;

    #[test]
    fn test_sdo_read_frame() {
        let frame = SDOFrame::new_sdo_read_frame(1.try_into().unwrap(), 0x1018, 2); // Product code
        assert_eq!(
            frame,
            SDOFrame {
                direction: Direction::Rx,
                ccs: ClientCommandSpecifier::InitiateUpload,
                node_id: 1.try_into().unwrap(),
                index: 0x1018,
                sub_index: 2,
                expedited: false,
                size_specified: false,
                data: vec![],
            }
        )
    }

    #[test]
    fn test_sdo_write_frame() {
        let frame = SDOFrame::new_sdo_write_frame(1.try_into().unwrap(), 0x1402, 2, &[255]); // Transmission type RxPDO3
        assert_eq!(
            frame,
            SDOFrame {
                direction: Direction::Rx,
                ccs: ClientCommandSpecifier::InitiateDownload,
                node_id: 1.try_into().unwrap(),
                index: 0x1402,
                sub_index: 2,
                expedited: true,
                size_specified: true,
                data: vec![0xFF],
            }
        );

        let frame = SDOFrame::new_sdo_write_frame(
            2.try_into().unwrap(),
            0x1017,
            0,
            &(1000u16.to_le_bytes()),
        ); // Producer heartbeat time
        assert_eq!(
            frame,
            SDOFrame {
                direction: Direction::Rx,
                ccs: ClientCommandSpecifier::InitiateDownload,
                node_id: 2.try_into().unwrap(),
                index: 0x1017,
                sub_index: 0,
                expedited: true,
                size_specified: true,
                data: vec![0xE8, 0x03],
            }
        );

        let frame = SDOFrame::new_sdo_write_frame(
            3.try_into().unwrap(),
            0x1200,
            1,
            &(0x060Au32.to_le_bytes()),
        ); // COB-ID SDO client to server
        assert_eq!(
            frame,
            SDOFrame {
                direction: Direction::Rx,
                ccs: ClientCommandSpecifier::InitiateDownload,
                node_id: 3.try_into().unwrap(),
                index: 0x1200,
                sub_index: 1,
                expedited: true,
                size_specified: true,
                data: vec![0x0A, 0x06, 0x00, 0x00],
            }
        )
    }

    #[test]
    fn test_communication_object() {
        let frame = SDOFrame {
            direction: Direction::Rx,
            ccs: ClientCommandSpecifier::InitiateUpload,
            node_id: 1.try_into().unwrap(),
            // Product code
            index: 0x1018,
            sub_index: 2,
            expedited: false,
            size_specified: false,
            data: vec![],
        };
        assert_eq!(
            frame.communication_object(),
            CommunicationObject::RxSDO(1.try_into().unwrap())
        );

        let frame = SDOFrame {
            direction: Direction::Rx,
            ccs: ClientCommandSpecifier::InitiateDownload,
            node_id: 3.try_into().unwrap(),
            // COB-ID SDO client to server
            index: 0x1200,
            sub_index: 1,
            expedited: true,
            size_specified: true,
            data: vec![0x0A, 0x06, 0x00, 0x00],
        };
        assert_eq!(
            frame.communication_object(),
            CommunicationObject::RxSDO(3.try_into().unwrap())
        );

        let frame = SDOFrame {
            direction: Direction::Tx,
            ccs: ClientCommandSpecifier::InitiateUpload,
            node_id: 4.try_into().unwrap(),
            // Device type
            index: 0x1000,
            sub_index: 0,
            expedited: true,
            size_specified: true,
            data: vec![0x92, 0x01, 0x02, 0x00],
        };
        assert_eq!(
            frame.communication_object(),
            CommunicationObject::TxSDO(4.try_into().unwrap())
        );

        let frame = SDOFrame {
            direction: Direction::Tx,
            ccs: ClientCommandSpecifier::AbortTransfer,
            node_id: 5.try_into().unwrap(),
            // Device type
            index: 0x1000,
            sub_index: 0,
            expedited: false,
            size_specified: false,
            data: vec![0x02, 0x00, 0x01, 0x06], // SDO_ERR_ACCESS_RO
        };
        assert_eq!(
            frame.communication_object(),
            CommunicationObject::TxSDO(5.try_into().unwrap())
        );
    }

    #[test]
    fn test_set_data() {
        let mut buf = [0u8; 8];

        let frame_data_size = SDOFrame {
            direction: Direction::Rx,
            ccs: ClientCommandSpecifier::InitiateUpload,
            node_id: 1.try_into().unwrap(),
            // Product code
            index: 0x1018,
            sub_index: 2,
            expedited: false,
            size_specified: false,
            data: vec![],
        }
        .set_data(&mut buf);
        assert_eq!(frame_data_size, 8);
        assert_eq!(
            &buf[..frame_data_size],
            &[0x40, 0x18, 0x10, 0x02, 0x00, 0x00, 0x00, 0x00]
        );

        buf.fill(0x00);
        let frame_data_size = SDOFrame {
            direction: Direction::Rx,
            ccs: ClientCommandSpecifier::InitiateDownload,
            node_id: 1.try_into().unwrap(),
            // Transmission type RxPDO3
            index: 0x1402,
            sub_index: 2,
            expedited: true,
            size_specified: true,
            data: vec![0xFF],
        }
        .set_data(&mut buf);
        assert_eq!(frame_data_size, 8);
        assert_eq!(
            &buf[..frame_data_size],
            &[0x2F, 0x02, 0x14, 0x02, 0xFF, 0x00, 0x00, 0x00]
        );

        buf.fill(0x00);
        let frame_data_size = SDOFrame {
            direction: Direction::Rx,
            ccs: ClientCommandSpecifier::InitiateDownload,
            node_id: 2.try_into().unwrap(),
            // Producer heartbeat time
            index: 0x1017,
            sub_index: 0,
            expedited: true,
            size_specified: true,
            data: vec![0xE8, 0x03],
        }
        .set_data(&mut buf);
        assert_eq!(frame_data_size, 8);
        assert_eq!(
            &buf[..frame_data_size],
            &[0x2B, 0x17, 0x10, 0x00, 0xE8, 0x03, 0x00, 0x00]
        );

        buf.fill(0x00);
        let frame_data_size = SDOFrame {
            direction: Direction::Rx,
            ccs: ClientCommandSpecifier::InitiateDownload,
            node_id: 3.try_into().unwrap(),
            // COB-ID SDO client to server
            index: 0x1200,
            sub_index: 1,
            expedited: true,
            size_specified: true,
            data: vec![0x0A, 0x06, 0x00, 0x00],
        }
        .set_data(&mut buf);
        assert_eq!(frame_data_size, 8);
        assert_eq!(
            &buf[0..frame_data_size],
            &[0x23, 0x00, 0x12, 0x01, 0x0A, 0x06, 0x00, 0x00]
        );

        buf.fill(0x00);
        let frame_data_size = SDOFrame {
            direction: Direction::Tx,
            ccs: ClientCommandSpecifier::InitiateUpload,
            node_id: 4.try_into().unwrap(),
            // Device type
            index: 0x1000,
            sub_index: 0,
            expedited: true,
            size_specified: true,
            data: vec![0x92, 0x01, 0x02, 0x00],
        }
        .set_data(&mut buf);
        assert_eq!(frame_data_size, 8);
        assert_eq!(
            &buf[0..frame_data_size],
            &[0x43, 0x00, 0x10, 0x00, 0x92, 0x01, 0x02, 0x00]
        );

        buf.fill(0x00);
        let frame_data_size = SDOFrame {
            direction: Direction::Tx,
            ccs: ClientCommandSpecifier::AbortTransfer,
            node_id: 5.try_into().unwrap(),
            // Device type
            index: 0x1000,
            sub_index: 0,
            expedited: false,
            size_specified: false,
            data: vec![0x02, 0x00, 0x01, 0x06], // SDO_ERR_ACCESS_RO
        }
        .set_data(&mut buf);
        assert_eq!(frame_data_size, 8);
        assert_eq!(
            &buf[0..frame_data_size],
            &[0x80, 0x00, 0x10, 0x00, 0x02, 0x00, 0x01, 0x06]
        );
    }

    #[test]
    fn test_sdo_frame_to_socketcan_frame() {
        let frame =
            SDOFrame::new_sdo_read_frame(1.try_into().unwrap(), 0x1018, 2).to_socketcan_frame(); // Product code
        assert_eq!(frame.raw_id(), 0x601);
        assert_eq!(
            frame.data(),
            &[0x40, 0x18, 0x10, 0x02, 0x00, 0x00, 0x00, 0x00]
        );

        let frame = SDOFrame::new_sdo_write_frame(1.try_into().unwrap(), 0x1402, 2, &[255]) // Transmission type RxPDO3
            .to_socketcan_frame();
        assert_eq!(frame.raw_id(), 0x601);
        assert_eq!(
            frame.data(),
            &[0x2F, 0x02, 0x14, 0x02, 0xFF, 0x00, 0x00, 0x00]
        );

        let frame = SDOFrame::new_sdo_write_frame(
            2.try_into().unwrap(),
            0x1017,
            0,
            &(1000u16.to_le_bytes()),
        ) // Producer heartbeat time
        .to_socketcan_frame();
        assert_eq!(frame.raw_id(), 0x602);
        assert_eq!(
            frame.data(),
            &[0x2B, 0x17, 0x10, 0x00, 0xE8, 0x03, 0x00, 0x00]
        );

        let frame = SDOFrame::new_sdo_write_frame(
            3.try_into().unwrap(),
            0x1200,
            1,
            &(0x060Au32.to_le_bytes()),
        ) // COB-ID SDO client to server
        .to_socketcan_frame();
        assert_eq!(frame.raw_id(), 0x603);
        assert_eq!(
            frame.data(),
            &[0x23, 0x00, 0x12, 0x01, 0x0A, 0x06, 0x00, 0x00]
        );

        let frame = SDOFrame {
            direction: Direction::Tx,
            ccs: ClientCommandSpecifier::InitiateUpload,
            node_id: 4.try_into().unwrap(),
            // Device type
            index: 0x1000,
            sub_index: 0,
            expedited: false,
            size_specified: false,
            data: vec![0x92, 0x01, 0x02, 0x00],
        }
        .to_socketcan_frame();
        assert_eq!(frame.raw_id(), 0x584);
        assert_eq!(
            frame.data(),
            &[0x40, 0x00, 0x10, 0x00, 0x92, 0x01, 0x02, 0x00]
        );

        let frame = SDOFrame {
            direction: Direction::Tx,
            ccs: ClientCommandSpecifier::AbortTransfer,
            node_id: 5.try_into().unwrap(),
            // Device type
            index: 0x1000,
            sub_index: 0,
            expedited: false,
            size_specified: false,
            data: vec![0x02, 0x00, 0x01, 0x06], // SDO_ERR_ACCESS_RO
        }
        .to_socketcan_frame();
        assert_eq!(frame.raw_id(), 0x585);
        assert_eq!(
            frame.data(),
            &[0x80, 0x00, 0x10, 0x00, 0x02, 0x00, 0x01, 0x06]
        );
    }
}
