// TODO - should bind value type to ID at code-gen time?

use crate::error::Error;
use heapless::Vec;
use params::{MaxParamsPerOp, Parameter, ParameterFlags, ParameterId, ParameterValue};

// Not sure having these make sense?
// could be auto-pushed into the vec, make it a `SYS_RO_PARAMS[]`
pub const CONST_RO_PARAM: Parameter = Parameter::new_with_value(
    ParameterId::new(1),
    ParameterFlags::new_read_only(),
    ParameterValue::U8(123),
);

pub struct Params {
    params: Vec<Parameter, MaxParamsPerOp>,
}

pub struct Event {
    pub id: ParameterId,
    pub value: ParameterValue,
}

impl Params {
    pub fn new() -> Self {
        Params { params: Vec::new() }
    }

    pub fn push(&mut self, parameter: Parameter) -> Result<(), Error> {
        if self.params.iter().any(|p| p.id() == parameter.id()) {
            Err(Error::Duplicate)
        } else {
            self.params.push(parameter).map_err(|_| Error::Capacity)?;
            Ok(())
        }
    }

    // getter fn's
    //
    // get(id) | get_param(id)
    // get_broadcast() -> []
    // get_read_only()

    // setter fn's
    //
    // set(id, val) | set_param(id, val)
    pub fn set(&mut self, id: ParameterId, value: ParameterValue) -> Result<(), Error> {
        self.params
            .iter_mut()
            .find(|p| p.id() == id)
            .map_or(Ok(()), |p| {
                if p.flags().read_only() {
                    Err(Error::PermissionDenied)
                } else {
                    p.set_value(value)?;
                    Ok(())
                }
            })
    }

    // TODO
    // async/queue setter fn's
    // heapless::mpmc::Q (static is safe in interrupt context)
    // static Event Q in main, deq. and call update/process_event?
    // wrapper methods to push_event() can be used anywhere
    pub fn process_event(&mut self, event: Event) -> Result<(), Error> {
        self.set(event.id, event.value)
    }
}

impl AsRef<[Parameter]> for Params {
    fn as_ref(&self) -> &[Parameter] {
        self.params.as_ref()
    }
}
