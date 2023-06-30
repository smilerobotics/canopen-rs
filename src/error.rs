//use thiserror::Error;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("Invalid Node ID ({})", .0)]
    InvalidNodeId(u8),
    #[error("Invalid COB ID ({:03X})", .0)]
    InvalidCobId(u16),
    #[error("Invalid NMT Command (0x{:02X})", .0)]
    InvalidNmtCommand(u8),
    #[error("Invalid NMT State(0x{:02X})", .0)]
    InvalidNmtState(u8),
    #[error("Invalid data length ({} bytes for {})", .length, .data_type)]
    InvalidDataLength { length: usize, data_type: String },
    #[error("Invalid client command specifier ({})", .0)]
    InvalidClientCommandSpecifier(u8),
    #[error("CAN-FD is not supported")]
    CanFdNotSupported,
    #[error("Not implemented")]
    NotImplemented,
}

pub type Result<T> = std::result::Result<T, Error>;
