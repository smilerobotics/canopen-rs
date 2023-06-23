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
}
