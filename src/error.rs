//use thiserror::Error;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("Invalid Node ID ({})", .0)]
    InvalidNodeId(u8),
    #[error("Invalid COB ID ({:03X})", .0)]
    InvalidCobId(u16),
    #[error("CAN-FD is not supported")]
    CanFdNotSupported,
}

pub type Result<T> = std::result::Result<T, Error>;
