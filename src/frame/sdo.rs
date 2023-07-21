use crate::error::{Error, Result};
use crate::frame::{CanOpenFrame, ConvertibleFrame};
use crate::id::{CommunicationObject, NodeId};

const SDO_FRAME_DATA_SIZE: usize = 8;

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
    const BIT_MASK: u8 = 0b1110_0000;
    const BIT_OFFSET: usize = 5;

    fn new(direction: Direction, value: u8) -> Result<Self> {
        if value == 4 {
            return Ok(Self::AbortTransfer);
        }
        match direction {
            Direction::Tx => Ok(Self::Server(ServerCommandSpecifier::new(value)?)),
            Direction::Rx => Ok(Self::Client(ClientCommandSpecifier::new(value)?)),
        }
    }

    fn new_with_byte(direction: Direction, byte: u8) -> Result<Self> {
        Self::new(direction, (byte & Self::BIT_MASK) >> Self::BIT_OFFSET)
    }

    fn as_num(&self) -> u8 {
        match self {
            Self::AbortTransfer => 4,
            Self::Client(ccs) => *ccs as u8,
            Self::Server(scs) => *scs as u8,
        }
    }

    fn as_byte_fragment(&self) -> u8 {
        (self.as_num() << Self::BIT_OFFSET) & Self::BIT_MASK
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
    fn new(value: u8) -> Result<Self> {
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
    fn new(value: u8) -> Result<Self> {
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
enum SdoTransferType {
    #[allow(dead_code)]
    Normal { size: usize },
    Expedited {
        sized: bool,
        data: std::vec::Vec<u8>,
    },
}

impl SdoTransferType {
    const BIT_MASK_TRANSFER_TYPE: u8 = 0b0000_0010;
    const BIT_MASK_SIZE_INDICATOR: u8 = 0b0000_0001;
    const BIT_MASK_VOID_BYTES: u8 = 0b0000_1100;
    const BIT_OFFSET_VOID_BYTES: usize = 2;
    const BIT_EXPEDITED_TRANSFER: u8 = 0b0000_0010;
    const MAX_DATA_BYTES: usize = 4;
    const DATA_START_POS: usize = 4;

    fn new_with_bytes(bytes: &[u8]) -> Result<Self> {
        match bytes[0] & Self::BIT_MASK_TRANSFER_TYPE {
            0 => match bytes[0] & Self::BIT_MASK_SIZE_INDICATOR {
                0 => Err(Error::NotImplemented),
                _ => {
                    if bytes.len() < SDO_FRAME_DATA_SIZE {
                        Err(Error::InvalidDataLength {
                            length: bytes.len(),
                            data_type: "SdoFrame".to_owned(),
                        })
                    } else {
                        Ok(Self::Normal {
                            size: u32::from_le_bytes(
                                bytes[Self::DATA_START_POS..SDO_FRAME_DATA_SIZE]
                                    .try_into()
                                    .unwrap(),
                            ) as usize,
                        })
                    }
                }
            },
            _ => {
                let sized = bytes[0] & Self::BIT_MASK_SIZE_INDICATOR != 0;
                let data_end_pos = match sized {
                    true => {
                        SDO_FRAME_DATA_SIZE
                            - (((bytes[0] & Self::BIT_MASK_VOID_BYTES)
                                >> Self::BIT_OFFSET_VOID_BYTES)
                                as usize)
                    }
                    false => SDO_FRAME_DATA_SIZE,
                };
                if bytes.len() < data_end_pos {
                    Err(Error::InvalidDataLength {
                        length: bytes.len(),
                        data_type: "SdoFrame".to_owned(),
                    })
                } else {
                    Ok(Self::Expedited {
                        sized,
                        data: bytes[Self::DATA_START_POS..data_end_pos].to_owned(),
                    })
                }
            }
        }
    }

    fn as_first_byte_fragment(&self) -> u8 {
        match self {
            Self::Normal { size: _ } => 0x01,
            Self::Expedited { sized, data } => {
                (match sized {
                    true => {
                        (((Self::MAX_DATA_BYTES - data.len()) as u8) << Self::BIT_OFFSET_VOID_BYTES)
                            & Self::BIT_MASK_VOID_BYTES
                    }
                    false => 0x00,
                } | Self::BIT_EXPEDITED_TRANSFER)
                    | (*sized as u8)
            }
        }
    }

    fn as_data_bytes(&self) -> [u8; Self::MAX_DATA_BYTES] {
        match self {
            SdoTransferType::Normal { size } => (*size as u32).to_le_bytes(),
            SdoTransferType::Expedited { sized: _, data } => {
                let mut buf = [0u8; Self::MAX_DATA_BYTES];
                buf[..data.len()].copy_from_slice(data);
                buf
            }
        }
    }
}

#[derive(Debug, PartialEq)]
struct SdoSegmentData(std::vec::Vec<u8>);

impl SdoSegmentData {
    const BIT_MASK_VOID_BYTES: u8 = 0b0000_1110;
    const BIT_OFFSET_VOID_BYTES: usize = 1;
    const MAX_DATA_BYTES: usize = 7;

    fn as_first_byte_fragment(&self) -> u8 {
        (((Self::MAX_DATA_BYTES - self.0.len()) as u8) << Self::BIT_OFFSET_VOID_BYTES)
            & Self::BIT_MASK_VOID_BYTES
    }
}

impl std::convert::AsRef<[u8]> for SdoSegmentData {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SdoSegmentToggle(bool);

impl SdoSegmentToggle {
    const BIT_OFFSET: usize = 4;

    fn as_first_byte_fragment(&self) -> u8 {
        (self.0 as u8) << Self::BIT_OFFSET
    }
}

#[derive(Debug, PartialEq)]
enum SdoCommand {
    InitiateDownloadRequest {
        index: u16,
        sub_index: u8,
        transfer_type: SdoTransferType,
    },
    InitiateDownloadResponse {
        index: u16,
        sub_index: u8,
    },
    #[allow(dead_code)]
    DownloadSegmentRequest {
        toggle: SdoSegmentToggle,
        data: SdoSegmentData,
        continued: bool,
    },
    #[allow(dead_code)]
    DownloadSegmentResponse {
        toggle: SdoSegmentToggle,
    },
    InitiateUploadRequest {
        index: u16,
        sub_index: u8,
    },
    InitiateUploadResponse {
        index: u16,
        sub_index: u8,
        transfer_type: SdoTransferType,
    },
    #[allow(dead_code)]
    UploadSegmentRequest {
        toggle: SdoSegmentToggle,
    },
    #[allow(dead_code)]
    UploadSegmentResponse {
        toggle: SdoSegmentToggle,
        data: SdoSegmentData,
        continued: bool,
    },
    AbortTransfer {
        index: u16,
        sub_index: u8,
        abort_code: u32,
    },
}

impl SdoCommand {
    fn as_bytes(&self) -> std::vec::Vec<u8> {
        // cf. https://en.wikipedia.org/wiki/CANopen#Service_Data_Object_(SDO)_protocol
        let mut buf = std::vec::Vec::with_capacity(SDO_FRAME_DATA_SIZE);
        match self {
            SdoCommand::AbortTransfer {
                index,
                sub_index,
                abort_code,
            } => {
                buf.push(CommandSpecifier::AbortTransfer.as_byte_fragment());
                buf.extend_from_slice(&index.to_le_bytes());
                buf.push(*sub_index);
                buf.extend_from_slice(&abort_code.to_le_bytes())
            }
            SdoCommand::InitiateDownloadRequest {
                index,
                sub_index,
                transfer_type,
            } => {
                buf.push(
                    CommandSpecifier::Client(ClientCommandSpecifier::InitiateDownloadRequest)
                        .as_byte_fragment()
                        | transfer_type.as_first_byte_fragment(),
                );
                buf.extend_from_slice(&index.to_le_bytes());
                buf.push(*sub_index);
                buf.extend_from_slice(&transfer_type.as_data_bytes());
            }
            SdoCommand::InitiateDownloadResponse { index, sub_index } => {
                buf.push(
                    CommandSpecifier::Server(ServerCommandSpecifier::InitiateDownloadResponse)
                        .as_byte_fragment(),
                );
                buf.extend_from_slice(&index.to_le_bytes());
                buf.push(*sub_index);
            }
            SdoCommand::DownloadSegmentRequest {
                toggle,
                data,
                continued,
            } => {
                buf.push(
                    CommandSpecifier::Client(ClientCommandSpecifier::DownloadSegmentRequest)
                        .as_byte_fragment()
                        | toggle.as_first_byte_fragment()
                        | data.as_first_byte_fragment()
                        | (*continued as u8),
                );
                buf.extend_from_slice(data.as_ref());
            }
            SdoCommand::DownloadSegmentResponse { toggle } => {
                buf.push(
                    CommandSpecifier::Server(ServerCommandSpecifier::DownloadSegmentResponse)
                        .as_byte_fragment()
                        | toggle.as_first_byte_fragment(),
                );
            }
            SdoCommand::InitiateUploadRequest { index, sub_index } => {
                buf.push(
                    CommandSpecifier::Client(ClientCommandSpecifier::InitiateUploadRequest)
                        .as_byte_fragment(),
                );
                buf.extend_from_slice(&index.to_le_bytes());
                buf.push(*sub_index);
            }
            SdoCommand::InitiateUploadResponse {
                index,
                sub_index,
                transfer_type,
            } => {
                buf.push(
                    CommandSpecifier::Server(ServerCommandSpecifier::InitiateUploadResponse)
                        .as_byte_fragment()
                        | transfer_type.as_first_byte_fragment(),
                );
                buf.extend_from_slice(&index.to_le_bytes());
                buf.push(*sub_index);
                buf.extend_from_slice(&transfer_type.as_data_bytes());
            }
            SdoCommand::UploadSegmentRequest { toggle } => {
                buf.push(
                    CommandSpecifier::Client(ClientCommandSpecifier::UploadSegmentRequest)
                        .as_byte_fragment()
                        | toggle.as_first_byte_fragment(),
                );
            }
            SdoCommand::UploadSegmentResponse {
                toggle,
                data,
                continued,
            } => {
                buf.push(
                    CommandSpecifier::Server(ServerCommandSpecifier::UploadSegmentResponse)
                        .as_byte_fragment()
                        | toggle.as_first_byte_fragment()
                        | data.as_first_byte_fragment()
                        | (*continued as u8),
                );
                buf.extend_from_slice(data.as_ref());
            }
        }
        buf.resize(SDO_FRAME_DATA_SIZE, 0x00);
        buf
    }
}

#[derive(Debug, PartialEq)]
pub struct SdoFrame {
    direction: Direction,
    node_id: NodeId,
    command: SdoCommand,
}

impl SdoFrame {
    pub fn new_sdo_read_frame(node_id: NodeId, index: u16, sub_index: u8) -> Self {
        Self {
            direction: Direction::Rx,
            node_id,
            command: SdoCommand::InitiateUploadRequest { index, sub_index },
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
            command: SdoCommand::InitiateDownloadRequest {
                index,
                sub_index,
                transfer_type: SdoTransferType::Expedited { sized: true, data },
            },
        }
    }

    pub(crate) fn new_with_bytes(
        direction: Direction,
        node_id: NodeId,
        bytes: &[u8],
    ) -> Result<Self> {
        if bytes.len() < SDO_FRAME_DATA_SIZE {
            return Err(Error::InvalidDataLength {
                length: bytes.len(),
                data_type: "SdoFrame".to_owned(),
            });
        }
        // cf. https://en.wikipedia.org/wiki/CANopen#Service_Data_Object_(SDO)_protocol
        let command_specifier = CommandSpecifier::new_with_byte(direction, bytes[0])?;
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
                        command: SdoCommand::AbortTransfer {
                            index,
                            sub_index,
                            abort_code: u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
                        },
                    }),
                    CommandSpecifier::Server(ServerCommandSpecifier::InitiateDownloadResponse) => {
                        Ok(Self {
                            direction,
                            node_id,
                            command: SdoCommand::InitiateDownloadResponse { index, sub_index },
                        })
                    }
                    CommandSpecifier::Client(ClientCommandSpecifier::InitiateUploadRequest) => {
                        Ok(Self {
                            direction,
                            node_id,
                            command: SdoCommand::InitiateUploadRequest { index, sub_index },
                        })
                    }
                    _ => {
                        let transfer_type = SdoTransferType::new_with_bytes(bytes)?;
                        match command_specifier {
                            CommandSpecifier::Client(
                                ClientCommandSpecifier::InitiateDownloadRequest,
                            ) => Ok(Self {
                                direction,
                                node_id,
                                command: SdoCommand::InitiateDownloadRequest {
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
                                command: SdoCommand::InitiateUploadResponse {
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
        self.command.as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_specifier_new() {
        assert_eq!(
            CommandSpecifier::new(Direction::Rx, 0).unwrap(),
            CommandSpecifier::Client(ClientCommandSpecifier::DownloadSegmentRequest)
        );
        assert_eq!(
            CommandSpecifier::new(Direction::Rx, 1).unwrap(),
            CommandSpecifier::Client(ClientCommandSpecifier::InitiateDownloadRequest)
        );
        assert_eq!(
            CommandSpecifier::new(Direction::Rx, 2).unwrap(),
            CommandSpecifier::Client(ClientCommandSpecifier::InitiateUploadRequest)
        );
        assert_eq!(
            CommandSpecifier::new(Direction::Rx, 3).unwrap(),
            CommandSpecifier::Client(ClientCommandSpecifier::UploadSegmentRequest)
        );
        assert_eq!(
            CommandSpecifier::new(Direction::Rx, 4).unwrap(),
            CommandSpecifier::AbortTransfer
        );
        assert_eq!(
            CommandSpecifier::new(Direction::Rx, 5).unwrap(),
            CommandSpecifier::Client(ClientCommandSpecifier::BlockUpload)
        );
        assert_eq!(
            CommandSpecifier::new(Direction::Rx, 6).unwrap(),
            CommandSpecifier::Client(ClientCommandSpecifier::BlockDownload)
        );
        assert!(CommandSpecifier::new(Direction::Rx, 7).is_err());
        assert!(CommandSpecifier::new(Direction::Rx, 8).is_err());

        assert_eq!(
            CommandSpecifier::new(Direction::Tx, 0).unwrap(),
            CommandSpecifier::Server(ServerCommandSpecifier::UploadSegmentResponse)
        );
        assert_eq!(
            CommandSpecifier::new(Direction::Tx, 1).unwrap(),
            CommandSpecifier::Server(ServerCommandSpecifier::DownloadSegmentResponse)
        );
        assert_eq!(
            CommandSpecifier::new(Direction::Tx, 2).unwrap(),
            CommandSpecifier::Server(ServerCommandSpecifier::InitiateUploadResponse)
        );
        assert_eq!(
            CommandSpecifier::new(Direction::Tx, 3).unwrap(),
            CommandSpecifier::Server(ServerCommandSpecifier::InitiateDownloadResponse)
        );
        assert_eq!(
            CommandSpecifier::new(Direction::Tx, 4).unwrap(),
            CommandSpecifier::AbortTransfer
        );
        assert_eq!(
            CommandSpecifier::new(Direction::Tx, 5).unwrap(),
            CommandSpecifier::Server(ServerCommandSpecifier::BlockDownload)
        );
        assert_eq!(
            CommandSpecifier::new(Direction::Tx, 6).unwrap(),
            CommandSpecifier::Server(ServerCommandSpecifier::BlockUpload)
        );
        assert!(CommandSpecifier::new(Direction::Tx, 7).is_err());
        assert!(CommandSpecifier::new(Direction::Tx, 8).is_err());
    }

    #[test]
    fn test_command_specifier_new_with_byte() {
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Rx, 0x00).unwrap(),
            CommandSpecifier::Client(ClientCommandSpecifier::DownloadSegmentRequest)
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Rx, 0x01).unwrap(),
            CommandSpecifier::Client(ClientCommandSpecifier::DownloadSegmentRequest)
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Rx, 0x10).unwrap(),
            CommandSpecifier::Client(ClientCommandSpecifier::DownloadSegmentRequest)
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Rx, 0x17).unwrap(),
            CommandSpecifier::Client(ClientCommandSpecifier::DownloadSegmentRequest)
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Rx, 0x21).unwrap(),
            CommandSpecifier::Client(ClientCommandSpecifier::InitiateDownloadRequest)
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Rx, 0x22).unwrap(),
            CommandSpecifier::Client(ClientCommandSpecifier::InitiateDownloadRequest)
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Rx, 0x23).unwrap(),
            CommandSpecifier::Client(ClientCommandSpecifier::InitiateDownloadRequest)
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Rx, 0x2B).unwrap(),
            CommandSpecifier::Client(ClientCommandSpecifier::InitiateDownloadRequest)
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Rx, 0x2F).unwrap(),
            CommandSpecifier::Client(ClientCommandSpecifier::InitiateDownloadRequest)
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Rx, 0x40).unwrap(),
            CommandSpecifier::Client(ClientCommandSpecifier::InitiateUploadRequest)
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Rx, 0x50).unwrap(),
            CommandSpecifier::Client(ClientCommandSpecifier::InitiateUploadRequest)
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Rx, 0x60).unwrap(),
            CommandSpecifier::Client(ClientCommandSpecifier::UploadSegmentRequest)
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Rx, 0x70).unwrap(),
            CommandSpecifier::Client(ClientCommandSpecifier::UploadSegmentRequest)
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Rx, 0x80).unwrap(),
            CommandSpecifier::AbortTransfer
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Rx, 0xA0).unwrap(),
            CommandSpecifier::Client(ClientCommandSpecifier::BlockUpload)
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Rx, 0xC0).unwrap(),
            CommandSpecifier::Client(ClientCommandSpecifier::BlockDownload)
        );
        assert!(CommandSpecifier::new_with_byte(Direction::Rx, 0xE0).is_err());

        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Tx, 0x00).unwrap(),
            CommandSpecifier::Server(ServerCommandSpecifier::UploadSegmentResponse)
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Tx, 0x01).unwrap(),
            CommandSpecifier::Server(ServerCommandSpecifier::UploadSegmentResponse)
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Tx, 0x10).unwrap(),
            CommandSpecifier::Server(ServerCommandSpecifier::UploadSegmentResponse)
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Tx, 0x17).unwrap(),
            CommandSpecifier::Server(ServerCommandSpecifier::UploadSegmentResponse)
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Tx, 0x20).unwrap(),
            CommandSpecifier::Server(ServerCommandSpecifier::DownloadSegmentResponse)
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Tx, 0x41).unwrap(),
            CommandSpecifier::Server(ServerCommandSpecifier::InitiateUploadResponse)
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Tx, 0x42).unwrap(),
            CommandSpecifier::Server(ServerCommandSpecifier::InitiateUploadResponse)
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Tx, 0x43).unwrap(),
            CommandSpecifier::Server(ServerCommandSpecifier::InitiateUploadResponse)
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Tx, 0x4B).unwrap(),
            CommandSpecifier::Server(ServerCommandSpecifier::InitiateUploadResponse)
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Tx, 0x4F).unwrap(),
            CommandSpecifier::Server(ServerCommandSpecifier::InitiateUploadResponse)
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Tx, 0x60).unwrap(),
            CommandSpecifier::Server(ServerCommandSpecifier::InitiateDownloadResponse)
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Tx, 0x80).unwrap(),
            CommandSpecifier::AbortTransfer
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Tx, 0xA0).unwrap(),
            CommandSpecifier::Server(ServerCommandSpecifier::BlockDownload)
        );
        assert_eq!(
            CommandSpecifier::new_with_byte(Direction::Tx, 0xC0).unwrap(),
            CommandSpecifier::Server(ServerCommandSpecifier::BlockUpload)
        );
        assert!(CommandSpecifier::new_with_byte(Direction::Tx, 0xE0).is_err());
    }

    #[test]
    fn test_command_specifier_as_byte_fragment() {
        assert_eq!(
            CommandSpecifier::Client(ClientCommandSpecifier::DownloadSegmentRequest)
                .as_byte_fragment(),
            0x00
        );
        assert_eq!(
            CommandSpecifier::Client(ClientCommandSpecifier::InitiateDownloadRequest)
                .as_byte_fragment(),
            0x20
        );
        assert_eq!(
            CommandSpecifier::Client(ClientCommandSpecifier::InitiateUploadRequest)
                .as_byte_fragment(),
            0x40
        );
        assert_eq!(
            CommandSpecifier::Client(ClientCommandSpecifier::UploadSegmentRequest)
                .as_byte_fragment(),
            0x60
        );
        assert_eq!(
            CommandSpecifier::Client(ClientCommandSpecifier::BlockUpload).as_byte_fragment(),
            0xA0
        );
        assert_eq!(
            CommandSpecifier::Client(ClientCommandSpecifier::BlockDownload).as_byte_fragment(),
            0xC0
        );

        assert_eq!(
            CommandSpecifier::Server(ServerCommandSpecifier::UploadSegmentResponse)
                .as_byte_fragment(),
            0x00
        );
        assert_eq!(
            CommandSpecifier::Server(ServerCommandSpecifier::DownloadSegmentResponse)
                .as_byte_fragment(),
            0x20
        );
        assert_eq!(
            CommandSpecifier::Server(ServerCommandSpecifier::InitiateUploadResponse)
                .as_byte_fragment(),
            0x40
        );
        assert_eq!(
            CommandSpecifier::Server(ServerCommandSpecifier::InitiateDownloadResponse)
                .as_byte_fragment(),
            0x60
        );
        assert_eq!(
            CommandSpecifier::Server(ServerCommandSpecifier::BlockDownload).as_byte_fragment(),
            0xA0
        );
        assert_eq!(
            CommandSpecifier::Server(ServerCommandSpecifier::BlockUpload).as_byte_fragment(),
            0xC0
        );

        assert_eq!(CommandSpecifier::AbortTransfer.as_byte_fragment(), 0x80);
    }

    #[test]
    fn test_sdo_transfer_type_new_with_bytes() {
        assert!(
            SdoTransferType::new_with_bytes(&[0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
                .is_err()
        );
        assert_eq!(
            SdoTransferType::new_with_bytes(&[0x21, 0x01, 0x02, 0x03, 0x45, 0x23, 0x01, 0x00])
                .unwrap(),
            SdoTransferType::Normal { size: 74565 },
        );
        assert_eq!(
            SdoTransferType::new_with_bytes(&[0x22, 0x01, 0x02, 0x03, 0x12, 0x34, 0x56, 0x78])
                .unwrap(),
            SdoTransferType::Expedited {
                sized: false,
                data: vec![0x12, 0x34, 0x56, 0x78],
            },
        );
        assert_eq!(
            SdoTransferType::new_with_bytes(&[0x23, 0x01, 0x02, 0x03, 0x12, 0x34, 0x56, 0x78])
                .unwrap(),
            SdoTransferType::Expedited {
                sized: true,
                data: vec![0x12, 0x34, 0x56, 0x78],
            },
        );
        assert_eq!(
            SdoTransferType::new_with_bytes(&[0x2B, 0x01, 0x02, 0x03, 0x12, 0x34, 0x00, 0x00])
                .unwrap(),
            SdoTransferType::Expedited {
                sized: true,
                data: vec![0x12, 0x34],
            },
        );
        assert_eq!(
            SdoTransferType::new_with_bytes(&[0x2F, 0x01, 0x02, 0x03, 0x12, 0x00, 0x00, 0x00])
                .unwrap(),
            SdoTransferType::Expedited {
                sized: true,
                data: vec![0x12],
            },
        );

        assert!(
            SdoTransferType::new_with_bytes(&[0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
                .is_err()
        );
        assert_eq!(
            SdoTransferType::new_with_bytes(&[0x41, 0x01, 0x02, 0x03, 0x45, 0x23, 0x01, 0x00])
                .unwrap(),
            SdoTransferType::Normal { size: 74565 },
        );
        assert_eq!(
            SdoTransferType::new_with_bytes(&[0x42, 0x01, 0x02, 0x03, 0x12, 0x34, 0x56, 0x78])
                .unwrap(),
            SdoTransferType::Expedited {
                sized: false,
                data: vec![0x12, 0x34, 0x56, 0x78],
            },
        );
        assert_eq!(
            SdoTransferType::new_with_bytes(&[0x43, 0x01, 0x02, 0x03, 0x12, 0x34, 0x56, 0x78])
                .unwrap(),
            SdoTransferType::Expedited {
                sized: true,
                data: vec![0x12, 0x34, 0x56, 0x78],
            },
        );
        assert_eq!(
            SdoTransferType::new_with_bytes(&[0x4B, 0x01, 0x02, 0x03, 0x12, 0x34, 0x00, 0x00])
                .unwrap(),
            SdoTransferType::Expedited {
                sized: true,
                data: vec![0x12, 0x34],
            },
        );
        assert_eq!(
            SdoTransferType::new_with_bytes(&[0x4F, 0x01, 0x02, 0x03, 0x12, 0x00, 0x00, 0x00])
                .unwrap(),
            SdoTransferType::Expedited {
                sized: true,
                data: vec![0x12],
            },
        );
    }

    #[test]
    fn test_sdo_transfer_type_as_first_byte_fragment() {
        assert_eq!(
            SdoTransferType::Normal { size: 74565 }.as_first_byte_fragment(),
            0x01
        );
        assert_eq!(
            SdoTransferType::Expedited {
                sized: false,
                data: vec![0x12, 0x34, 0x56, 0x78],
            }
            .as_first_byte_fragment(),
            0x02
        );
        assert_eq!(
            SdoTransferType::Expedited {
                sized: true,
                data: vec![0x12],
            }
            .as_first_byte_fragment(),
            0x0F
        );
        assert_eq!(
            SdoTransferType::Expedited {
                sized: true,
                data: vec![0x12, 0x34],
            }
            .as_first_byte_fragment(),
            0x0B
        );
        assert_eq!(
            SdoTransferType::Expedited {
                sized: true,
                data: vec![0x12, 0x34, 0x56, 0x78],
            }
            .as_first_byte_fragment(),
            0x03
        );
    }

    #[test]
    fn test_sdo_transfer_type_as_data_bytes() {
        assert_eq!(
            SdoTransferType::Normal { size: 74565 }.as_data_bytes(),
            [0x45, 0x23, 0x01, 0x00],
        );
        assert_eq!(
            SdoTransferType::Expedited {
                sized: false,
                data: vec![0x12, 0x34, 0x56, 0x78],
            }
            .as_data_bytes(),
            [0x12, 0x34, 0x56, 0x78]
        );
        assert_eq!(
            SdoTransferType::Expedited {
                sized: true,
                data: vec![0x12, 0x00, 0x00, 0x00],
            }
            .as_data_bytes(),
            [0x12, 0x00, 0x00, 0x00]
        );
        assert_eq!(
            SdoTransferType::Expedited {
                sized: true,
                data: vec![0x12, 0x34, 0x00, 0x00],
            }
            .as_data_bytes(),
            [0x12, 0x34, 0x00, 0x00],
        );
        assert_eq!(
            SdoTransferType::Expedited {
                sized: true,
                data: vec![0x12, 0x34, 0x56, 0x78],
            }
            .as_data_bytes(),
            [0x12, 0x34, 0x56, 0x78]
        );
    }

    #[test]
    fn test_sdo_segment_data_as_first_byte_fragment() {
        assert_eq!(SdoSegmentData(vec![0x01]).as_first_byte_fragment(), 0x0C);
        assert_eq!(
            SdoSegmentData(vec![0x01, 0x23]).as_first_byte_fragment(),
            0x0A
        );
        assert_eq!(
            SdoSegmentData(vec![0x01, 0x23, 0x45]).as_first_byte_fragment(),
            0x08
        );
        assert_eq!(
            SdoSegmentData(vec![0x01, 0x23, 0x45, 0x67]).as_first_byte_fragment(),
            0x06
        );
        assert_eq!(
            SdoSegmentData(vec![0x01, 0x23, 0x45, 0x67, 0x89]).as_first_byte_fragment(),
            0x04
        );
        assert_eq!(
            SdoSegmentData(vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xAB]).as_first_byte_fragment(),
            0x02
        );
        assert_eq!(
            SdoSegmentData(vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD]).as_first_byte_fragment(),
            0x00
        );
    }

    #[test]
    fn test_sdo_segment_toggle_as_first_byte_fragment() {
        assert_eq!(SdoSegmentToggle(true).as_first_byte_fragment(), 0x10);
        assert_eq!(SdoSegmentToggle(false).as_first_byte_fragment(), 0x00);
    }

    #[test]
    fn test_sdo_command_as_bytes() {
        assert_eq!(
            SdoCommand::InitiateDownloadRequest {
                index: 0x2001,
                sub_index: 3,
                transfer_type: SdoTransferType::Expedited {
                    sized: true,
                    data: vec![0x12, 0x34, 0x56, 0x78]
                }
            }
            .as_bytes(),
            vec![0x23, 0x01, 0x20, 0x03, 0x12, 0x34, 0x56, 0x78]
        );
        assert_eq!(
            SdoCommand::InitiateDownloadResponse {
                index: 0x2001,
                sub_index: 3,
            }
            .as_bytes(),
            vec![0x60, 0x01, 0x20, 0x03, 0x00, 0x00, 0x00, 0x00]
        );
        assert_eq!(
            SdoCommand::InitiateUploadRequest {
                index: 0x2001,
                sub_index: 3,
            }
            .as_bytes(),
            vec![0x40, 0x01, 0x20, 0x03, 0x00, 0x00, 0x00, 0x00]
        );
        assert_eq!(
            SdoCommand::InitiateUploadResponse {
                index: 0x2001,
                sub_index: 3,
                transfer_type: SdoTransferType::Expedited {
                    sized: true,
                    data: vec![0x12, 0x34, 0x56, 0x78]
                }
            }
            .as_bytes(),
            vec![0x43, 0x01, 0x20, 0x03, 0x12, 0x34, 0x56, 0x78]
        );
        assert_eq!(
            SdoCommand::AbortTransfer {
                index: 0x2001,
                sub_index: 3,
                abort_code: 0x05040001,
            }
            .as_bytes(),
            vec![0x80, 0x01, 0x20, 0x03, 0x01, 0x00, 0x04, 0x05]
        );
    }

    #[test]
    fn test_sdo_read_frame() {
        assert_eq!(
            SdoFrame::new_sdo_read_frame(1.try_into().unwrap(), 0x1018, 2),
            SdoFrame {
                direction: Direction::Rx,
                node_id: 1.try_into().unwrap(),
                command: SdoCommand::InitiateUploadRequest {
                    index: 0x1018,
                    sub_index: 2
                }
            }
        );
    }

    #[test]
    fn test_sdo_write_frame() {
        assert_eq!(
            SdoFrame::new_sdo_write_frame(1.try_into().unwrap(), 0x1402, 2, vec![255]),
            SdoFrame {
                direction: Direction::Rx,
                node_id: 1.try_into().unwrap(),
                command: SdoCommand::InitiateDownloadRequest {
                    index: 0x1402,
                    sub_index: 2,
                    transfer_type: SdoTransferType::Expedited {
                        sized: true,
                        data: vec![0xFF]
                    }
                }
            }
        );
        assert_eq!(
            SdoFrame::new_sdo_write_frame(
                2.try_into().unwrap(),
                0x1017,
                0,
                1000u16.to_le_bytes().into(),
            ),
            SdoFrame {
                direction: Direction::Rx,
                node_id: 2.try_into().unwrap(),
                command: SdoCommand::InitiateDownloadRequest {
                    index: 0x1017,
                    sub_index: 0,
                    transfer_type: SdoTransferType::Expedited {
                        sized: true,
                        data: vec![0xE8, 0x03],
                    }
                }
            }
        );
        assert_eq!(
            SdoFrame::new_sdo_write_frame(
                3.try_into().unwrap(),
                0x1200,
                1,
                0x060Au32.to_le_bytes().into(),
            ),
            SdoFrame {
                direction: Direction::Rx,
                node_id: 3.try_into().unwrap(),
                command: SdoCommand::InitiateDownloadRequest {
                    index: 0x1200,
                    sub_index: 1,
                    transfer_type: SdoTransferType::Expedited {
                        sized: true,
                        data: vec![0x0A, 0x06, 0x00, 0x00],
                    }
                }
            }
        );
    }

    #[test]
    fn test_new_with_bytes() {
        assert_eq!(
            SdoFrame::new_with_bytes(
                Direction::Rx,
                1.try_into().unwrap(),
                &[0x40, 0x18, 0x10, 0x02, 0x00, 0x00, 0x00, 0x00],
            )
            .unwrap(),
            SdoFrame {
                direction: Direction::Rx,
                node_id: 1.try_into().unwrap(),
                command: SdoCommand::InitiateUploadRequest {
                    index: 0x1018,
                    sub_index: 2
                }
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
                node_id: 1.try_into().unwrap(),
                command: SdoCommand::InitiateDownloadRequest {
                    index: 0x1402,
                    sub_index: 2,
                    transfer_type: SdoTransferType::Expedited {
                        sized: true,
                        data: vec![0xFF]
                    }
                }
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
                node_id: 2.try_into().unwrap(),
                command: SdoCommand::InitiateDownloadRequest {
                    index: 0x1017,
                    sub_index: 0,
                    transfer_type: SdoTransferType::Expedited {
                        sized: true,
                        data: vec![0xE8, 0x03],
                    }
                }
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
                node_id: 3.try_into().unwrap(),
                command: SdoCommand::InitiateDownloadRequest {
                    index: 0x1200,
                    sub_index: 1,
                    transfer_type: SdoTransferType::Expedited {
                        sized: true,
                        data: vec![0x0A, 0x06, 0x00, 0x00],
                    }
                }
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
                node_id: 4.try_into().unwrap(),
                command: SdoCommand::InitiateUploadResponse {
                    index: 0x1000,
                    sub_index: 0,
                    transfer_type: SdoTransferType::Expedited {
                        sized: true,
                        data: vec![0x92, 0x01, 0x02, 0x00],
                    }
                }
            }
        );
        assert_eq!(
            SdoFrame::new_with_bytes(
                Direction::Tx,
                1.try_into().unwrap(),
                &[0x60, 0x02, 0x14, 0x02, 0x00, 0x00, 0x00, 0x00],
            )
            .unwrap(),
            SdoFrame {
                direction: Direction::Tx,
                node_id: 1.try_into().unwrap(),
                command: SdoCommand::InitiateDownloadResponse {
                    index: 0x1402,
                    sub_index: 2,
                }
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
                node_id: 5.try_into().unwrap(),
                command: SdoCommand::AbortTransfer {
                    index: 0x1000,
                    sub_index: 0,
                    abort_code: 0x06010002
                }
            }
        );
    }

    #[test]
    fn test_communication_object() {
        assert_eq!(
            SdoFrame {
                direction: Direction::Rx,
                node_id: 1.try_into().unwrap(),
                command: SdoCommand::InitiateUploadRequest {
                    index: 0x1018,
                    sub_index: 2,
                }
            }
            .communication_object(),
            CommunicationObject::RxSdo(1.try_into().unwrap())
        );
        assert_eq!(
            SdoFrame {
                direction: Direction::Rx,
                node_id: 3.try_into().unwrap(),
                command: SdoCommand::InitiateDownloadRequest {
                    index: 0x1200,
                    sub_index: 1,
                    transfer_type: SdoTransferType::Expedited {
                        sized: true,
                        data: vec![0x0A, 0x06, 0x00, 0x00],
                    }
                }
            }
            .communication_object(),
            CommunicationObject::RxSdo(3.try_into().unwrap())
        );
        /*
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
        */
    }
    /*
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
