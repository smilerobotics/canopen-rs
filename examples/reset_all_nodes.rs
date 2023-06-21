use socketcan::{BlockingCan, CanSocket, Socket};

use canopen_rs::frame::{CANOpenFrame, NMTCommand, NMTNodeControlAddress, NMTNodeControlFrame};

const INTERFACE_NAME: &str = "can0";

fn main() {
    let mut sock = CanSocket::open(INTERFACE_NAME).unwrap();
    sock.transmit(
        &NMTNodeControlFrame::new(NMTCommand::ResetNode, NMTNodeControlAddress::AllNodes)
            .to_socketcan_frame(),
    )
    .unwrap();
}
