use crate::{Error, GetSetOp, MaxParamsPerOp, Parameter};
use heapless::Vec;

// TODO - consider a vec of references so caller
// can build the vec via pointers to params in various places?
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Response {
    op: GetSetOp,
    params: Vec<Parameter, MaxParamsPerOp>,
}

impl Response {
    pub fn new(op: GetSetOp) -> Self {
        Response {
            op,
            params: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.params.len()
    }

    pub fn op(&self) -> GetSetOp {
        self.op
    }

    pub fn push(&mut self, param: Parameter) -> Result<(), Error> {
        self.params.push(param).map_err(|_| Error::Capacity)?;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<Parameter> {
        self.params.pop()
    }

    pub fn clear(&mut self) {
        self.params.clear();
    }
}
