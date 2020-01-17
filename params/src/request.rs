use crate::{
    Error, GetSetFlags, GetSetFrame, GetSetNodeId, GetSetOp, GetSetPayloadType, MaxParamsPerOp,
    Parameter, ParameterId, ParameterIdListPacket, ParameterListPacket, PREAMBLE_WORD,
};
use core::fmt;
use heapless::Vec;

// TODO - revist this impl, could be much more memory conscious
// wrap a slice of refs instead?
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Request {
    node_id: GetSetNodeId,
    flags: GetSetFlags,
    op: GetSetOp,
    payload_type: GetSetPayloadType,
    ids: Vec<ParameterId, MaxParamsPerOp>,
    params: Vec<Parameter, MaxParamsPerOp>,
}

impl Request {
    pub fn new(
        node_id: GetSetNodeId,
        flags: GetSetFlags,
        op: GetSetOp,
        payload_type: GetSetPayloadType,
    ) -> Self {
        Request {
            node_id,
            flags,
            op,
            payload_type,
            ids: Vec::new(),
            params: Vec::new(),
        }
    }

    pub fn op(&self) -> GetSetOp {
        self.op
    }

    pub fn push_id(&mut self, id: ParameterId) -> Result<(), Error> {
        self.ids.push(id).map_err(|_| Error::Capacity)?;
        Ok(())
    }

    pub fn push_parameter(&mut self, parameter: Parameter) -> Result<(), Error> {
        self.params.push(parameter).map_err(|_| Error::Capacity)?;
        Ok(())
    }

    pub fn pop_id(&mut self) -> Option<ParameterId> {
        self.ids.pop()
    }

    pub fn pop_parameter(&mut self) -> Option<Parameter> {
        self.params.pop()
    }

    pub fn clear(&mut self) {
        self.ids.clear();
        self.params.clear();
    }

    pub fn wire_size(&self) -> usize {
        GetSetFrame::<&[u8]>::buffer_len(self.payload_wire_size())
    }

    fn payload_wire_size(&self) -> usize {
        match self.payload_type {
            GetSetPayloadType::None => 0,
            GetSetPayloadType::ParameterIdListPacket => ParameterIdListPacket::<&[u8]>::buffer_len(
                self.ids.iter().map(|id| id.wire_size()).sum(),
            ),
            GetSetPayloadType::ParameterListPacket => ParameterListPacket::<&[u8]>::buffer_len(
                self.params.iter().map(|p| p.wire_size()).sum(),
            ),
        }
    }

    pub fn parse<T: AsRef<[u8]> + ?Sized>(frame: &GetSetFrame<&T>) -> Result<Self, Error> {
        frame.check_len()?;
        frame.check_preamble()?;
        let node_id = frame.node_id();
        let flags = frame.flags();
        let _ver = frame.version();
        let op = frame.op();
        let payload_type = frame.payload_type();

        match payload_type {
            GetSetPayloadType::None => Ok(Request::new(node_id, flags, op, payload_type)),
            GetSetPayloadType::ParameterIdListPacket => {
                let mut r = Request::new(node_id, flags, op, payload_type);
                let p = ParameterIdListPacket::new_checked(frame.payload())?;
                for index in 0..usize::from(p.count()) {
                    r.push_id(p.id_at(index)?)?
                }
                Ok(r)
            }
            GetSetPayloadType::ParameterListPacket => {
                let mut r = Request::new(node_id, flags, op, payload_type);
                let p = ParameterListPacket::new_checked(frame.payload())?;
                for index in 0..usize::from(p.count()) {
                    r.push_parameter(p.parameter_at(index)?)?
                }
                Ok(r)
            }
        }
    }

    pub fn emit<T: AsRef<[u8]> + AsMut<[u8]>>(
        &self,
        frame: &mut GetSetFrame<T>,
    ) -> Result<(), Error> {
        frame.set_preamble(PREAMBLE_WORD);
        frame.set_node_id(self.node_id);
        frame.set_flags(self.flags);
        frame.set_version(1);
        frame.set_op(self.op);
        frame.set_payload_type(self.payload_type);
        frame.set_payload_size(self.payload_wire_size() as u16);
        match self.payload_type {
            GetSetPayloadType::None => Ok(()),
            GetSetPayloadType::ParameterIdListPacket => {
                let mut p = ParameterIdListPacket::new_unchecked(frame.payload_mut());
                p.set_count(self.ids.len() as _);
                for (index, id) in self.ids.iter().enumerate() {
                    p.set_id_at(index, *id)?;
                }
                Ok(())
            }
            GetSetPayloadType::ParameterListPacket => {
                let mut p = ParameterListPacket::new_unchecked(frame.payload_mut());
                p.set_count(self.params.len() as _);
                for (index, param) in self.params.iter().enumerate() {
                    p.set_parameter_at(index, *param)?;
                }
                Ok(())
            }
        }
    }
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Request {{ op: {} }}", self.op())?;
        for p in &self.params {
            writeln!(f, "{}", p)?;
        }
        for id in &self.ids {
            writeln!(f, "{}", id)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GetSetPayloadType, ParameterFlags, ParameterPacket, ParameterValue};
    use core::convert::TryInto;
    use core::mem;

    static FRAME_BYTES: [u8; 151] = [
        0xAB, 0xCD, 0xEF, 0xFF, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x02,
        0x86, 0x00, 7, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        11, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 2, 1, 0, 0, 0,
        0, 0, 0, 0, 0, 13, 0, 0, 0, 0, 0, 0, 0, 3, 171, 0, 0, 0, 0, 0, 0, 0, 0, 14, 0, 0, 0, 0, 0,
        0, 0, 4, 210, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15, 0, 0, 0, 0, 0, 0, 0, 5, 46, 251, 255,
        255, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 6, 182, 243, 157, 191,
    ];

    static PAYLOAD_BYTES: [u8; 134] = [
        7, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 11, 0, 0, 0,
        0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 2, 1, 0, 0, 0, 0, 0, 0, 0,
        0, 13, 0, 0, 0, 0, 0, 0, 0, 3, 171, 0, 0, 0, 0, 0, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 0, 4,
        210, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15, 0, 0, 0, 0, 0, 0, 0, 5, 46, 251, 255, 255, 0, 0,
        0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 6, 182, 243, 157, 191,
    ];

    static PARAMS: [Parameter; 7] = [
        Parameter::new_with_value(
            ParameterId::new(0x0A),
            ParameterFlags(0),
            ParameterValue::None,
        ),
        Parameter::new_with_value(
            ParameterId::new(0x0B),
            ParameterFlags(0),
            ParameterValue::Notification,
        ),
        Parameter::new_with_value(
            ParameterId::new(0x0C),
            ParameterFlags(0),
            ParameterValue::Bool(true),
        ),
        Parameter::new_with_value(
            ParameterId::new(0x0D),
            ParameterFlags(0),
            ParameterValue::U8(0xAB),
        ),
        Parameter::new_with_value(
            ParameterId::new(0x0E),
            ParameterFlags(0),
            ParameterValue::U32(1234),
        ),
        Parameter::new_with_value(
            ParameterId::new(0x0F),
            ParameterFlags(0),
            ParameterValue::I32(-1234),
        ),
        Parameter::new_with_value(
            ParameterId::new(0x10),
            ParameterFlags(0),
            ParameterValue::F32(-1.234),
        ),
    ];

    #[test]
    fn wire_size() {
        let req = Request::new(0, 0, GetSetOp::ListAll, GetSetPayloadType::None);
        assert_eq!(req.ids.len(), 0);
        assert_eq!(req.params.len(), 0);
        assert_eq!(req.wire_size(), GetSetFrame::<&[u8]>::header_len());

        let mut req = Request::new(0, 0, GetSetOp::Set, GetSetPayloadType::ParameterListPacket);
        assert_eq!(
            req.push_parameter(Parameter::new_with_value(
                ParameterId::new(0x0A),
                ParameterFlags(0),
                ParameterValue::I32(-1234),
            )),
            Ok(())
        );
        assert_eq!(
            req.push_parameter(Parameter::new_with_value(
                ParameterId::new(0x0B),
                ParameterFlags(0),
                ParameterValue::Bool(true),
            )),
            Ok(())
        );
        assert_eq!(req.ids.len(), 0);
        assert_eq!(req.params.len(), 2);
        assert_eq!(
            req.wire_size(),
            GetSetFrame::<&[u8]>::header_len()
                + ParameterListPacket::<&[u8]>::header_len()
                + ParameterPacket::<&[u8]>::header_len()
                + mem::size_of::<i32>()
                + ParameterPacket::<&[u8]>::header_len()
                + mem::size_of::<u8>()
        );

        let mut req = Request::new(
            0,
            0,
            GetSetOp::Get,
            GetSetPayloadType::ParameterIdListPacket,
        );
        assert_eq!(req.push_id(ParameterId::new(0x0A)), Ok(()));
        assert_eq!(req.push_id(ParameterId::new(0x0B)), Ok(()));
        assert_eq!(req.ids.len(), 2);
        assert_eq!(req.params.len(), 0);
        assert_eq!(
            req.wire_size(),
            GetSetFrame::<&[u8]>::header_len()
                + ParameterIdListPacket::<&[u8]>::header_len()
                + mem::size_of::<u32>()
                + mem::size_of::<u32>()
        );
    }

    #[test]
    fn emit() {
        let req = Request::new(0, 0, GetSetOp::ListAll, GetSetPayloadType::None);
        assert_eq!(req.op(), GetSetOp::ListAll);
        let mut bytes = [0xFF; 64];
        let mut frame = GetSetFrame::new_unchecked(&mut bytes[..]);
        assert_eq!(req.emit(&mut frame), Ok(()));
        assert_eq!(frame.check_len(), Ok(()));
        assert_eq!(frame.check_preamble(), Ok(()));
        assert_eq!(frame.op(), GetSetOp::ListAll);
        assert_eq!(frame.payload_size(), req.payload_wire_size() as u16);

        let mut req = Request::new(0, 0, GetSetOp::Set, GetSetPayloadType::ParameterListPacket);
        assert_eq!(req.op(), GetSetOp::Set);
        let p_a = Parameter::new_with_value(
            ParameterId::new(0x0A),
            ParameterFlags(0),
            ParameterValue::I32(-1234),
        );
        let p_b = Parameter::new_with_value(
            ParameterId::new(0x0B),
            ParameterFlags(0),
            ParameterValue::Bool(true),
        );
        assert_eq!(req.push_parameter(p_a), Ok(()));
        assert_eq!(req.push_parameter(p_b), Ok(()));

        let mut bytes = [0xFF; 64];
        let mut frame = GetSetFrame::new_unchecked(&mut bytes[..]);
        assert_eq!(req.emit(&mut frame), Ok(()));
        assert_eq!(frame.check_len(), Ok(()));
        assert_eq!(frame.check_preamble(), Ok(()));
        assert_eq!(frame.op(), GetSetOp::Set);
        assert_eq!(frame.payload_size(), req.payload_wire_size() as u16);
        let packet = ParameterListPacket::new_checked(frame.payload_mut()).unwrap();
        assert_eq!(packet.check_len(), Ok(()));
        assert_eq!(packet.count(), 2);
        assert_eq!(packet.parameter_at(0), Ok(p_a));
        assert_eq!(packet.parameter_at(1), Ok(p_b));

        let mut req = Request::new(
            0,
            0,
            GetSetOp::Get,
            GetSetPayloadType::ParameterIdListPacket,
        );
        assert_eq!(req.op(), GetSetOp::Get);
        let id_a = ParameterId::new(0x0A);
        let id_b = ParameterId::new(0x0B);
        let id_c = ParameterId::new(0x0C);
        assert_eq!(req.push_id(id_a), Ok(()));
        assert_eq!(req.push_id(id_b), Ok(()));
        assert_eq!(req.push_id(id_c), Ok(()));
        assert_eq!(req.emit(&mut frame), Ok(()));
        assert_eq!(frame.check_len(), Ok(()));
        assert_eq!(frame.check_preamble(), Ok(()));
        assert_eq!(frame.op(), GetSetOp::Get);
        assert_eq!(frame.payload_size(), req.payload_wire_size() as u16);
        let packet = ParameterIdListPacket::new_checked(frame.payload_mut()).unwrap();
        assert_eq!(packet.check_len(), Ok(()));
        assert_eq!(packet.count(), 3);
        assert_eq!(packet.id_at(0), Ok(id_a));
        assert_eq!(packet.id_at(1), Ok(id_b));
        assert_eq!(packet.id_at(2), Ok(id_c));
    }

    #[test]
    fn parse() {
        let f = GetSetFrame::new_checked(&FRAME_BYTES[..]).unwrap();
        assert_eq!(f.preamble(), PREAMBLE_WORD);
        assert_eq!(f.node_id(), 0x01);
        assert_eq!(f.flags(), 0);
        assert_eq!(f.version(), 1);
        assert_eq!(f.op(), GetSetOp::Set);
        assert_eq!(f.payload_type(), GetSetPayloadType::ParameterListPacket);
        assert_eq!(f.payload_size(), PAYLOAD_BYTES.len().try_into().unwrap());
        assert_eq!(f.payload(), &PAYLOAD_BYTES[..]);
        let p = ParameterListPacket::new_checked(f.payload()).unwrap();
        assert_eq!(p.count(), PARAMS.len().try_into().unwrap());
        for index in 0..PARAMS.len() {
            assert_eq!(p.parameter_at(index), Ok(PARAMS[index]));
        }

        let req = Request::parse(&f).unwrap();
        assert_eq!(req.params.len(), PARAMS.len());
        for (index, p) in req.params.iter().enumerate() {
            assert_eq!(*p, PARAMS[index]);
        }
    }
}
