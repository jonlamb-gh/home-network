use crate::{Error, Flags, ParameterFrame, ParameterId, ParameterValue};

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct Parameter {
    local_time_ms: u64,
    id: ParameterId,
    flags: Flags,
    value: ParameterValue,
}

impl Parameter {
    pub fn parse<T: AsRef<[u8]> + ?Sized>(frame: &ParameterFrame<&T>) -> Result<Self, Error> {
        frame.check_len()?;
        Ok(Parameter {
            local_time_ms: frame.local_time_ms(),
            id: frame.id(),
            flags: frame.flags(),
            value: frame.value(),
        })
    }
}
