use crate::error::{Error, Result};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct NodeId(u8);

impl NodeId {
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

impl TryFrom<u8> for NodeId {
    type Error = Error;
    fn try_from(raw_id: u8) -> std::result::Result<Self, Self::Error> {
        NodeId::new(raw_id)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CommunicationObject {
    NmtNodeControl,
    GlobalFailsafeCommand,
    Sync,
    Emergency(NodeId),
    TimeStamp,
    TxPdo1(NodeId),
    RxPdo1(NodeId),
    TxPdo2(NodeId),
    RxPdo2(NodeId),
    TxPdo3(NodeId),
    RxPdo3(NodeId),
    TxPdo4(NodeId),
    RxPdo4(NodeId),
    TxSdo(NodeId),
    RxSdo(NodeId),
    NmtNodeMonitoring(NodeId),
    TxLss,
    RxLss,
}

impl CommunicationObject {
    fn raw_id_to_node_id(cob_id: u16) -> NodeId {
        ((cob_id & 0x7F) as u8).try_into().unwrap()
    }

    fn new(id: u16) -> Result<Self> {
        match id & !0x07FF {
            0 => match id & 0b00000111_10000000 {
                0x000 => match id {
                    0 => Ok(CommunicationObject::NmtNodeControl),
                    1 => Ok(CommunicationObject::GlobalFailsafeCommand),
                    _ => Err(Error::InvalidCobId(id)),
                },
                0x080 => match id & 0x007F {
                    0 => Ok(CommunicationObject::Sync),
                    _ => Ok(CommunicationObject::Emergency(Self::raw_id_to_node_id(id))),
                },
                0x100 => Ok(CommunicationObject::TimeStamp),
                0x180 => Ok(CommunicationObject::TxPdo1(Self::raw_id_to_node_id(id))),
                0x200 => Ok(CommunicationObject::RxPdo1(Self::raw_id_to_node_id(id))),
                0x280 => Ok(CommunicationObject::TxPdo2(Self::raw_id_to_node_id(id))),
                0x300 => Ok(CommunicationObject::RxPdo2(Self::raw_id_to_node_id(id))),
                0x380 => Ok(CommunicationObject::TxPdo3(Self::raw_id_to_node_id(id))),
                0x400 => Ok(CommunicationObject::RxPdo3(Self::raw_id_to_node_id(id))),
                0x480 => Ok(CommunicationObject::TxPdo4(Self::raw_id_to_node_id(id))),
                0x500 => Ok(CommunicationObject::RxPdo4(Self::raw_id_to_node_id(id))),
                0x580 => Ok(CommunicationObject::TxSdo(Self::raw_id_to_node_id(id))),
                0x600 => Ok(CommunicationObject::RxSdo(Self::raw_id_to_node_id(id))),
                0x700 => Ok(CommunicationObject::NmtNodeMonitoring(
                    Self::raw_id_to_node_id(id),
                )),
                0x780 => match id {
                    0x7E4 => Ok(CommunicationObject::TxLss),
                    0x7E5 => Ok(CommunicationObject::RxLss),
                    _ => Err(Error::InvalidCobId(id)),
                },
                _ => Err(Error::InvalidCobId(id)),
            },
            _ => Err(Error::InvalidCobId(id)),
        }
    }

    fn as_cob_id(&self) -> u16 {
        match self {
            CommunicationObject::NmtNodeControl => 0x000,
            CommunicationObject::GlobalFailsafeCommand => 0x001,
            CommunicationObject::Sync => 0x080,
            CommunicationObject::Emergency(node_id) => 0x080 + node_id.as_raw() as u16,
            CommunicationObject::TimeStamp => 0x100,
            CommunicationObject::TxPdo1(node_id) => 0x180 + node_id.as_raw() as u16,
            CommunicationObject::RxPdo1(node_id) => 0x200 + node_id.as_raw() as u16,
            CommunicationObject::TxPdo2(node_id) => 0x280 + node_id.as_raw() as u16,
            CommunicationObject::RxPdo2(node_id) => 0x300 + node_id.as_raw() as u16,
            CommunicationObject::TxPdo3(node_id) => 0x380 + node_id.as_raw() as u16,
            CommunicationObject::RxPdo3(node_id) => 0x400 + node_id.as_raw() as u16,
            CommunicationObject::TxPdo4(node_id) => 0x480 + node_id.as_raw() as u16,
            CommunicationObject::RxPdo4(node_id) => 0x500 + node_id.as_raw() as u16,
            CommunicationObject::TxSdo(node_id) => 0x580 + node_id.as_raw() as u16,
            CommunicationObject::RxSdo(node_id) => 0x600 + node_id.as_raw() as u16,
            CommunicationObject::NmtNodeMonitoring(node_id) => 0x700 + node_id.as_raw() as u16,
            CommunicationObject::TxLss => 0x7E4,
            CommunicationObject::RxLss => 0x7E5,
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
        assert_eq!(NodeId::new(1), Ok(NodeId(1)));
        assert_eq!(NodeId::new(2), Ok(NodeId(2)));
        assert_eq!(NodeId::new(3), Ok(NodeId(3)));
        assert_eq!(NodeId::new(127), Ok(NodeId(127)));
        assert!(NodeId::new(128).is_err());
        assert!(NodeId::new(255).is_err());
    }

    #[test]
    fn test_node_id_try_into() {
        let node_id: Result<NodeId> = 1.try_into();
        assert_eq!(node_id, Ok(NodeId(1)));
        let node_id: Result<NodeId> = 2.try_into();
        assert_eq!(node_id, Ok(NodeId(2)));
        let node_id: Result<NodeId> = 3.try_into();
        assert_eq!(node_id, Ok(NodeId(3)));
        let node_id: Result<NodeId> = 127.try_into();
        assert_eq!(node_id, Ok(NodeId(127)));
        let node_id: Result<NodeId> = 128.try_into();
        assert!(node_id.is_err());
        let node_id: Result<NodeId> = 255.try_into();
        assert!(node_id.is_err());
    }

    #[test]
    fn test_as_cob_id() {
        assert_eq!(CommunicationObject::NmtNodeControl.as_cob_id(), 0x000);
        assert_eq!(
            CommunicationObject::RxPdo1(3.try_into().unwrap()).as_cob_id(),
            0x203
        );
        assert_eq!(
            CommunicationObject::TxPdo2(4.try_into().unwrap()).as_cob_id(),
            0x284
        );
        assert_eq!(
            CommunicationObject::RxPdo2(5.try_into().unwrap()).as_cob_id(),
            0x305
        );
        assert_eq!(
            CommunicationObject::TxPdo3(6.try_into().unwrap()).as_cob_id(),
            0x386
        );
        assert_eq!(
            CommunicationObject::RxPdo3(7.try_into().unwrap()).as_cob_id(),
            0x407
        );
        assert_eq!(
            CommunicationObject::TxPdo4(8.try_into().unwrap()).as_cob_id(),
            0x488
        );
        assert_eq!(
            CommunicationObject::RxPdo4(9.try_into().unwrap()).as_cob_id(),
            0x509
        );
        assert_eq!(
            CommunicationObject::TxSdo(10.try_into().unwrap()).as_cob_id(),
            0x58A
        );
        assert_eq!(
            CommunicationObject::RxSdo(11.try_into().unwrap()).as_cob_id(),
            0x60B
        );
        assert_eq!(
            CommunicationObject::NmtNodeMonitoring(12.try_into().unwrap()).as_cob_id(),
            0x70C
        );
        assert_eq!(CommunicationObject::TxLss.as_cob_id(), 0x7E4);
        assert_eq!(CommunicationObject::RxLss.as_cob_id(), 0x7E5);
    }

    #[test]
    fn test_into_id() {
        let id: socketcan::Id = CommunicationObject::NmtNodeControl.into();
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
        let id: socketcan::Id = CommunicationObject::TxPdo1(2.try_into().unwrap()).into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x182).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::RxPdo1(3.try_into().unwrap()).into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x203).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::TxPdo2(4.try_into().unwrap()).into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x284).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::RxPdo2(5.try_into().unwrap()).into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x305).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::TxPdo3(6.try_into().unwrap()).into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x386).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::RxPdo3(7.try_into().unwrap()).into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x407).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::TxPdo4(8.try_into().unwrap()).into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x488).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::RxPdo4(9.try_into().unwrap()).into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x509).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::TxSdo(10.try_into().unwrap()).into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x58A).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::RxSdo(11.try_into().unwrap()).into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x60B).unwrap())
        );
        let id: socketcan::Id =
            CommunicationObject::NmtNodeMonitoring(12.try_into().unwrap()).into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x70C).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::TxLss.into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x7E4).unwrap())
        );
        let id: socketcan::Id = CommunicationObject::RxLss.into();
        assert_eq!(
            id,
            socketcan::Id::Standard(socketcan::StandardId::new(0x7E5).unwrap())
        );
    }

    #[test]
    fn test_new() {
        let cob = CommunicationObject::new(0x000);
        assert_eq!(cob, Ok(CommunicationObject::NmtNodeControl));
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
        assert_eq!(cob, Ok(CommunicationObject::TxPdo1(1.try_into().unwrap())));
        let cob = CommunicationObject::new(0x18F);
        assert_eq!(cob, Ok(CommunicationObject::TxPdo1(15.try_into().unwrap())));
        let cob = CommunicationObject::new(0x1FF);
        assert_eq!(
            cob,
            Ok(CommunicationObject::TxPdo1(127.try_into().unwrap()))
        );
        let cob = CommunicationObject::new(0x201);
        assert_eq!(cob, Ok(CommunicationObject::RxPdo1(1.try_into().unwrap())));
        let cob = CommunicationObject::new(0x20F);
        assert_eq!(cob, Ok(CommunicationObject::RxPdo1(15.try_into().unwrap())));
        let cob = CommunicationObject::new(0x27F);
        assert_eq!(
            cob,
            Ok(CommunicationObject::RxPdo1(127.try_into().unwrap()))
        );
        let cob = CommunicationObject::new(0x281);
        assert_eq!(cob, Ok(CommunicationObject::TxPdo2(1.try_into().unwrap())));
        let cob = CommunicationObject::new(0x28F);
        assert_eq!(cob, Ok(CommunicationObject::TxPdo2(15.try_into().unwrap())));
        let cob = CommunicationObject::new(0x2FF);
        assert_eq!(
            cob,
            Ok(CommunicationObject::TxPdo2(127.try_into().unwrap()))
        );
        let cob = CommunicationObject::new(0x301);
        assert_eq!(cob, Ok(CommunicationObject::RxPdo2(1.try_into().unwrap())));
        let cob = CommunicationObject::new(0x30F);
        assert_eq!(cob, Ok(CommunicationObject::RxPdo2(15.try_into().unwrap())));
        let cob = CommunicationObject::new(0x37F);
        assert_eq!(
            cob,
            Ok(CommunicationObject::RxPdo2(127.try_into().unwrap()))
        );
        let cob = CommunicationObject::new(0x381);
        assert_eq!(cob, Ok(CommunicationObject::TxPdo3(1.try_into().unwrap())));
        let cob = CommunicationObject::new(0x38F);
        assert_eq!(cob, Ok(CommunicationObject::TxPdo3(15.try_into().unwrap())));
        let cob = CommunicationObject::new(0x3FF);
        assert_eq!(
            cob,
            Ok(CommunicationObject::TxPdo3(127.try_into().unwrap()))
        );
        let cob = CommunicationObject::new(0x401);
        assert_eq!(cob, Ok(CommunicationObject::RxPdo3(1.try_into().unwrap())));
        let cob = CommunicationObject::new(0x40F);
        assert_eq!(cob, Ok(CommunicationObject::RxPdo3(15.try_into().unwrap())));
        let cob = CommunicationObject::new(0x47F);
        assert_eq!(
            cob,
            Ok(CommunicationObject::RxPdo3(127.try_into().unwrap()))
        );
        let cob = CommunicationObject::new(0x481);
        assert_eq!(cob, Ok(CommunicationObject::TxPdo4(1.try_into().unwrap())));
        let cob = CommunicationObject::new(0x48F);
        assert_eq!(cob, Ok(CommunicationObject::TxPdo4(15.try_into().unwrap())));
        let cob = CommunicationObject::new(0x4FF);
        assert_eq!(
            cob,
            Ok(CommunicationObject::TxPdo4(127.try_into().unwrap()))
        );
        let cob = CommunicationObject::new(0x501);
        assert_eq!(cob, Ok(CommunicationObject::RxPdo4(1.try_into().unwrap())));
        let cob = CommunicationObject::new(0x50F);
        assert_eq!(cob, Ok(CommunicationObject::RxPdo4(15.try_into().unwrap())));
        let cob = CommunicationObject::new(0x57F);
        assert_eq!(
            cob,
            Ok(CommunicationObject::RxPdo4(127.try_into().unwrap()))
        );
        let cob = CommunicationObject::new(0x581);
        assert_eq!(cob, Ok(CommunicationObject::TxSdo(1.try_into().unwrap())));
        let cob = CommunicationObject::new(0x58F);
        assert_eq!(cob, Ok(CommunicationObject::TxSdo(15.try_into().unwrap())));
        let cob = CommunicationObject::new(0x5FF);
        assert_eq!(cob, Ok(CommunicationObject::TxSdo(127.try_into().unwrap())));
        let cob = CommunicationObject::new(0x601);
        assert_eq!(cob, Ok(CommunicationObject::RxSdo(1.try_into().unwrap())));
        let cob = CommunicationObject::new(0x60F);
        assert_eq!(cob, Ok(CommunicationObject::RxSdo(15.try_into().unwrap())));
        let cob = CommunicationObject::new(0x67F);
        assert_eq!(cob, Ok(CommunicationObject::RxSdo(127.try_into().unwrap())));
        let cob = CommunicationObject::new(0x701);
        assert_eq!(
            cob,
            Ok(CommunicationObject::NmtNodeMonitoring(
                1.try_into().unwrap()
            ))
        );
        let cob = CommunicationObject::new(0x70F);
        assert_eq!(
            cob,
            Ok(CommunicationObject::NmtNodeMonitoring(
                15.try_into().unwrap()
            ))
        );
        let cob = CommunicationObject::new(0x77F);
        assert_eq!(
            cob,
            Ok(CommunicationObject::NmtNodeMonitoring(
                127.try_into().unwrap()
            ))
        );
        let cob = CommunicationObject::new(0x7E4);
        assert_eq!(cob, Ok(CommunicationObject::TxLss));
        let cob = CommunicationObject::new(0x7E5);
        assert_eq!(cob, Ok(CommunicationObject::RxLss));
    }

    #[test]
    fn test_socketcan_id_into_cob() {
        let cob: Result<CommunicationObject> =
            socketcan::Id::Standard(socketcan::StandardId::new(0x000).unwrap()).try_into();
        assert_eq!(cob, Ok(CommunicationObject::NmtNodeControl));
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
        assert_eq!(cob, Ok(CommunicationObject::RxSdo(127.try_into().unwrap())));
        let cob: Result<CommunicationObject> =
            socketcan::Id::Extended(socketcan::ExtendedId::new(0x0000).unwrap()).try_into();
        assert_eq!(cob, Err(Error::CanFdNotSupported));
    }
}
