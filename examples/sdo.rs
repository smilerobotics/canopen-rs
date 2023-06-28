use socketcan::{BlockingCan, CanSocket, EmbeddedFrame, Socket};

use canopen_rs::frame::{CANOpenFrame, NMTCommand, NMTNodeControlAddress};
use canopen_rs::id::CommunicationObject;

const INTERFACE_NAME: &str = "can0";
const NODE_ID: u8 = 1;

fn main() {
    let mut sock = CanSocket::open(INTERFACE_NAME).unwrap();
    sock.transmit(
        &CANOpenFrame::new_nmt_node_control_frame(
            NMTCommand::ResetCommunication,
            NMTNodeControlAddress::AllNodes,
        )
        .into(),
    )
    .unwrap();

    let cob: CommunicationObject = sock.receive().unwrap().id().try_into().unwrap();
    println!("received communication object: {:?}", cob);

    sock.transmit(
        &CANOpenFrame::new_sdo_read_frame(NODE_ID.try_into().unwrap(), 0x1018, 2) // read `Product code`
            .into(),
    )
    .unwrap();

    let frame: CANOpenFrame = sock.receive().unwrap().try_into().unwrap();
    println!("received: {:?}", frame);
}
