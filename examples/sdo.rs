use socketcan::{BlockingCan, CanSocket, Socket};

use canopen_rs::frame::{CanOpenFrame, NmtCommand, NmtNodeControlAddress};

const INTERFACE_NAME: &str = "can0";
const NODE_ID: u8 = 1;

fn main() {
    let mut sock = CanSocket::open(INTERFACE_NAME).unwrap();
    sock.transmit(
        &CanOpenFrame::new_nmt_node_control_frame(
            NmtCommand::ResetCommunication,
            NmtNodeControlAddress::AllNodes,
        )
        .into(),
    )
    .unwrap();

    let frame: CanOpenFrame = sock.receive().unwrap().try_into().unwrap();
    println!("received: {:?}", frame);

    let frame = CanOpenFrame::new_sdo_read_frame(NODE_ID.try_into().unwrap(), 0x1018, 2); // read `Product code`
    println!("send: {:?}", frame);
    sock.transmit(&frame.into()).unwrap();

    let frame: CanOpenFrame = sock.receive().unwrap().try_into().unwrap();
    println!("received: {:?}", frame);
}
