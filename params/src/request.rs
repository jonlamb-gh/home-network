use crate::{
    Error, GetSetFrame, GetSetOp, MaxParamsPerOp, Parameter, ParameterId, ParameterIdListPacket,
    ParameterListPacket, PREAMBLE_WORD,
};
use heapless::Vec;

// TODO - revist this impl, could be much more memory conscious
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Request {
    op: GetSetOp,
    ids: Vec<ParameterId, MaxParamsPerOp>,
    params: Vec<Parameter, MaxParamsPerOp>,
}

impl Request {
    pub fn new(op: GetSetOp) -> Self {
        Request {
            op,
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
        let payload_size = match self.op {
            GetSetOp::ListAll => 0,
            GetSetOp::Get => ParameterListPacket::<&[u8]>::buffer_len(
                self.ids.iter().map(|id| id.wire_size()).sum(),
            ),
            GetSetOp::Set => ParameterIdListPacket::<&[u8]>::buffer_len(
                self.params.iter().map(|p| p.wire_size()).sum(),
            ),
        };

        GetSetFrame::<&[u8]>::buffer_len(payload_size)
    }

    pub fn parse<T: AsRef<[u8]> + ?Sized>(frame: &GetSetFrame<&T>) -> Result<Self, Error> {
        frame.check_len()?;
        frame.check_preamble()?;
        let op = frame.op();
        match op {
            GetSetOp::ListAll => Ok(Request::new(op)),
            GetSetOp::Get => {
                let mut r = Request::new(op);
                let p = ParameterIdListPacket::new_checked(frame.payload())?;
                for index in 0..usize::from(p.count()) {
                    r.push_id(p.id_at(index)?)?
                }
                Ok(r)
            }
            GetSetOp::Set => {
                let mut r = Request::new(op);
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
        frame.set_op(self.op);
        match self.op {
            GetSetOp::ListAll => Ok(()),
            GetSetOp::Get => {
                let mut p = ParameterIdListPacket::new_unchecked(frame.payload_mut());
                p.set_count(self.ids.len() as _);
                for (index, id) in self.ids.iter().enumerate() {
                    p.set_id_at(index, *id)?;
                }
                Ok(())
            }
            GetSetOp::Set => {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ParameterFlags, ParameterPacket, ParameterValue};
    use core::mem;

    #[test]
    fn wire_size() {
        let req = Request::new(GetSetOp::ListAll);
        assert_eq!(req.ids.len(), 0);
        assert_eq!(req.params.len(), 0);
        assert_eq!(req.wire_size(), GetSetFrame::<&[u8]>::header_len());

        let mut req = Request::new(GetSetOp::Set);
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

        let mut req = Request::new(GetSetOp::Get);
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
        let req = Request::new(GetSetOp::ListAll);
        let mut bytes = [0xFF; 64];
        let mut frame = GetSetFrame::new_unchecked(&mut bytes[..]);
        assert_eq!(req.emit(&mut frame), Ok(()));
        assert_eq!(frame.check_len(), Ok(()));
        assert_eq!(frame.check_preamble(), Ok(()));

        let mut req = Request::new(GetSetOp::Set);
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
        let packet = ParameterListPacket::new_checked(frame.payload_mut()).unwrap();
        assert_eq!(packet.check_len(), Ok(()));
        assert_eq!(packet.count(), 2);
        assert_eq!(packet.parameter_at(0), Ok(p_a));
        assert_eq!(packet.parameter_at(1), Ok(p_b));

        let mut req = Request::new(GetSetOp::Get);
        let id_a = ParameterId::new(0x0A);
        let id_b = ParameterId::new(0x0B);
        let id_c = ParameterId::new(0x0C);
        assert_eq!(req.push_id(id_a), Ok(()));
        assert_eq!(req.push_id(id_b), Ok(()));
        assert_eq!(req.push_id(id_c), Ok(()));
        assert_eq!(req.emit(&mut frame), Ok(()));
        let packet = ParameterIdListPacket::new_checked(frame.payload_mut()).unwrap();
        assert_eq!(packet.check_len(), Ok(()));
        assert_eq!(packet.count(), 3);
        assert_eq!(packet.id_at(0), Ok(id_a));
        assert_eq!(packet.id_at(1), Ok(id_b));
        assert_eq!(packet.id_at(2), Ok(id_c));
    }
}
