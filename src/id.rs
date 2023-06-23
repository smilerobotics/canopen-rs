use crate::error::{Error, Result};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct NodeID(u8);

impl NodeID {
    pub fn new(raw_id: u8) -> Result<Self> {
        match raw_id & 0x80 {
            0 => Ok(Self(raw_id)),
            _ => Err(Error::InvalidNodeId(raw_id)),
        }
    }

    pub fn as_raw(&self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for NodeID {
    type Error = Error;
    fn try_from(raw_id: u8) -> std::result::Result<Self, Self::Error> {
        NodeID::new(raw_id)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CommunicationObject {
    NMTNodeControl,
    GlobalFailsafeCommand,
    Sync,
    Emergency(NodeID),
    TimeStamp,
    TxPDO1(NodeID),
    RxPDO1(NodeID),
    TxPDO2(NodeID),
    RxPDO2(NodeID),
    TxPDO3(NodeID),
    RxPDO3(NodeID),
    TxPDO4(NodeID),
    RxPDO4(NodeID),
    TxSDO(NodeID),
    RxSDO(NodeID),
    NMTNodeMonitoring(NodeID),
    TxLSS,
    RxLSS,
}

impl CommunicationObject {
    fn raw_id_to_node_id(cob_id: u16) -> NodeID {
        ((cob_id & 0x7F) as u8).try_into().unwrap()
    }

    fn new(id: u16) -> Result<Self> {
        match id & !0x07FF {
            0 => match id & 0b00000111_10000000 {
                0x000 => match id {
                    0 => Ok(CommunicationObject::NMTNodeControl),
                    1 => Ok(CommunicationObject::GlobalFailsafeCommand),
                    _ => Err(Error::InvalidCobId(id)),
                },
                0x080 => match id & 0x007F {
                    0 => Ok(CommunicationObject::Sync),
                    _ => Ok(CommunicationObject::Emergency(Self::raw_id_to_node_id(id))),
                },
                0x100 => Ok(CommunicationObject::TimeStamp),
                0x180 => Ok(CommunicationObject::TxPDO1(Self::raw_id_to_node_id(id))),
                0x200 => Ok(CommunicationObject::RxPDO1(Self::raw_id_to_node_id(id))),
                0x280 => Ok(CommunicationObject::TxPDO2(Self::raw_id_to_node_id(id))),
                0x300 => Ok(CommunicationObject::RxPDO2(Self::raw_id_to_node_id(id))),
                0x380 => Ok(CommunicationObject::TxPDO3(Self::raw_id_to_node_id(id))),
                0x400 => Ok(CommunicationObject::RxPDO3(Self::raw_id_to_node_id(id))),
                0x480 => Ok(CommunicationObject::TxPDO4(Self::raw_id_to_node_id(id))),
                0x500 => Ok(CommunicationObject::RxPDO4(Self::raw_id_to_node_id(id))),
                0x580 => Ok(CommunicationObject::TxSDO(Self::raw_id_to_node_id(id))),
                0x600 => Ok(CommunicationObject::RxSDO(Self::raw_id_to_node_id(id))),
                0x700 => Ok(CommunicationObject::NMTNodeMonitoring(
                    Self::raw_id_to_node_id(id),
                )),
                0x780 => match id {
                    0x7E4 => Ok(CommunicationObject::TxLSS),
                    0x7E5 => Ok(CommunicationObject::RxLSS),
                    _ => Err(Error::InvalidCobId(id)),
                },
                _ => Err(Error::InvalidCobId(id)),
            },
            _ => Err(Error::InvalidCobId(id)),
        }
    }

    fn as_cob_id(&self) -> u16 {
        match self {
            CommunicationObject::NMTNodeControl => 0x000,
            CommunicationObject::GlobalFailsafeCommand => 0x001,
            CommunicationObject::Sync => 0x080,
            CommunicationObject::Emergency(node_id) => 0x080 + node_id.as_raw() as u16,
            CommunicationObject::TimeStamp => 0x100,
            CommunicationObject::TxPDO1(node_id) => 0x180 + node_id.as_raw() as u16,
            CommunicationObject::RxPDO1(node_id) => 0x200 + node_id.as_raw() as u16,
            CommunicationObject::TxPDO2(node_id) => 0x280 + node_id.as_raw() as u16,
            CommunicationObject::RxPDO2(node_id) => 0x300 + node_id.as_raw() as u16,
            CommunicationObject::TxPDO3(node_id) => 0x380 + node_id.as_raw() as u16,
            CommunicationObject::RxPDO3(node_id) => 0x400 + node_id.as_raw() as u16,
            CommunicationObject::TxPDO4(node_id) => 0x480 + node_id.as_raw() as u16,
            CommunicationObject::RxPDO4(node_id) => 0x500 + node_id.as_raw() as u16,
            CommunicationObject::TxSDO(node_id) => 0x580 + node_id.as_raw() as u16,
            CommunicationObject::RxSDO(node_id) => 0x600 + node_id.as_raw() as u16,
            CommunicationObject::NMTNodeMonitoring(node_id) => 0x700 + node_id.as_raw() as u16,
            CommunicationObject::TxLSS => 0x7E4,
            CommunicationObject::RxLSS => 0x7E5,
        }
    }
}

impl From<CommunicationObject> for socketcan::Id {
    fn from(cob: CommunicationObject) -> Self {
        socketcan::Id::Standard(socketcan::StandardId::new(cob.as_cob_id()).unwrap())
        // The `new` method could return `None` if the raw ID is greater than 0x7FF.
        // But `CommunicationObject::as_cob_id` method never returns such values,
        // because its inner node_id is limited.
    }
}

impl TryFrom<socketcan::Id> for CommunicationObject {
    type Error = Error;
    fn try_from(id: socketcan::Id) -> Result<Self> {
        match id {
            socketcan::Id::Standard(id) => CommunicationObject::new(id.as_raw()),
            socketcan::Id::Extended(_id) => Err(Error::CanFdNotSupported),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_new() {
        assert_eq!(NodeID::new(1), Ok(NodeID(1)));
        assert_eq!(NodeID::new(2), Ok(NodeID(2)));
        assert_eq!(NodeID::new(3), Ok(NodeID(3)));
        assert_eq!(NodeID::new(127), Ok(NodeID(127)));
        assert!(NodeID::new(128).is_err());
        assert!(NodeID::new(255).is_err());
    }

    #[test]
    fn test_node_id_try_into() {
        let node_id: Result<NodeID> = 1.try_into();
        assert_eq!(node_id, Ok(NodeID(1)));
        let node_id: Result<NodeID> = 2.try_into();
        assert_eq!(node_id, Ok(NodeID(2)));
        let node_id: Result<NodeID> = 3.try_into();
        assert_eq!(node_id, Ok(NodeID(3)));
        let node_id: Result<NodeID> = 127.try_into();
        assert_eq!(node_id, Ok(NodeID(127)));
        let node_id: Result<NodeID> = 128.try_into();
        assert!(node_id.is_err());
        let node_id: Result<NodeID> = 255.try_into();
        assert!(node_id.is_err());
    }

    #[test]
    fn test_as_cob_id() {
        assert_eq!(CommunicationObject::NMTNodeControl.as_cob_id(), 0x000);
        assert_eq!(
            CommunicationObject::RxPDO1(3.try_into().unwrap()).as_cob_id(),
            0x203
        );
        assert_eq!(
            CommunicationObject::TxPDO2(4.try_into().unwrap()).as_cob_id(),
            0x284
        );
        assert_eq!(
            CommunicationObject::RxPDO2(5.try_into().unwrap()).as_cob_id(),
            0x305
        );
        assert_eq!(
            CommunicationObject::TxPDO3(6.try_into().unwrap()).as_cob_id(),
            0x386
        );
        assert_eq!(
            CommunicationObject::RxPDO3(7.try_into().unwrap()).as_cob_id(),
            0x407
        );
        assert_eq!(
            CommunicationObject::TxPDO4(8.try_into().unwrap()).as_cob_id(),
            0x488
        );
        assert_eq!(
            CommunicationObject::RxPDO4(9.try_into().unwrap()).as_cob_id(),
            0x509
        );
        assert_eq!(
            CommunicationObject::TxSDO(10.try_into().unwrap()).as_cob_id(),
            0x58A
        );
        assert_eq!(
            CommunicationObject::RxSDO(11.try_into().unwrap()).as_cob_id(),
            0x60B
        );
        assert_eq!(
            CommunicationObject::NMTNodeMonitoring(12.try_into().unwrap()).as_cob_id(),
            0x70C
        );
        assert_eq!(CommunicationObject::TxLSS.as_cob_id(), 0x7E4);
        assert_eq!(CommunicationObject::RxLSS.as_cob_id(), 0x7E5);
    }

    #[test]
    fn test_into_id() {
        let id: socketcan::Id = CommunicationObject::NMTNodeControl.into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x000).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::GlobalFailsafeCommand.into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x001).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::Sync.into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x080).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::Emergency(1.try_into().unwrap()).into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x081).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::TimeStamp.into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x100).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::TxPDO1(2.try_into().unwrap()).into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x182).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::RxPDO1(3.try_into().unwrap()).into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x203).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::TxPDO2(4.try_into().unwrap()).into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x284).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::RxPDO2(5.try_into().unwrap()).into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x305).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::TxPDO3(6.try_into().unwrap()).into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x386).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::RxPDO3(7.try_into().unwrap()).into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x407).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::TxPDO4(8.try_into().unwrap()).into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x488).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::RxPDO4(9.try_into().unwrap()).into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x509).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::TxSDO(10.try_into().unwrap()).into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x58A).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::RxSDO(11.try_into().unwrap()).into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x60B).unwrap())
        );
        let id: socketcan::Id =
            CommunicationObject::NMTNodeMonitoring(12.try_into().unwrap()).into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x70C).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::TxLSS.into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x7E4).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::RxLSS.into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x7E5).unwrap())
        );
    }

    #[test]
    fn test_new() {
        let cob = CommunicationObject::new(0x000);
        assert_eq!(cob, Ok(CommunicationObject::NMTNodeControl));
        let cob = CommunicationObject::new(0x001);
        assert_eq!(cob, Ok(CommunicationObject::GlobalFailsafeCommand));
        let cob = CommunicationObject::new(0x080);
        assert_eq!(cob, Ok(CommunicationObject::Sync));
        let cob = CommunicationObject::new(0x81);
        assert_eq!(
            cob,
            Ok(CommunicationObject::Emergency(1.try_into().unwrap()))
        );
        let cob = CommunicationObject::new(0x08F);
        assert_eq!(
            cob,
            Ok(CommunicationObject::Emergency(15.try_into().unwrap()))
        );
        let cob = CommunicationObject::new(0x0FF);
        assert_eq!(
            cob,
            Ok(CommunicationObject::Emergency(127.try_into().unwrap()))
        );
        let cob = CommunicationObject::new(0x100);
        assert_eq!(cob, Ok(CommunicationObject::TimeStamp));
        let cob = CommunicationObject::new(0x181);
        assert_eq!(cob, Ok(CommunicationObject::TxPDO1(1.try_into().unwrap())));
        let cob = CommunicationObject::new(0x18F);
        assert_eq!(cob, Ok(CommunicationObject::TxPDO1(15.try_into().unwrap())));
        let cob = CommunicationObject::new(0x1FF);
        assert_eq!(
            cob,
            Ok(CommunicationObject::TxPDO1(127.try_into().unwrap()))
        );
        let cob = CommunicationObject::new(0x201);
        assert_eq!(cob, Ok(CommunicationObject::RxPDO1(1.try_into().unwrap())));
        let cob = CommunicationObject::new(0x20F);
        assert_eq!(cob, Ok(CommunicationObject::RxPDO1(15.try_into().unwrap())));
        let cob = CommunicationObject::new(0x27F);
        assert_eq!(
            cob,
            Ok(CommunicationObject::RxPDO1(127.try_into().unwrap()))
        );
        let cob = CommunicationObject::new(0x281);
        assert_eq!(cob, Ok(CommunicationObject::TxPDO2(1.try_into().unwrap())));
        let cob = CommunicationObject::new(0x28F);
        assert_eq!(cob, Ok(CommunicationObject::TxPDO2(15.try_into().unwrap())));
        let cob = CommunicationObject::new(0x2FF);
        assert_eq!(
            cob,
            Ok(CommunicationObject::TxPDO2(127.try_into().unwrap()))
        );
        let cob = CommunicationObject::new(0x301);
        assert_eq!(cob, Ok(CommunicationObject::RxPDO2(1.try_into().unwrap())));
        let cob = CommunicationObject::new(0x30F);
        assert_eq!(cob, Ok(CommunicationObject::RxPDO2(15.try_into().unwrap())));
        let cob = CommunicationObject::new(0x37F);
        assert_eq!(
            cob,
            Ok(CommunicationObject::RxPDO2(127.try_into().unwrap()))
        );
        let cob = CommunicationObject::new(0x381);
        assert_eq!(cob, Ok(CommunicationObject::TxPDO3(1.try_into().unwrap())));
        let cob = CommunicationObject::new(0x38F);
        assert_eq!(cob, Ok(CommunicationObject::TxPDO3(15.try_into().unwrap())));
        let cob = CommunicationObject::new(0x3FF);
        assert_eq!(
            cob,
            Ok(CommunicationObject::TxPDO3(127.try_into().unwrap()))
        );
        let cob = CommunicationObject::new(0x401);
        assert_eq!(cob, Ok(CommunicationObject::RxPDO3(1.try_into().unwrap())));
        let cob = CommunicationObject::new(0x40F);
        assert_eq!(cob, Ok(CommunicationObject::RxPDO3(15.try_into().unwrap())));
        let cob = CommunicationObject::new(0x47F);
        assert_eq!(
            cob,
            Ok(CommunicationObject::RxPDO3(127.try_into().unwrap()))
        );
        let cob = CommunicationObject::new(0x481);
        assert_eq!(cob, Ok(CommunicationObject::TxPDO4(1.try_into().unwrap())));
        let cob = CommunicationObject::new(0x48F);
        assert_eq!(cob, Ok(CommunicationObject::TxPDO4(15.try_into().unwrap())));
        let cob = CommunicationObject::new(0x4FF);
        assert_eq!(
            cob,
            Ok(CommunicationObject::TxPDO4(127.try_into().unwrap()))
        );
        let cob = CommunicationObject::new(0x501);
        assert_eq!(cob, Ok(CommunicationObject::RxPDO4(1.try_into().unwrap())));
        let cob = CommunicationObject::new(0x50F);
        assert_eq!(cob, Ok(CommunicationObject::RxPDO4(15.try_into().unwrap())));
        let cob = CommunicationObject::new(0x57F);
        assert_eq!(
            cob,
            Ok(CommunicationObject::RxPDO4(127.try_into().unwrap()))
        );
        let cob = CommunicationObject::new(0x581);
        assert_eq!(cob, Ok(CommunicationObject::TxSDO(1.try_into().unwrap())));
        let cob = CommunicationObject::new(0x58F);
        assert_eq!(cob, Ok(CommunicationObject::TxSDO(15.try_into().unwrap())));
        let cob = CommunicationObject::new(0x5FF);
        assert_eq!(cob, Ok(CommunicationObject::TxSDO(127.try_into().unwrap())));
        let cob = CommunicationObject::new(0x601);
        assert_eq!(cob, Ok(CommunicationObject::RxSDO(1.try_into().unwrap())));
        let cob = CommunicationObject::new(0x60F);
        assert_eq!(cob, Ok(CommunicationObject::RxSDO(15.try_into().unwrap())));
        let cob = CommunicationObject::new(0x67F);
        assert_eq!(cob, Ok(CommunicationObject::RxSDO(127.try_into().unwrap())));
        let cob = CommunicationObject::new(0x701);
        assert_eq!(
            cob,
            Ok(CommunicationObject::NMTNodeMonitoring(
                1.try_into().unwrap()
            ))
        );
        let cob = CommunicationObject::new(0x70F);
        assert_eq!(
            cob,
            Ok(CommunicationObject::NMTNodeMonitoring(
                15.try_into().unwrap()
            ))
        );
        let cob = CommunicationObject::new(0x77F);
        assert_eq!(
            cob,
            Ok(CommunicationObject::NMTNodeMonitoring(
                127.try_into().unwrap()
            ))
        );
        let cob = CommunicationObject::new(0x7E4);
        assert_eq!(cob, Ok(CommunicationObject::TxLSS));
        let cob = CommunicationObject::new(0x7E5);
        assert_eq!(cob, Ok(CommunicationObject::RxLSS));
    }

    #[test]
    fn test_socketcan_id_into_cob() {
        let cob: Result<CommunicationObject> =
            socketcan::Id::Standard(socketcan::StandardId::new(0x000).unwrap()).try_into();
        assert_eq!(cob, Ok(CommunicationObject::NMTNodeControl));
        let cob: Result<CommunicationObject> =
            socketcan::Id::Standard(socketcan::StandardId::new(0x001).unwrap()).try_into();
        assert_eq!(cob, Ok(CommunicationObject::GlobalFailsafeCommand));
        let cob: Result<CommunicationObject> =
            socketcan::Id::Standard(socketcan::StandardId::new(0x080).unwrap()).try_into();
        assert_eq!(cob, Ok(CommunicationObject::Sync));
        let cob: Result<CommunicationObject> =
            socketcan::Id::Standard(socketcan::StandardId::new(0x081).unwrap()).try_into();
        assert_eq!(
            cob,
            Ok(CommunicationObject::Emergency(1.try_into().unwrap()))
        );
        let cob: Result<CommunicationObject> =
            socketcan::Id::Standard(socketcan::StandardId::new(0x67F).unwrap()).try_into();
        assert_eq!(cob, Ok(CommunicationObject::RxSDO(127.try_into().unwrap())));
        let cob: Result<CommunicationObject> =
            socketcan::Id::Extended(socketcan::ExtendedId::new(0x0000).unwrap()).try_into();
        assert_eq!(cob, Err(Error::CanFdNotSupported));
    }
}
