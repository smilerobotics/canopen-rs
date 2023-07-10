use async_trait::async_trait;
//use futures_util::StreamExt;
use socketcan::async_io::CanSocket;

use crate::error::Result;
use crate::frame::CanOpenFrame;
use crate::CanInterface;

pub struct SocketCanInterface(CanSocket);

impl SocketCanInterface {
    pub fn new(interface_name: &str) -> Self {
        Self(CanSocket::open(interface_name).unwrap())
    }
}

#[async_trait]
impl CanInterface for SocketCanInterface {
    async fn send_frame(&self, frame: CanOpenFrame) -> Result<()> {
        Ok(self
            .0
            .write_frame::<socketcan::CanFrame>(&frame.into())
            .await?)
    }

    async fn wait_for_frame(&self) -> Result<CanOpenFrame> {
        self.0.read_frame().await?.try_into()
    }
}
