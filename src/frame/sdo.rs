use crate::error::{Error, Result};
use crate::frame::{CanOpenFrame, ConvertibleFrame};
use crate::id::{CommunicationObject, NodeId};

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Direction {
    Tx,
    Rx,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum CommandSpecifier {
    AbortTransfer,
    Client(ClientCommandSpecifier),
    Server(ServerCommandSpecifier),
}

impl CommandSpecifier {
    fn new(direction: Direction, value: u8) -> Result<Self> {
        if value == 4 {
            return Ok(Self::AbortTransfer);
        }
        match direction {
            Direction::Tx => Ok(Self::Server(ServerCommandSpecifier::from_num(value)?)),
            Direction::Rx => Ok(Self::Client(ClientCommandSpecifier::from_num(value)?)),
        }
    }

    fn to_num(&self) -> u8 {
        match self {
            Self::AbortTransfer => 4,
            Self::Client(ccs) => *ccs as u8,
            Self::Server(scs) => *scs as u8,
        }
    }

    fn to_byte_fragment(&self) -> u8 {
        self.to_num() << 5
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum ClientCommandSpecifier {
    DownloadSegmentRequest = 0,
    InitiateDownloadRequest = 1,
    InitiateUploadRequest = 2,
    UploadSegmentRequest = 3,
    BlockUpload = 5,
    BlockDownload = 6,
}

impl ClientCommandSpecifier {
    fn from_num(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::DownloadSegmentRequest),
            1 => Ok(Self::InitiateDownloadRequest),
            2 => Ok(Self::InitiateUploadRequest),
            3 => Ok(Self::UploadSegmentRequest),
            5 => Ok(Self::BlockUpload),
            6 => Ok(Self::BlockDownload),
            _ => Err(Error::InvalidCommandSpecifier(value)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum ServerCommandSpecifier {
    UploadSegmentResponse = 0,
    DownloadSegmentResponse = 1,
    InitiateUploadResponse = 2,
    InitiateDownloadResponse = 3,
    BlockDownload = 5,
    BlockUpload = 6,
}

impl ServerCommandSpecifier {
    fn from_num(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::UploadSegmentResponse),
            1 => Ok(Self::DownloadSegmentResponse),
            2 => Ok(Self::InitiateUploadResponse),
            3 => Ok(Self::InitiateDownloadResponse),
            5 => Ok(Self::BlockDownload),
            6 => Ok(Self::BlockUpload),
            _ => Err(Error::InvalidCommandSpecifier(value)),
        }
    }
}

#[derive(Debug, PartialEq)]
enum SdoFrameTransferType {
    #[allow(dead_code)]
    Normal { size: usize },
    Expedited {
        size: Option<usize>,
        data: std::vec::Vec<u8>,
    },
}

impl SdoFrameTransferType {
    fn new_with_bytes(bytes: &[u8]) -> Result<Self> {
        assert!(bytes.len() == 8);
        if (bytes[0] & 0b0010) != 0 {
            let size = match bytes[0] & 0b0001 {
                0 => None,
                _ => Some((4 - ((bytes[0] & 0b1100) >> 2)) as usize),
            };
            let data_end_pos = 4 + size.unwrap_or(0);
            Ok(Self::Expedited {
                size,
                data: bytes[4..data_end_pos].to_owned(),
            })
        } else {
            Ok(Self::Normal {
                size: u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize,
            })
        }
    }

    fn to_byte_fragment(&self) -> u8 {
        match self {
            Self::Normal { size: _ } => 0x00,
            Self::Expedited { size, data: _ } => {
                size.map_or(0, |size| (((4 - size) as u8) << 2) & 0b1100)
                    + 0x02
                    + (size.is_some() as u8)
            }
        }
    }
}

#[derive(Debug, PartialEq)]
enum SdoFrameData {
    InitiateDownloadRequest {
        index: u16,
        sub_index: u8,
        transfer_type: SdoFrameTransferType,
    },
    InitiateDownloadResponse {
        index: u16,
        sub_index: u8,
    },
    InitiateUploadRequest {
        index: u16,
        sub_index: u8,
    },
    InitiateUploadResponse {
        index: u16,
        sub_index: u8,
        transfer_type: SdoFrameTransferType,
    },
    AbortTransfer {
        index: u16,
        sub_index: u8,
        abort_code: u32,
    },
}

fn prepare_frame_bytes_without_data(
    buf: &mut std::vec::Vec<u8>,
    index: u16,
    sub_index: u8,
    command_specifier: CommandSpecifier,
) {
    buf.push(command_specifier.to_byte_fragment());
    buf.extend_from_slice(&index.to_le_bytes());
    buf.push(sub_index);
}

fn prepare_frame_bytes_with_data(
    buf: &mut std::vec::Vec<u8>,
    index: u16,
    sub_index: u8,
    command_specifier: CommandSpecifier,
    transfer_type: &SdoFrameTransferType,
) {
    buf.push(command_specifier.to_byte_fragment() + transfer_type.to_byte_fragment());
    buf.extend_from_slice(&index.to_le_bytes());
    buf.push(sub_index);
    match transfer_type {
        SdoFrameTransferType::Normal { size } => {
            buf.extend_from_slice(&size.to_le_bytes());
        }
        SdoFrameTransferType::Expedited { size: _, data } => {
            buf.extend_from_slice(&data);
        }
    }
}

impl SdoFrameData {
    const DATA_SIZE: usize = 8;

    fn to_bytes(&self) -> std::vec::Vec<u8> {
        // cf. https://en.wikipedia.org/wiki/CANopen#Service_Data_Object_(SDO)_protocol
        let mut buf = std::vec::Vec::with_capacity(Self::DATA_SIZE);
        match self {
            SdoFrameData::AbortTransfer {
                index,
                sub_index,
                abort_code,
            } => {
                buf.push(CommandSpecifier::AbortTransfer.to_byte_fragment());
                buf.extend_from_slice(&index.to_le_bytes());
                buf.push(*sub_index);
                buf.extend_from_slice(&abort_code.to_le_bytes())
            }
            SdoFrameData::InitiateDownloadRequest {
                index,
                sub_index,
                transfer_type,
            } => {
                prepare_frame_bytes_with_data(
                    &mut buf,
                    *index,
                    *sub_index,
                    CommandSpecifier::Client(ClientCommandSpecifier::InitiateDownloadRequest),
                    transfer_type,
                );
            }
            SdoFrameData::InitiateDownloadResponse { index, sub_index } => {
                prepare_frame_bytes_without_data(
                    &mut buf,
                    *index,
                    *sub_index,
                    CommandSpecifier::Server(ServerCommandSpecifier::InitiateDownloadResponse),
                );
            }
            SdoFrameData::InitiateUploadRequest { index, sub_index } => {
                prepare_frame_bytes_without_data(
                    &mut buf,
                    *index,
                    *sub_index,
                    CommandSpecifier::Client(ClientCommandSpecifier::InitiateUploadRequest),
                );
            }
            SdoFrameData::InitiateUploadResponse {
                index,
                sub_index,
                transfer_type,
            } => {
                prepare_frame_bytes_with_data(
                    &mut buf,
                    *index,
                    *sub_index,
                    CommandSpecifier::Server(ServerCommandSpecifier::InitiateUploadResponse),
                    transfer_type,
                );
            }
        }
        buf.resize(Self::DATA_SIZE, 0x00);
        buf
    }
}

#[derive(Debug, PartialEq)]
pub struct SdoFrame {
    direction: Direction,
    node_id: NodeId,
    frame_data: SdoFrameData,
}

impl SdoFrame {
    const FRAME_DATA_SIZE: usize = 8;

    pub fn new_sdo_read_frame(node_id: NodeId, index: u16, sub_index: u8) -> Self {
        Self {
            direction: Direction::Rx,
            node_id,
            frame_data: SdoFrameData::InitiateUploadRequest { index, sub_index },
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
            frame_data: SdoFrameData::InitiateDownloadRequest {
                index,
                sub_index,
                transfer_type: SdoFrameTransferType::Expedited {
                    size: Some(data.len()),
                    data,
                },
            },
        }
    }

    pub(crate) fn new_with_bytes(
        direction: Direction,
        node_id: NodeId,
        bytes: &[u8],
    ) -> Result<Self> {
        if bytes.len() < Self::FRAME_DATA_SIZE {
            return Err(Error::InvalidDataLength {
                length: bytes.len(),
                data_type: "SdoFrame".to_owned(),
            });
        }
        // cf. https://en.wikipedia.org/wiki/CANopen#Service_Data_Object_(SDO)_protocol
        let command_specifier = CommandSpecifier::new(direction, bytes[0] >> 5)?;
        match command_specifier {
            CommandSpecifier::AbortTransfer
            | CommandSpecifier::Client(ClientCommandSpecifier::InitiateDownloadRequest)
            | CommandSpecifier::Server(ServerCommandSpecifier::InitiateDownloadResponse)
            | CommandSpecifier::Client(ClientCommandSpecifier::InitiateUploadRequest)
            | CommandSpecifier::Server(ServerCommandSpecifier::InitiateUploadResponse) => {
                let index: u16 = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                let sub_index: u8 = bytes[3];
                match command_specifier {
                    CommandSpecifier::AbortTransfer => Ok(Self {
                        direction,
                        node_id,
                        frame_data: SdoFrameData::AbortTransfer {
                            index,
                            sub_index,
                            abort_code: u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
                        },
                    }),
                    CommandSpecifier::Server(ServerCommandSpecifier::InitiateDownloadResponse) => {
                        Ok(Self {
                            direction,
                            node_id,
                            frame_data: SdoFrameData::InitiateDownloadResponse { index, sub_index },
                        })
                    }
                    CommandSpecifier::Client(ClientCommandSpecifier::InitiateUploadRequest) => {
                        Ok(Self {
                            direction,
                            node_id,
                            frame_data: SdoFrameData::InitiateUploadRequest { index, sub_index },
                        })
                    }
                    _ => {
                        let transfer_type = SdoFrameTransferType::new_with_bytes(bytes)?;
                        match command_specifier {
                            CommandSpecifier::Client(
                                ClientCommandSpecifier::InitiateDownloadRequest,
                            ) => Ok(Self {
                                direction,
                                node_id,
                                frame_data: SdoFrameData::InitiateUploadResponse {
                                    index,
                                    sub_index,
                                    transfer_type,
                                },
                            }),
                            CommandSpecifier::Server(
                                ServerCommandSpecifier::InitiateUploadResponse,
                            ) => Ok(Self {
                                direction,
                                node_id,
                                frame_data: SdoFrameData::InitiateUploadResponse {
                                    index,
                                    sub_index,
                                    transfer_type,
                                },
                            }),
                            _ => Err(Error::NotImplemented),
                        }
                    }
                }
            }
            _ => Err(Error::NotImplemented),
        }
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
        self.frame_data.to_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /*
        #[test]
        fn test_ccs_from_num() {
            assert_eq!(
                ClientCommandSpecifier::from_num(0),
                Ok(ClientCommandSpecifier::DownloadSegmentRequest)
            );
            assert_eq!(
                ClientCommandSpecifier::from_num(1),
                Ok(ClientCommandSpecifier::InitiateDownloadRequest)
            );
            assert_eq!(
                ClientCommandSpecifier::from_num(2),
                Ok(ClientCommandSpecifier::InitiateUploadRequest)
            );
            assert_eq!(
                ClientCommandSpecifier::from_num(3),
                Ok(ClientCommandSpecifier::UploadSegmentRequest)
            );
            assert_eq!(
                ClientCommandSpecifier::from_num(4),
                Ok(ClientCommandSpecifier::AbortTransfer)
            );
            assert_eq!(
                ClientCommandSpecifier::from_num(5),
                Ok(ClientCommandSpecifier::BlockUpload)
            );
            assert_eq!(
                ClientCommandSpecifier::from_num(6),
                Ok(ClientCommandSpecifier::BlockDownload)
            );
            assert_eq!(
                ClientCommandSpecifier::from_num(7),
                Err(Error::InvalidClientCommandSpecifier(7))
            );
            assert_eq!(
                ClientCommandSpecifier::from_num(8),
                Err(Error::InvalidClientCommandSpecifier(8))
            );
            assert_eq!(
                ClientCommandSpecifier::from_num(255),
                Err(Error::InvalidClientCommandSpecifier(255))
            );
        }

        #[test]
        fn test_sdo_read_frame() {
            let frame = SdoFrame::new_sdo_read_frame(1.try_into().unwrap(), 0x1018, 2); // Product code
            assert_eq!(
                frame,
                SdoFrame {
                    direction: Direction::Rx,
                    ccs: ClientCommandSpecifier::InitiateUploadRequest,
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
                    ccs: ClientCommandSpecifier::InitiateDownloadRequest,
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
                    ccs: ClientCommandSpecifier::InitiateDownloadRequest,
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
                    ccs: ClientCommandSpecifier::InitiateDownloadRequest,
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
                ),
                Ok(SdoFrame {
                    direction: Direction::Rx,
                    ccs: ClientCommandSpecifier::InitiateUploadRequest,
                    node_id: 1.try_into().unwrap(),
                    index: 0x1018,
                    sub_index: 2,
                    size: None,
                    expedited: false,
                    data: vec![],
                })
            );
            assert_eq!(
                SdoFrame::new_with_bytes(
                    Direction::Rx,
                    1.try_into().unwrap(),
                    &[0x2F, 0x02, 0x14, 0x02, 0xFF, 0x00, 0x00, 0x00],
                ),
                Ok(SdoFrame {
                    direction: Direction::Rx,
                    ccs: ClientCommandSpecifier::InitiateDownloadRequest,
                    node_id: 1.try_into().unwrap(),
                    index: 0x1402,
                    sub_index: 2,
                    size: Some(1),
                    expedited: true,
                    data: vec![0xFF],
                })
            );
            assert_eq!(
                SdoFrame::new_with_bytes(
                    Direction::Rx,
                    2.try_into().unwrap(),
                    &[0x2B, 0x17, 0x10, 0x00, 0xE8, 0x03, 0x00, 0x00],
                ),
                Ok(SdoFrame {
                    direction: Direction::Rx,
                    ccs: ClientCommandSpecifier::InitiateDownloadRequest,
                    node_id: 2.try_into().unwrap(),
                    index: 0x1017,
                    sub_index: 0,
                    size: Some(2),
                    expedited: true,
                    data: vec![0xE8, 0x03],
                })
            );
            assert_eq!(
                SdoFrame::new_with_bytes(
                    Direction::Rx,
                    3.try_into().unwrap(),
                    &[0x23, 0x00, 0x12, 0x01, 0x0A, 0x06, 0x00, 0x00],
                ),
                Ok(SdoFrame {
                    direction: Direction::Rx,
                    ccs: ClientCommandSpecifier::InitiateDownloadRequest,
                    node_id: 3.try_into().unwrap(),
                    index: 0x1200,
                    sub_index: 1,
                    size: Some(4),
                    expedited: true,
                    data: vec![0x0A, 0x06, 0x00, 0x00],
                })
            );
            assert_eq!(
                SdoFrame::new_with_bytes(
                    Direction::Tx,
                    4.try_into().unwrap(),
                    &[0x43, 0x00, 0x10, 0x00, 0x92, 0x01, 0x02, 0x00],
                ),
                Ok(SdoFrame {
                    direction: Direction::Tx,
                    ccs: ClientCommandSpecifier::InitiateUploadRequest,
                    node_id: 4.try_into().unwrap(),
                    index: 0x1000,
                    sub_index: 0,
                    size: Some(4),
                    expedited: true,
                    data: vec![0x92, 0x01, 0x02, 0x00],
                })
            );
            assert_eq!(
                SdoFrame::new_with_bytes(
                    Direction::Tx,
                    5.try_into().unwrap(),
                    &[0x80, 0x00, 0x10, 0x00, 0x02, 0x00, 0x01, 0x06],
                ),
                Ok(SdoFrame {
                    direction: Direction::Tx,
                    ccs: ClientCommandSpecifier::AbortTransfer,
                    node_id: 5.try_into().unwrap(),
                    index: 0x1000,
                    sub_index: 0,
                    size: None,
                    expedited: false,
                    data: vec![0x02, 0x00, 0x01, 0x06],
                })
            );
        }

        #[test]
        fn test_communication_object() {
            let frame = SdoFrame {
                direction: Direction::Rx,
                ccs: ClientCommandSpecifier::InitiateUploadRequest,
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
                ccs: ClientCommandSpecifier::InitiateDownloadRequest,
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
                ccs: ClientCommandSpecifier::InitiateUploadRequest,
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
                ccs: ClientCommandSpecifier::InitiateUploadRequest,
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
                ccs: ClientCommandSpecifier::InitiateDownloadRequest,
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
                ccs: ClientCommandSpecifier::InitiateDownloadRequest,
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
                ccs: ClientCommandSpecifier::InitiateDownloadRequest,
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
                ccs: ClientCommandSpecifier::InitiateUploadRequest,
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
    */
}
