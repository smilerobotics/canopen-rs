use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use futures::channel::oneshot::{self, Receiver, Sender};
//use futures::lock::Mutex;
use tokio::sync::Mutex;

use crate::error::{Error, Result};
use crate::frame::CanOpenFrame;
use crate::frame::SdoFrame;
use crate::frame::{NmtCommand, NmtNodeControlAddress, NmtNodeControlFrame};
use crate::id::NodeId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ObjectDictionaryAddress {
    node_id: NodeId,
    index: u16,
    sub_index: u8,
}

#[async_trait]
pub trait CanInterface {
    async fn send_frame(&self, frame: CanOpenFrame) -> Result<()>;
    async fn wait_for_frame(&self) -> Result<CanOpenFrame>;
}

pub struct FrameHandler<I> {
    interface: Arc<I>,
    waiting_table: Arc<Mutex<HashMap<ObjectDictionaryAddress, Sender<std::vec::Vec<u8>>>>>,
}

impl<I> FrameHandler<I>
where
    I: Send + Sync + CanInterface + 'static,
{
    pub fn new(interface: I) -> Self {
        let interface = Arc::new(interface);
        let waiting_table = Arc::new(Mutex::new(HashMap::new()));

        let _ = FrameReceiver::new(Arc::clone(&interface), Arc::clone(&waiting_table));

        Self {
            interface,
            waiting_table,
        }
    }

    async fn add_waiting_item(
        &self,
        node_id: NodeId,
        index: u16,
        sub_index: u8,
    ) -> Receiver<Vec<u8>> {
        let (response_sender, response_receiver) = oneshot::channel();
        self.waiting_table.clone().lock_owned().await.insert(
            ObjectDictionaryAddress {
                node_id,
                index,
                sub_index,
            },
            response_sender,
        );
        response_receiver
    }

    pub async fn nmt_node_control(
        &self,
        command: NmtCommand,
        address: NmtNodeControlAddress,
    ) -> Result<()> {
        let frame = NmtNodeControlFrame::new(command, address);
        self.interface.send_frame(frame.into()).await
    }

    pub async fn sdo_read(
        &mut self,
        node_id: NodeId,
        index: u16,
        sub_index: u8,
    ) -> Result<std::vec::Vec<u8>> {
        let response_receiver = self.add_waiting_item(node_id, index, sub_index).await;

        let request_frame = SdoFrame::new_sdo_read_frame(node_id, index, sub_index);
        self.interface.send_frame(request_frame.into()).await?;

        response_receiver.await.or(Err(Error::NotImplemented))
    }
}

struct FrameReceiver;

impl FrameReceiver {
    pub fn new<I: Send + Sync + CanInterface + 'static>(
        interface: Arc<I>,
        waiting_table: Arc<Mutex<HashMap<ObjectDictionaryAddress, Sender<std::vec::Vec<u8>>>>>,
    ) {
        tokio::spawn(async move {
            loop {
                let frame = interface.wait_for_frame().await.unwrap();
                if let CanOpenFrame::SdoFrame(frame) = frame {
                    if let Some(sender) =
                        waiting_table.lock().await.remove(&ObjectDictionaryAddress {
                            node_id: frame.node_id,
                            index: frame.index,
                            sub_index: frame.sub_index,
                        })
                    {
                        sender.send(frame.data).unwrap();
                    }
                } else {
                    println!("received: {:?}", frame);
                }
            }
        });
    }
}
