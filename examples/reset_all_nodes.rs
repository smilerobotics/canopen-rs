use socketcan::{BlockingCan, CanSocket, Socket};

use canopen_rs::frame::{CANOpenFrame, NMTCommand, NMTNodeControlAddress};

const INTERFACE_NAME: &str = "can0";

fn main() {
    let mut sock = CanSocket::open(INTERFACE_NAME).unwrap();
    sock.transmit(
        &CANOpenFrame::new_nmt_node_control_frame(
            NMTCommand::ResetNode,
            NMTNodeControlAddress::AllNodes,
        )
        .into(),
    )
    .unwrap();
}
