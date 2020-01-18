// TODO - should bind value type to ID at code-gen time?

use crate::error::Error;
use crate::sys_clock;
use crate::PARAM_EVENT_Q;
use heapless::Vec;
use log::debug;
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

    pub fn add(&mut self, mut parameter: Parameter) -> Result<(), Error> {
        debug!("Adding parameter ID {}", parameter.id());
        if self.params.iter().any(|p| p.id() == parameter.id()) {
            Err(Error::Duplicate)
        } else {
            parameter.set_local_time_ms(sys_clock::system_millis());
            self.params.push(parameter).map_err(|_| Error::Capacity)?;
            let p: &mut [Parameter] = self.params.as_mut();
            // Sort them so all the parameters with bcast flag set
            // are at the head
            p.sort_unstable_by(|a, b| b.flags().broadcast().cmp(&a.flags().broadcast()));
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
                    p.set_local_time_ms(sys_clock::system_millis());
                    Ok(())
                }
            })
    }

    // TODO
    //
    // add local_time_ms updating
    //
    // async/queue setter fn's
    // heapless::mpmc::Q (static is safe in interrupt context)
    // static Event Q in main, deq. and call update/process_event?
    // wrapper methods to push_event() can be used anywhere
    pub fn process_event(&mut self, event: Event) -> Result<(), Error> {
        self.set(event.id, event.value)
    }

    pub fn get_all_broadcast(&self) -> &[Parameter] {
        // Expects to be sorted, broadcast flags up front
        let num_bcast = self
            .params
            .iter()
            // From the right, stop at the first bcast
            .rposition(|p| p.flags().broadcast() == true)
            .map(|index| index + 1)
            .unwrap_or(0);
        &self.params[..num_bcast]
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
    use pretty_assertions::assert_eq;

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
        assert_eq!(params.get_all_broadcast().len(), 0);
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

    #[test]
    fn sorted_by_bcast_flag() {
        let mut params = Params::new();

        let mut flags = ParameterFlags::default();
        flags.set_broadcast(false);

        let mut ro_flags = ParameterFlags::default();
        ro_flags.set_read_only(true);

        let mut bcast_flags = ParameterFlags::default();
        bcast_flags.set_broadcast(true);

        let p1 = Parameter::new_with_value(ParameterId::new(1), flags, ParameterValue::U8(123));
        let p2 = Parameter::new_with_value(ParameterId::new(2), flags, ParameterValue::U8(123));
        let p3 =
            Parameter::new_with_value(ParameterId::new(3), ro_flags, ParameterValue::Bool(false));
        let p4 = Parameter::new_with_value(ParameterId::new(4), flags, ParameterValue::U8(123));
        let p5 =
            Parameter::new_with_value(ParameterId::new(5), ro_flags, ParameterValue::Bool(false));
        let p6 =
            Parameter::new_with_value(ParameterId::new(6), bcast_flags, ParameterValue::I32(-123));
        let p7 =
            Parameter::new_with_value(ParameterId::new(7), bcast_flags, ParameterValue::I32(-123));
        let p8 = Parameter::new_with_value(ParameterId::new(8), flags, ParameterValue::U8(123));

        assert_eq!(params.add(p1), Ok(()));
        assert_eq!(params.add(p2), Ok(()));
        assert_eq!(params.add(p3), Ok(()));
        assert_eq!(params.add(p4), Ok(()));
        assert_eq!(params.add(p5), Ok(()));
        assert_eq!(params.get_all_broadcast(), &[]);

        assert_eq!(params.add(p6), Ok(()));
        assert_eq!(params.add(p7), Ok(()));
        assert_eq!(params.add(p8), Ok(()));
        assert_eq!(params.params.len(), 8);

        // p6, p7 should be the first 2
        assert_eq!(params.params[0], p6);
        assert_eq!(params.params[1], p7);
        assert_eq!(params.get_all_broadcast(), &[p6, p7]);
    }

    #[test]
    fn all_bcast() {
        let mut params = Params::new();

        let mut flags = ParameterFlags::default();
        flags.set_broadcast(true);
        flags.set_read_only(true);

        let p1 = Parameter::new_with_value(ParameterId::new(1), flags, ParameterValue::U8(123));
        let p2 = Parameter::new_with_value(ParameterId::new(2), flags, ParameterValue::U8(123));
        assert_eq!(params.add(p1), Ok(()));
        assert_eq!(params.add(p2), Ok(()));
        assert_eq!(params.params.len(), 2);

        assert_eq!(params.params[0], p1);
        assert_eq!(params.params[1], p2);
        assert_eq!(params.get_all_broadcast(), &[p1, p2]);
    }
}
