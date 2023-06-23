use socketcan::{BlockingCan, CanSocket, Socket};

use canopen_rs::frame::{CANOpenFrame, NMTCommand, NMTNodeControlAddress};

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
    std::thread::sleep(std::time::Duration::from_millis(1000));
    sock.transmit(
        &CANOpenFrame::new_sdo_read_frame(NODE_ID.try_into().unwrap(), 0x1018, 2) // read `Product code`
            .into(),
    )
    .unwrap();
}
