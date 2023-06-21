use socketcan::{embedded_can::StandardId, Id};

pub type NodeID = u8;
type RawIDType = u16;

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

impl From<CommunicationObject> for RawIDType {
    fn from(cob: CommunicationObject) -> Self {
        match cob {
            CommunicationObject::NMTNodeControl => 0x000,
            CommunicationObject::GlobalFailsafeCommand => 0x001,
            CommunicationObject::Sync => 0x080,
            CommunicationObject::Emergency(node_id) => 0x080 + node_id as u16,
            CommunicationObject::TimeStamp => 0x100,
            CommunicationObject::TxPDO1(node_id) => 0x180 + node_id as u16,
            CommunicationObject::RxPDO1(node_id) => 0x200 + node_id as u16,
            CommunicationObject::TxPDO2(node_id) => 0x280 + node_id as u16,
            CommunicationObject::RxPDO2(node_id) => 0x300 + node_id as u16,
            CommunicationObject::TxPDO3(node_id) => 0x380 + node_id as u16,
            CommunicationObject::RxPDO3(node_id) => 0x400 + node_id as u16,
            CommunicationObject::TxPDO4(node_id) => 0x480 + node_id as u16,
            CommunicationObject::RxPDO4(node_id) => 0x500 + node_id as u16,
            CommunicationObject::TxSDO(node_id) => 0x580 + node_id as u16,
            CommunicationObject::RxSDO(node_id) => 0x600 + node_id as u16,
            CommunicationObject::NMTNodeMonitoring(node_id) => 0x700 + node_id as u16,
            CommunicationObject::TxLSS => 0x7E4,
            CommunicationObject::RxLSS => 0x7E5,
        }
    }
}

// TODO: define original error type and change this to TryFrom
impl From<CommunicationObject> for Id {
    fn from(cob: CommunicationObject) -> Self {
        Id::Standard(StandardId::new(cob.into()).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_raw_id() {
        let raw_id: RawIDType = CommunicationObject::NMTNodeControl.into();
        assert_eq!(raw_id, 0x000);
        let raw_id: RawIDType = CommunicationObject::GlobalFailsafeCommand.into();
        assert_eq!(raw_id, 0x001);
        let raw_id: RawIDType = CommunicationObject::Sync.into();
        assert_eq!(raw_id, 0x080);
        let raw_id: RawIDType = CommunicationObject::Emergency(1).into();
        assert_eq!(raw_id, 0x081);
        let raw_id: RawIDType = CommunicationObject::TimeStamp.into();
        assert_eq!(raw_id, 0x100);
        let raw_id: RawIDType = CommunicationObject::TxPDO1(2).into();
        assert_eq!(raw_id, 0x182);
        let raw_id: RawIDType = CommunicationObject::RxPDO1(3).into();
        assert_eq!(raw_id, 0x203);
        let raw_id: RawIDType = CommunicationObject::TxPDO2(4).into();
        assert_eq!(raw_id, 0x284);
        let raw_id: RawIDType = CommunicationObject::RxPDO2(5).into();
        assert_eq!(raw_id, 0x305);
        let raw_id: RawIDType = CommunicationObject::TxPDO3(6).into();
        assert_eq!(raw_id, 0x386);
        let raw_id: RawIDType = CommunicationObject::RxPDO3(7).into();
        assert_eq!(raw_id, 0x407);
        let raw_id: RawIDType = CommunicationObject::TxPDO4(8).into();
        assert_eq!(raw_id, 0x488);
        let raw_id: RawIDType = CommunicationObject::RxPDO4(9).into();
        assert_eq!(raw_id, 0x509);
        let raw_id: RawIDType = CommunicationObject::TxSDO(10).into();
        assert_eq!(raw_id, 0x58A);
        let raw_id: RawIDType = CommunicationObject::RxSDO(11).into();
        assert_eq!(raw_id, 0x60B);
        let raw_id: RawIDType = CommunicationObject::NMTNodeMonitoring(12).into();
        assert_eq!(raw_id, 0x70C);
        let raw_id: RawIDType = CommunicationObject::TxLSS.into();
        assert_eq!(raw_id, 0x7E4);
        let raw_id: RawIDType = CommunicationObject::RxLSS.into();
        assert_eq!(raw_id, 0x7E5);
    }

    #[test]
    fn test_into_id() {
        let id: Id = CommunicationObject::NMTNodeControl.into();
        assert_eq!(id, Id::Standard(StandardId::new(0x000).unwrap()));
        let id: Id = CommunicationObject::GlobalFailsafeCommand.into();
        assert_eq!(id, Id::Standard(StandardId::new(0x001).unwrap()));
        let id: Id = CommunicationObject::Sync.into();
        assert_eq!(id, Id::Standard(StandardId::new(0x080).unwrap()));
        let id: Id = CommunicationObject::Emergency(1).into();
        assert_eq!(id, Id::Standard(StandardId::new(0x081).unwrap()));
        let id: Id = CommunicationObject::TimeStamp.into();
        assert_eq!(id, Id::Standard(StandardId::new(0x100).unwrap()));
        let id: Id = CommunicationObject::TxPDO1(2).into();
        assert_eq!(id, Id::Standard(StandardId::new(0x182).unwrap()));
        let id: Id = CommunicationObject::RxPDO1(3).into();
        assert_eq!(id, Id::Standard(StandardId::new(0x203).unwrap()));
        let id: Id = CommunicationObject::TxPDO2(4).into();
        assert_eq!(id, Id::Standard(StandardId::new(0x284).unwrap()));
        let id: Id = CommunicationObject::RxPDO2(5).into();
        assert_eq!(id, Id::Standard(StandardId::new(0x305).unwrap()));
        let id: Id = CommunicationObject::TxPDO3(6).into();
        assert_eq!(id, Id::Standard(StandardId::new(0x386).unwrap()));
        let id: Id = CommunicationObject::RxPDO3(7).into();
        assert_eq!(id, Id::Standard(StandardId::new(0x407).unwrap()));
        let id: Id = CommunicationObject::TxPDO4(8).into();
        assert_eq!(id, Id::Standard(StandardId::new(0x488).unwrap()));
        let id: Id = CommunicationObject::RxPDO4(9).into();
        assert_eq!(id, Id::Standard(StandardId::new(0x509).unwrap()));
        let id: Id = CommunicationObject::TxSDO(10).into();
        assert_eq!(id, Id::Standard(StandardId::new(0x58A).unwrap()));
        let id: Id = CommunicationObject::RxSDO(11).into();
        assert_eq!(id, Id::Standard(StandardId::new(0x60B).unwrap()));
        let id: Id = CommunicationObject::NMTNodeMonitoring(12).into();
        assert_eq!(id, Id::Standard(StandardId::new(0x70C).unwrap()));
        let id: Id = CommunicationObject::TxLSS.into();
        assert_eq!(id, Id::Standard(StandardId::new(0x7E4).unwrap()));
        let id: Id = CommunicationObject::RxLSS.into();
        assert_eq!(id, Id::Standard(StandardId::new(0x7E5).unwrap()));
    }
}
