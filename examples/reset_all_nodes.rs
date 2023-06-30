use socketcan::{BlockingCan, CanSocket, Socket};

use canopen_rs::frame::{CanOpenFrame, NmtCommand, NmtNodeControlAddress};

const INTERFACE_NAME: &str = "can0";

fn main() {
    let mut sock = CanSocket::open(INTERFACE_NAME).unwrap();
    sock.transmit(
        &CanOpenFrame::new_nmt_node_control_frame(
            NmtCommand::ResetNode,
            NmtNodeControlAddress::AllNodes,
        )
        .into(),
    )
    .unwrap();
}
