use crate::error::{Error, Result};
use crate::frame::{CanOpenFrame, ConvertibleFrame};
use crate::id::{CommunicationObject, NodeId};

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Direction {
    Tx,
    Rx,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum ClientCommandSpecifier {
    SegmentDownload = 0,
    InitiateDownload = 1,
    InitiateUpload = 2,
    SegmentUpload = 3,
    AbortTransfer = 4,
    BlockUpload = 5,
    BlockDownload = 6,
}

impl ClientCommandSpecifier {
    fn from_num(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::SegmentDownload),
            1 => Ok(Self::InitiateDownload),
            2 => Ok(Self::InitiateUpload),
            3 => Ok(Self::SegmentUpload),
            4 => Ok(Self::AbortTransfer),
            5 => Ok(Self::BlockUpload),
            6 => Ok(Self::BlockDownload),
            _ => Err(Error::InvalidClientCommandSpecifier(value)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SdoFrame {
    pub(crate) direction: Direction,
    pub(crate) node_id: NodeId,
    pub(crate) ccs: ClientCommandSpecifier,
    pub(crate) index: u16,
    pub(crate) sub_index: u8,
    pub(crate) size: Option<usize>,
    pub(crate) expedited: bool,
    pub(crate) data: std::vec::Vec<u8>,
}

impl SdoFrame {
    const FRAME_DATA_SIZE: usize = 8;
    const DATA_CONTENT_SIZE: usize = 4;

    pub fn new_sdo_read_frame(node_id: NodeId, index: u16, sub_index: u8) -> Self {
        Self {
            direction: Direction::Rx,
            node_id,
            ccs: ClientCommandSpecifier::InitiateUpload,
            index,
            sub_index,
            size: None,
            expedited: false,
            data: std::vec::Vec::new(),
        }
    }

    pub fn new_sdo_write_frame(
        node_id: NodeId,
        index: u16,
        sub_index: u8,
        data: std::vec::Vec<u8>,
    ) -> Self {
        Self {
            direction: Direction::Rx,
            node_id,
            ccs: ClientCommandSpecifier::InitiateDownload,
            index,
            sub_index,
            size: Some(data.len()),
            expedited: true,
            data,
        }
    }

    pub(crate) fn new_with_bytes(
        direction: Direction,
        node_id: NodeId,
        bytes: &[u8],
    ) -> Result<Self> {
        // cf. https://en.wikipedia.org/wiki/CANopen#Service_Data_Object_(SDO)_protocol
        let ccs = ClientCommandSpecifier::from_num(bytes[0] >> 5)?;
        let expedited: bool = (bytes[0] & 0b0010) != 0;
        let size = match bytes[0] & 0b0001 {
            0 => None,
            _ => Some((4 - ((bytes[0] & 0b1100) >> 2)) as usize),
        };
        let bytes_len_to_be = 4 + match ccs {
            ClientCommandSpecifier::AbortTransfer => 4,
            _ => size.unwrap_or(0),
        };
        if bytes.len() < bytes_len_to_be {
            return Err(Error::InvalidDataLength {
                length: bytes.len(),
                data_type: "SdoFrame".to_owned(),
            });
        }
        let index: u16 = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
        let sub_index: u8 = bytes[3];
        Ok(Self {
            direction,
            node_id,
            ccs,
            index,
            sub_index,
            size,
            expedited,
            data: bytes[4..bytes_len_to_be].to_owned(),
        })
    }
}

impl From<SdoFrame> for CanOpenFrame {
    fn from(frame: SdoFrame) -> Self {
        CanOpenFrame::SdoFrame(frame)
    }
}

impl ConvertibleFrame for SdoFrame {
    fn communication_object(&self) -> CommunicationObject {
        match self.direction {
            Direction::Tx => CommunicationObject::TxSdo(self.node_id),
            Direction::Rx => CommunicationObject::RxSdo(self.node_id),
        }
    }

    fn frame_data(&self) -> std::vec::Vec<u8> {
        assert!(self.data.len() <= Self::DATA_CONTENT_SIZE);
        let mut data = std::vec::Vec::with_capacity(Self::FRAME_DATA_SIZE);
        // cf. https://en.wikipedia.org/wiki/CANopen#Service_Data_Object_(SDO)_protocol
        data.push(
            ((self.ccs as u8) << 5)
                + self
                    .size
                    .map_or(0, |size| (((4 - size) as u8) << 2) & 0b1100)
                + ((self.expedited as u8) << 1)
                + (self.size.is_some() as u8),
        );
        data.extend_from_slice(&self.index.to_le_bytes());
        data.push(self.sub_index);
        data.extend_from_slice(self.data.as_ref());
        data.resize(Self::FRAME_DATA_SIZE, 0x00);
        assert_eq!(data.len(), Self::FRAME_DATA_SIZE);
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ccs_from_num() {
        assert_eq!(
            ClientCommandSpecifier::from_num(0).unwrap(),
            ClientCommandSpecifier::SegmentDownload
        );
        assert_eq!(
            ClientCommandSpecifier::from_num(1).unwrap(),
            ClientCommandSpecifier::InitiateDownload
        );
        assert_eq!(
            ClientCommandSpecifier::from_num(2).unwrap(),
            ClientCommandSpecifier::InitiateUpload
        );
        assert_eq!(
            ClientCommandSpecifier::from_num(3).unwrap(),
            ClientCommandSpecifier::SegmentUpload
        );
        assert_eq!(
            ClientCommandSpecifier::from_num(4).unwrap(),
            ClientCommandSpecifier::AbortTransfer
        );
        assert_eq!(
            ClientCommandSpecifier::from_num(5).unwrap(),
            ClientCommandSpecifier::BlockUpload
        );
        assert_eq!(
            ClientCommandSpecifier::from_num(6).unwrap(),
            ClientCommandSpecifier::BlockDownload
        );
        match ClientCommandSpecifier::from_num(7).unwrap_err() {
            Error::InvalidClientCommandSpecifier(7) => (),
            _ => panic!("Error mismatch"),
        };
        match ClientCommandSpecifier::from_num(8).unwrap_err() {
            Error::InvalidClientCommandSpecifier(8) => (),
            _ => panic!("Error mismatch"),
        };
        match ClientCommandSpecifier::from_num(255).unwrap_err() {
            Error::InvalidClientCommandSpecifier(255) => (),
            _ => panic!("Error mismatch"),
        };
    }

    #[test]
    fn test_sdo_read_frame() {
        let frame = SdoFrame::new_sdo_read_frame(1.try_into().unwrap(), 0x1018, 2); // Product code
        assert_eq!(
            frame,
            SdoFrame {
                direction: Direction::Rx,
                ccs: ClientCommandSpecifier::InitiateUpload,
                node_id: 1.try_into().unwrap(),
                index: 0x1018,
                sub_index: 2,
                size: None,
                expedited: false,
                data: vec![],
            }
        )
    }

    #[test]
    fn test_sdo_write_frame() {
        let frame = SdoFrame::new_sdo_write_frame(1.try_into().unwrap(), 0x1402, 2, vec![255]); // Transmission type RxPDO3
        assert_eq!(
            frame,
            SdoFrame {
                direction: Direction::Rx,
                ccs: ClientCommandSpecifier::InitiateDownload,
                node_id: 1.try_into().unwrap(),
                index: 0x1402,
                sub_index: 2,
                size: Some(1),
                expedited: true,
                data: vec![0xFF],
            }
        );

        let frame = SdoFrame::new_sdo_write_frame(
            2.try_into().unwrap(),
            0x1017,
            0,
            1000u16.to_le_bytes().into(),
        ); // Producer heartbeat time
        assert_eq!(
            frame,
            SdoFrame {
                direction: Direction::Rx,
                ccs: ClientCommandSpecifier::InitiateDownload,
                node_id: 2.try_into().unwrap(),
                index: 0x1017,
                sub_index: 0,
                size: Some(2),
                expedited: true,
                data: vec![0xE8, 0x03],
            }
        );

        let frame = SdoFrame::new_sdo_write_frame(
            3.try_into().unwrap(),
            0x1200,
            1,
            0x060Au32.to_le_bytes().into(),
        ); // COB-ID SDO client to server
        assert_eq!(
            frame,
            SdoFrame {
                direction: Direction::Rx,
                ccs: ClientCommandSpecifier::InitiateDownload,
                node_id: 3.try_into().unwrap(),
                index: 0x1200,
                sub_index: 1,
                size: Some(4),
                expedited: true,
                data: vec![0x0A, 0x06, 0x00, 0x00],
            }
        )
    }

    #[test]
    fn test_from_direction_node_id_bytes() {
        assert_eq!(
            SdoFrame::new_with_bytes(
                Direction::Rx,
                1.try_into().unwrap(),
                &[0x40, 0x18, 0x10, 0x02, 0x00, 0x00, 0x00, 0x00],
            )
            .unwrap(),
            SdoFrame {
                direction: Direction::Rx,
                ccs: ClientCommandSpecifier::InitiateUpload,
                node_id: 1.try_into().unwrap(),
                index: 0x1018,
                sub_index: 2,
                size: None,
                expedited: false,
                data: vec![],
            }
        );
        assert_eq!(
            SdoFrame::new_with_bytes(
                Direction::Rx,
                1.try_into().unwrap(),
                &[0x2F, 0x02, 0x14, 0x02, 0xFF, 0x00, 0x00, 0x00],
            )
            .unwrap(),
            SdoFrame {
                direction: Direction::Rx,
                ccs: ClientCommandSpecifier::InitiateDownload,
                node_id: 1.try_into().unwrap(),
                index: 0x1402,
                sub_index: 2,
                size: Some(1),
                expedited: true,
                data: vec![0xFF],
            }
        );
        assert_eq!(
            SdoFrame::new_with_bytes(
                Direction::Rx,
                2.try_into().unwrap(),
                &[0x2B, 0x17, 0x10, 0x00, 0xE8, 0x03, 0x00, 0x00],
            )
            .unwrap(),
            SdoFrame {
                direction: Direction::Rx,
                ccs: ClientCommandSpecifier::InitiateDownload,
                node_id: 2.try_into().unwrap(),
                index: 0x1017,
                sub_index: 0,
                size: Some(2),
                expedited: true,
                data: vec![0xE8, 0x03],
            }
        );
        assert_eq!(
            SdoFrame::new_with_bytes(
                Direction::Rx,
                3.try_into().unwrap(),
                &[0x23, 0x00, 0x12, 0x01, 0x0A, 0x06, 0x00, 0x00],
            )
            .unwrap(),
            SdoFrame {
                direction: Direction::Rx,
                ccs: ClientCommandSpecifier::InitiateDownload,
                node_id: 3.try_into().unwrap(),
                index: 0x1200,
                sub_index: 1,
                size: Some(4),
                expedited: true,
                data: vec![0x0A, 0x06, 0x00, 0x00],
            }
        );
        assert_eq!(
            SdoFrame::new_with_bytes(
                Direction::Tx,
                4.try_into().unwrap(),
                &[0x43, 0x00, 0x10, 0x00, 0x92, 0x01, 0x02, 0x00],
            )
            .unwrap(),
            SdoFrame {
                direction: Direction::Tx,
                ccs: ClientCommandSpecifier::InitiateUpload,
                node_id: 4.try_into().unwrap(),
                index: 0x1000,
                sub_index: 0,
                size: Some(4),
                expedited: true,
                data: vec![0x92, 0x01, 0x02, 0x00],
            }
        );
        assert_eq!(
            SdoFrame::new_with_bytes(
                Direction::Tx,
                5.try_into().unwrap(),
                &[0x80, 0x00, 0x10, 0x00, 0x02, 0x00, 0x01, 0x06],
            )
            .unwrap(),
            SdoFrame {
                direction: Direction::Tx,
                ccs: ClientCommandSpecifier::AbortTransfer,
                node_id: 5.try_into().unwrap(),
                index: 0x1000,
                sub_index: 0,
                size: None,
                expedited: false,
                data: vec![0x02, 0x00, 0x01, 0x06],
            }
        );
    }

    #[test]
    fn test_communication_object() {
        let frame = SdoFrame {
            direction: Direction::Rx,
            ccs: ClientCommandSpecifier::InitiateUpload,
            node_id: 1.try_into().unwrap(),
            // Product code
            index: 0x1018,
            sub_index: 2,
            size: None,
            expedited: false,
            data: vec![],
        };
        assert_eq!(
            frame.communication_object(),
            CommunicationObject::RxSdo(1.try_into().unwrap())
        );

        let frame = SdoFrame {
            direction: Direction::Rx,
            ccs: ClientCommandSpecifier::InitiateDownload,
            node_id: 3.try_into().unwrap(),
            // COB-ID SDO client to server
            index: 0x1200,
            sub_index: 1,
            size: Some(4),
            expedited: true,
            data: vec![0x0A, 0x06, 0x00, 0x00],
        };
        assert_eq!(
            frame.communication_object(),
            CommunicationObject::RxSdo(3.try_into().unwrap())
        );

        let frame = SdoFrame {
            direction: Direction::Tx,
            ccs: ClientCommandSpecifier::InitiateUpload,
            node_id: 4.try_into().unwrap(),
            // Device type
            index: 0x1000,
            sub_index: 0,
            size: Some(4),
            expedited: true,
            data: vec![0x92, 0x01, 0x02, 0x00],
        };
        assert_eq!(
            frame.communication_object(),
            CommunicationObject::TxSdo(4.try_into().unwrap())
        );

        let frame = SdoFrame {
            direction: Direction::Tx,
            ccs: ClientCommandSpecifier::AbortTransfer,
            node_id: 5.try_into().unwrap(),
            // Device type
            index: 0x1000,
            sub_index: 0,
            size: Some(4),
            expedited: false,
            data: vec![0x02, 0x00, 0x01, 0x06], // SDO_ERR_ACCESS_RO
        };
        assert_eq!(
            frame.communication_object(),
            CommunicationObject::TxSdo(5.try_into().unwrap())
        );
    }

    #[test]
    fn test_set_data() {
        let mut buf = [0u8; 8];

        let data = SdoFrame {
            direction: Direction::Rx,
            ccs: ClientCommandSpecifier::InitiateUpload,
            node_id: 1.try_into().unwrap(),
            // Product code
            index: 0x1018,
            sub_index: 2,
            size: None,
            expedited: false,
            data: vec![],
        }
        .frame_data();
        assert_eq!(data.len(), 8);
        assert_eq!(data, &[0x40, 0x18, 0x10, 0x02, 0x00, 0x00, 0x00, 0x00]);

        buf.fill(0x00);
        let data = SdoFrame {
            direction: Direction::Rx,
            ccs: ClientCommandSpecifier::InitiateDownload,
            node_id: 1.try_into().unwrap(),
            // Transmission type RxPDO3
            index: 0x1402,
            sub_index: 2,
            size: Some(1),
            expedited: true,
            data: vec![0xFF],
        }
        .frame_data();
        assert_eq!(data.len(), 8);
        assert_eq!(data, &[0x2F, 0x02, 0x14, 0x02, 0xFF, 0x00, 0x00, 0x00]);

        buf.fill(0x00);
        let data = SdoFrame {
            direction: Direction::Rx,
            ccs: ClientCommandSpecifier::InitiateDownload,
            node_id: 2.try_into().unwrap(),
            // Producer heartbeat time
            index: 0x1017,
            sub_index: 0,
            size: Some(2),
            expedited: true,
            data: vec![0xE8, 0x03],
        }
        .frame_data();
        assert_eq!(data.len(), 8);
        assert_eq!(data, &[0x2B, 0x17, 0x10, 0x00, 0xE8, 0x03, 0x00, 0x00]);

        buf.fill(0x00);
        let data = SdoFrame {
            direction: Direction::Rx,
            ccs: ClientCommandSpecifier::InitiateDownload,
            node_id: 3.try_into().unwrap(),
            // COB-ID SDO client to server
            index: 0x1200,
            sub_index: 1,
            size: Some(4),
            expedited: true,
            data: vec![0x0A, 0x06, 0x00, 0x00],
        }
        .frame_data();
        assert_eq!(data.len(), 8);
        assert_eq!(data, &[0x23, 0x00, 0x12, 0x01, 0x0A, 0x06, 0x00, 0x00]);

        buf.fill(0x00);
        let data = SdoFrame {
            direction: Direction::Tx,
            ccs: ClientCommandSpecifier::InitiateUpload,
            node_id: 4.try_into().unwrap(),
            // Device type
            index: 0x1000,
            sub_index: 0,
            size: Some(4),
            expedited: true,
            data: vec![0x92, 0x01, 0x02, 0x00],
        }
        .frame_data();
        assert_eq!(data.len(), 8);
        assert_eq!(data, &[0x43, 0x00, 0x10, 0x00, 0x92, 0x01, 0x02, 0x00]);

        buf.fill(0x00);
        let data = SdoFrame {
            direction: Direction::Tx,
            ccs: ClientCommandSpecifier::AbortTransfer,
            node_id: 5.try_into().unwrap(),
            // Device type
            index: 0x1000,
            sub_index: 0,
            size: None,
            expedited: false,
            data: vec![0x02, 0x00, 0x01, 0x06], // SDO_ERR_ACCESS_RO
        }
        .frame_data();
        assert_eq!(data.len(), 8);
        assert_eq!(data, &[0x80, 0x00, 0x10, 0x00, 0x02, 0x00, 0x01, 0x06]);
    }
}
