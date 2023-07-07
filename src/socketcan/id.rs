use crate::error::{Error, Result};
use crate::id::CommunicationObject;

impl From<CommunicationObject> for socketcan::Id {
    fn from(cob: CommunicationObject) -> Self {
        socketcan::Id::Standard(socketcan::StandardId::new(cob.as_cob_id()).expect(
            "Should have failed only when the passed raw ID was out of range (11-bit), but the COB-ID must not have been out of the range."
        ))
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
    fn test_cob_into_socketcan_id() {
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
