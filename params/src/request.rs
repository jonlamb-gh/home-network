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

    pub fn len(&self) -> usize {
        self.ids.len()
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
