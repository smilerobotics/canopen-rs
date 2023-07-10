use canopen_rs::frame::{NmtCommand, NmtNodeControlAddress};
use canopen_rs::FrameHandler;
use canopen_rs::SocketCanInterface;

const INTERFACE_NAME: &str = "can0";
const NODE_ID: u8 = 1;

#[tokio::main]
async fn main() {
    let interface = SocketCanInterface::new(INTERFACE_NAME);

    let mut frame_handler = FrameHandler::new(interface);

    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    frame_handler
        .nmt_node_control(
            NmtCommand::ResetCommunication,
            NmtNodeControlAddress::AllNodes,
        )
        .await
        .expect("Failed to send NMT node control frame");

    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    let result = frame_handler
        .sdo_read(NODE_ID.try_into().unwrap(), 0x1018, 2)
        .await
        .expect("Failed to read value via SDO");

    println!("received: {:?}", result);
}
