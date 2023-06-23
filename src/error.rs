//use thiserror::Error;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("Invalid Node ID ({})", .0)]
    InvalidNodeId(u8),
}

pub type Result<T> = std::result::Result<T, Error>;
