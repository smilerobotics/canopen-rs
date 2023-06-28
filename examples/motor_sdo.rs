use socketcan::{BlockingCan, CanSocket, EmbeddedFrame, Socket};

use canopen_rs::frame::{CANOpenFrame, NMTCommand, NMTNodeControlAddress};
use canopen_rs::id::{CommunicationObject, NodeID};

const INTERFACE_NAME: &str = "can0";
const NODE_ID: u8 = 1;

fn main() {
    let node_id: NodeID = NodeID::new(NODE_ID).unwrap();
    let mut sock = CanSocket::open(INTERFACE_NAME).unwrap();
    sock.transmit(
        &CANOpenFrame::new_nmt_node_control_frame(
            NMTCommand::ResetCommunication,
            NMTNodeControlAddress::Node(node_id),
        )
        .into(),
    )
    .unwrap();
    let cob: CommunicationObject = sock.receive().unwrap().id().try_into().unwrap();
    println!("received communication object: {:?}", cob);

    let frame = CANOpenFrame::new_sdo_write_frame(node_id, 0x6060, 0, &[3]);
    sock.transmit(&frame.into()).unwrap();
    let received = sock.receive().unwrap();
    let cob: CommunicationObject = received.id().try_into().unwrap();
    println!("received communication object: {:?}", cob);

    let frame = CANOpenFrame::new_nmt_node_control_frame(
        NMTCommand::Operational,
        NMTNodeControlAddress::Node(node_id),
    );
    sock.transmit(&frame.into()).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));

    let frame =
        CANOpenFrame::new_sdo_write_frame(node_id, 0x6040, 0, &0b0001_0000_0110u16.to_le_bytes());
    sock.transmit(&frame.into()).unwrap();
    let received = sock.receive().unwrap();
    let cob: CommunicationObject = received.id().try_into().unwrap();
    println!("received communication object: {:?}", cob);

    let frame =
        CANOpenFrame::new_sdo_write_frame(node_id, 0x6040, 0, &0b0001_0000_0111u16.to_le_bytes());
    sock.transmit(&frame.into()).unwrap();
    let received = sock.receive().unwrap();
    let cob: CommunicationObject = received.id().try_into().unwrap();
    println!("received communication object: {:?}", cob);

    let frame =
        CANOpenFrame::new_sdo_write_frame(node_id, 0x6040, 0, &0b0001_0000_1111u16.to_le_bytes());
    sock.transmit(&frame.into()).unwrap();
    let received = sock.receive().unwrap();
    let cob: CommunicationObject = received.id().try_into().unwrap();
    println!("received communication object: {:?}", cob);

    let frame =
        CANOpenFrame::new_sdo_write_frame(node_id, 0x6040, 0, &0b0000_0000_1111u16.to_le_bytes());
    sock.transmit(&frame.into()).unwrap();
    let received = sock.receive().unwrap();
    let cob: CommunicationObject = received.id().try_into().unwrap();
    println!("received communication object: {:?}", cob);

    let mut str_velocity = String::new();
    loop {
        std::io::stdin()
            .read_line(&mut str_velocity)
            .expect("Failed to read line");
        if let Ok(velocity) = str_velocity.trim().parse::<i32>() {
            let frame =
                CANOpenFrame::new_sdo_write_frame(node_id, 0x60FF, 0, &velocity.to_le_bytes());
            sock.transmit(&frame.into()).unwrap();
            let received = sock.receive().unwrap();
            let cob: CommunicationObject = received.id().try_into().unwrap();
            println!("received communication object: {:?}", cob);

            let frame = CANOpenFrame::new_sdo_write_frame(
                node_id,
                0x6040,
                0,
                &0b0000_0000_1111u16.to_le_bytes(),
            );
            sock.transmit(&frame.into()).unwrap();
            let received = sock.receive().unwrap();
            let cob: CommunicationObject = received.id().try_into().unwrap();
            println!("received communication object: {:?}", cob);
        } else {
            let frame =
                CANOpenFrame::new_sdo_write_frame(node_id, 0x6040, 0, &0x00u16.to_le_bytes());
            sock.transmit(&frame.into()).unwrap();
            let received = sock.receive().unwrap();
            let cob: CommunicationObject = received.id().try_into().unwrap();
            println!("received communication object: {:?}", cob);

            break;
        }
        str_velocity.clear();
    }
}
