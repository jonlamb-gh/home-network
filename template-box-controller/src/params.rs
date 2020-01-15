// TODO - should bind value type to ID at code-gen time?

use crate::error::Error;
use crate::PARAM_EVENT_Q;
use heapless::Vec;
use params::{MaxParamsPerOp, Parameter, ParameterId, ParameterValue};

pub struct Params {
    params: Vec<Parameter, MaxParamsPerOp>,
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct Event {
    pub id: ParameterId,
    pub value: ParameterValue,
}

impl Event {
    pub fn new(id: ParameterId, value: ParameterValue) -> Self {
        Event { id, value }
    }
}

pub fn push_event(event: Event) -> Result<(), Error> {
    PARAM_EVENT_Q.enqueue(event).map_err(|_| Error::Capacity)
}

pub fn pop_event() -> Option<Event> {
    PARAM_EVENT_Q.dequeue()
}

impl Params {
    pub fn new() -> Self {
        Params { params: Vec::new() }
    }

    pub fn add(&mut self, parameter: Parameter) -> Result<(), Error> {
        if self.params.iter().any(|p| p.id() == parameter.id()) {
            Err(Error::Duplicate)
        } else {
            self.params.push(parameter).map_err(|_| Error::Capacity)?;
            Ok(())
        }
    }

    // TODO
    // get_broadcast() -> []
    // get_read_only()
    pub fn get(&self, id: ParameterId) -> Option<ParameterValue> {
        self.params.iter().find(|p| p.id() == id).map(|p| p.value())
    }

    pub fn set(&mut self, id: ParameterId, value: ParameterValue) -> Result<(), Error> {
        self.params
            .iter_mut()
            .find(|p| p.id() == id)
            .map_or(Err(Error::NotFound), |p| {
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

#[cfg(test)]
mod tests {
    use super::*;
    use core::convert::TryInto;
    use params::{ParameterFlags, MAX_PARAMS_PER_OP};

    #[test]
    fn event_queue_capacity() {
        for i in 0..32 {
            let e = Event::new(i.into(), ParameterValue::U32(i));
            assert_eq!(push_event(e), Ok(()));
        }
        for i in 0..32 {
            let e = Event::new(i.into(), ParameterValue::U32(i));
            assert_eq!(pop_event(), Some(e));
        }
    }

    #[test]
    fn capacity() {
        let mut params = Params::new();
        for i in 0..=MAX_PARAMS_PER_OP {
            let p = Parameter::new_with_value(
                ParameterId::new(i.try_into().unwrap()),
                ParameterFlags::new_read_only(),
                ParameterValue::U32(i.try_into().unwrap()),
            );
            if i >= MAX_PARAMS_PER_OP {
                assert_eq!(params.add(p), Err(Error::Capacity));
            } else {
                assert_eq!(params.add(p), Ok(()));
            }
        }
        assert_eq!(params.params.len(), MAX_PARAMS_PER_OP);
    }

    #[test]
    fn add_duplicate_error() {
        let p = Parameter::new_with_value(
            ParameterId::new(1),
            ParameterFlags::new_read_only(),
            ParameterValue::U8(123),
        );
        let mut params = Params::new();
        assert_eq!(params.add(p), Ok(()));
        assert_eq!(params.add(p), Err(Error::Duplicate));
        assert_eq!(params.params.len(), 1);
    }

    #[test]
    fn set_not_found_error() {
        let id = ParameterId::new(1);
        let value = ParameterValue::U8(2);
        let mut params = Params::new();
        assert_eq!(params.set(id, value), Err(Error::NotFound));
        assert_eq!(params.params.len(), 0);
    }

    #[test]
    fn set_read_only_error() {
        let p = Parameter::new_with_value(
            ParameterId::new(1),
            ParameterFlags::new_read_only(),
            ParameterValue::U8(123),
        );
        let mut params = Params::new();
        assert_eq!(params.add(p), Ok(()));
        assert_eq!(
            params.set(p.id(), ParameterValue::U8(2)),
            Err(Error::PermissionDenied)
        );
    }

    #[test]
    fn set_type_mismatch_error() {
        let p = Parameter::new_with_value(
            ParameterId::new(1),
            ParameterFlags::default(),
            ParameterValue::U8(123),
        );
        let mut params = Params::new();
        assert_eq!(params.add(p), Ok(()));
        assert_eq!(
            params.set(p.id(), ParameterValue::Bool(false)),
            Err(Error::ParamsError(params::Error::ValueTypeMismatch))
        );
    }

    #[test]
    fn set_updates_value() {
        let p = Parameter::new_with_value(
            ParameterId::new(1),
            ParameterFlags::default(),
            ParameterValue::U8(123),
        );
        let mut params = Params::new();
        assert_eq!(params.add(p), Ok(()));
        assert_eq!(params.get(p.id()), Some(p.value()));
        assert_eq!(params.set(p.id(), ParameterValue::U8(2)), Ok(()));
        assert_eq!(params.get(p.id()), Some(ParameterValue::U8(2)));
    }
}
