use socketcan::{BlockingCan, CanSocket, Socket};

use canopen_rs::frame::{
    CANOpenFrame, NMTCommand, NMTNodeControlAddress, NMTNodeControlFrame, SDOFrame,
};

const INTERFACE_NAME: &str = "can0";

fn main() {
    let mut sock = CanSocket::open(INTERFACE_NAME).unwrap();
    sock.transmit(
        &NMTNodeControlFrame::new(
            NMTCommand::ResetCommunication,
            NMTNodeControlAddress::AllNodes,
        )
        .to_socketcan_frame(),
    )
    .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1000));
    // read `Product code`
    sock.transmit(&SDOFrame::sdo_read_frame(1, 0x1018, 2).to_socketcan_frame())
        .unwrap();
}
