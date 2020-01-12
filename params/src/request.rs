use crate::{Error, GetSetOp, MaxParamsPerOp, ParameterId};
use heapless::Vec;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Request {
    op: GetSetOp,
    ids: Vec<ParameterId, MaxParamsPerOp>,
}

impl Request {
    pub fn new(op: GetSetOp) -> Self {
        Request {
            op,
            ids: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn op(&self) -> GetSetOp {
        self.op
    }

    pub fn push(&mut self, id: ParameterId) -> Result<(), Error> {
        self.ids.push(id).map_err(|_| Error::Capacity)?;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<ParameterId> {
        self.ids.pop()
    }

    pub fn clear(&mut self) {
        self.ids.clear();
    }
}
