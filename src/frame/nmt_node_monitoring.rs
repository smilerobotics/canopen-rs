use crate::error::{Error, Result};
use crate::frame::{CanOpenFrame, ConvertibleFrame};
use crate::id::{CommunicationObject, NodeId};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NmtState {
    BootUp = 0x00,
    Stopped = 0x04,
    Operational = 0x05,
    PreOperational = 0x7F,
}

impl NmtState {
    fn as_byte(&self) -> u8 {
        self.to_owned() as u8
    }

    fn from_byte(byte: u8) -> Result<Self> {
        match byte {
            0x00 => Ok(Self::BootUp),
            0x04 => Ok(Self::Stopped),
            0x05 => Ok(Self::Operational),
            0x7F => Ok(Self::PreOperational),
            _ => Err(Error::InvalidNmtState(byte)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NmtNodeMonitoringFrame {
    pub node_id: NodeId,
    pub state: NmtState,
}

impl NmtNodeMonitoringFrame {
    const FRAME_DATA_SIZE: usize = 1;

    pub fn new(node_id: NodeId, state: NmtState) -> Self {
        Self { node_id, state }
    }

    pub(crate) fn new_with_bytes(node_id: NodeId, bytes: &[u8]) -> Result<Self> {
        if bytes.len() != Self::FRAME_DATA_SIZE {
            return Err(Error::InvalidDataLength {
                length: bytes.len(),
                data_type: "NmtNodeMonitoringFrame".to_owned(),
            });
        }
        Ok(Self::new(node_id, NmtState::from_byte(bytes[0])?))
    }
}

impl From<NmtNodeMonitoringFrame> for CanOpenFrame {
    fn from(frame: NmtNodeMonitoringFrame) -> Self {
        CanOpenFrame::NmtNodeMonitoringFrame(frame)
    }
}

impl ConvertibleFrame for NmtNodeMonitoringFrame {
    fn communication_object(&self) -> CommunicationObject {
        CommunicationObject::NmtNodeMonitoring(self.node_id)
    }

    fn frame_data(&self) -> std::vec::Vec<u8> {
        let mut data = std::vec::Vec::new();
        data.push(self.state.as_byte());
        assert_eq!(data.len(), Self::FRAME_DATA_SIZE);
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nmt_state_to_byte() {
        assert_eq!(NmtState::BootUp.as_byte(), 0x00);
        assert_eq!(NmtState::Stopped.as_byte(), 0x04);
        assert_eq!(NmtState::Operational.as_byte(), 0x05);
        assert_eq!(NmtState::PreOperational.as_byte(), 0x7F);
    }

    #[test]
    fn test_nmt_state_from_byte() {
        assert_eq!(NmtState::from_byte(0x00), Ok(NmtState::BootUp));
        assert_eq!(NmtState::from_byte(0x01), Err(Error::InvalidNmtState(0x01)));
        assert_eq!(NmtState::from_byte(0x02), Err(Error::InvalidNmtState(0x02)));
        assert_eq!(NmtState::from_byte(0x03), Err(Error::InvalidNmtState(0x03)));
        assert_eq!(NmtState::from_byte(0x04), Ok(NmtState::Stopped));
        assert_eq!(NmtState::from_byte(0x05), Ok(NmtState::Operational));
        assert_eq!(NmtState::from_byte(0x06), Err(Error::InvalidNmtState(0x06)));
        assert_eq!(NmtState::from_byte(0x7E), Err(Error::InvalidNmtState(0x7E)));
        assert_eq!(NmtState::from_byte(0x7F), Ok(NmtState::PreOperational));
        assert_eq!(NmtState::from_byte(0x80), Err(Error::InvalidNmtState(0x80)));
        assert_eq!(NmtState::from_byte(0xFF), Err(Error::InvalidNmtState(0xFF)));
    }

    #[test]
    fn test_from_node_id_bytes() {
        assert_eq!(
            NmtNodeMonitoringFrame::new_with_bytes(1.try_into().unwrap(), &[0x00]),
            Ok(NmtNodeMonitoringFrame {
                node_id: 1.try_into().unwrap(),
                state: NmtState::BootUp
            })
        );
        assert_eq!(
            NmtNodeMonitoringFrame::new_with_bytes(2.try_into().unwrap(), &[0x04]),
            Ok(NmtNodeMonitoringFrame {
                node_id: 2.try_into().unwrap(),
                state: NmtState::Stopped
            })
        );
        assert_eq!(
            NmtNodeMonitoringFrame::new_with_bytes(3.try_into().unwrap(), &[0x05]),
            Ok(NmtNodeMonitoringFrame {
                node_id: 3.try_into().unwrap(),
                state: NmtState::Operational
            })
        );
        assert_eq!(
            NmtNodeMonitoringFrame::new_with_bytes(4.try_into().unwrap(), &[0x7F]),
            Ok(NmtNodeMonitoringFrame {
                node_id: 4.try_into().unwrap(),
                state: NmtState::PreOperational
            })
        );

        assert_eq!(
            NmtNodeMonitoringFrame::new_with_bytes(5.try_into().unwrap(), &[0x01]),
            Err(Error::InvalidNmtState(0x01))
        );
        assert_eq!(
            NmtNodeMonitoringFrame::new_with_bytes(6.try_into().unwrap(), &[0x06]),
            Err(Error::InvalidNmtState(0x06))
        );
        assert_eq!(
            NmtNodeMonitoringFrame::new_with_bytes(7.try_into().unwrap(), &[0x80]),
            Err(Error::InvalidNmtState(0x80))
        );
    }

    #[test]
    fn test_communication_object() {
        assert_eq!(
            NmtNodeMonitoringFrame::new(1.try_into().unwrap(), NmtState::BootUp)
                .communication_object(),
            CommunicationObject::NmtNodeMonitoring(1.try_into().unwrap())
        );
        assert_eq!(
            NmtNodeMonitoringFrame::new(2.try_into().unwrap(), NmtState::Stopped)
                .communication_object(),
            CommunicationObject::NmtNodeMonitoring(2.try_into().unwrap())
        );
        assert_eq!(
            NmtNodeMonitoringFrame::new(3.try_into().unwrap(), NmtState::Operational)
                .communication_object(),
            CommunicationObject::NmtNodeMonitoring(3.try_into().unwrap())
        );
        assert_eq!(
            NmtNodeMonitoringFrame::new(4.try_into().unwrap(), NmtState::PreOperational)
                .communication_object(),
            CommunicationObject::NmtNodeMonitoring(4.try_into().unwrap())
        );
    }

    #[test]
    fn test_set_data() {
        let mut buf = [0u8; 8];

        let data =
            NmtNodeMonitoringFrame::new(1.try_into().unwrap(), NmtState::BootUp).frame_data();
        assert_eq!(data.len(), 1);
        assert_eq!(data, &[0x00]);

        buf.fill(0x00);
        let data =
            NmtNodeMonitoringFrame::new(2.try_into().unwrap(), NmtState::Stopped).frame_data();
        assert_eq!(data.len(), 1);
        assert_eq!(data, &[0x04]);

        buf.fill(0x00);
        let data =
            NmtNodeMonitoringFrame::new(3.try_into().unwrap(), NmtState::Operational).frame_data();
        assert_eq!(data.len(), 1);
        assert_eq!(data, &[0x05]);

        buf.fill(0x00);
        let data = NmtNodeMonitoringFrame::new(4.try_into().unwrap(), NmtState::PreOperational)
            .frame_data();
        assert_eq!(data.len(), 1);
        assert_eq!(data, &[0x7F]);
    }
}
