use crate::value::TypeId;
use crate::{Error, ParameterFlags, ParameterId, ParameterPacket, ParameterValue};
use core::fmt;

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct Parameter {
    local_time_ms: u64,
    id: ParameterId,
    flags: ParameterFlags,
    value: ParameterValue,
}

impl Parameter {
    pub fn new(id: ParameterId, flags: ParameterFlags) -> Self {
        Parameter {
            local_time_ms: 0,
            id,
            flags,
            value: ParameterValue::default(),
        }
    }

    pub const fn new_with_value(
        id: ParameterId,
        flags: ParameterFlags,
        value: ParameterValue,
    ) -> Self {
        Parameter {
            local_time_ms: 0,
            id,
            flags,
            value,
        }
    }

    pub fn local_time_ms(&self) -> u64 {
        self.local_time_ms
    }

    pub fn set_local_time_ms(&mut self, time: u64) {
        self.local_time_ms = time;
    }

    pub fn id(&self) -> ParameterId {
        self.id
    }

    pub fn flags(&self) -> ParameterFlags {
        self.flags
    }

    pub fn value(&self) -> ParameterValue {
        self.value
    }

    pub fn set_value(&mut self, value: ParameterValue) -> Result<(), Error> {
        if self.value.type_id() != value.type_id() {
            Err(Error::ValueTypeMismatch)
        } else {
            self.value = value;
            Ok(())
        }
    }

    pub fn wire_size(&self) -> usize {
        let value_size = TypeId::from(self.value()).wire_size();
        ParameterPacket::<&[u8]>::buffer_len(value_size)
    }

    pub fn parse<T: AsRef<[u8]> + ?Sized>(frame: &ParameterPacket<&T>) -> Result<Self, Error> {
        frame.check_len()?;
        Ok(Parameter {
            local_time_ms: frame.local_time_ms(),
            id: frame.id(),
            flags: frame.flags(),
            value: frame.value(),
        })
    }

    pub fn emit<T: AsRef<[u8]> + AsMut<[u8]>>(&self, frame: &mut ParameterPacket<T>) {
        frame.set_local_time_ms(self.local_time_ms);
        frame.set_id(self.id);
        frame.set_flags(self.flags);
        frame.set_value(self.value);
    }
}

impl fmt::Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Parameter {{ id: {} t: {} flags: {} value: {} }}",
            self.id(),
            self.local_time_ms(),
            self.flags(),
            self.value(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem;

    #[test]
    fn wire_size() {
        let p = Parameter::new_with_value(
            ParameterId::new(0x0F),
            ParameterFlags(0),
            ParameterValue::I32(-1234),
        );
        assert_eq!(
            p.wire_size(),
            ParameterPacket::<&[u8]>::header_len() + mem::size_of::<i32>()
        );
    }

    #[test]
    fn getter_methods() {
        let p = Parameter::new_with_value(
            ParameterId::new(0x0A),
            ParameterFlags::new_read_only(),
            ParameterValue::I32(-1234),
        );
        assert_eq!(p.id(), ParameterId::new(0x0A));
        assert_eq!(p.flags(), ParameterFlags::new_read_only());
        assert_eq!(p.value(), ParameterValue::I32(-1234));
    }

    #[test]
    fn setter_methods() {
        let mut p = Parameter::new_with_value(
            ParameterId::new(0x0A),
            ParameterFlags::new_read_only(),
            ParameterValue::I32(-1234),
        );
        assert_eq!(p.value(), ParameterValue::I32(-1234));
        assert_eq!(p.set_value(ParameterValue::I32(23)), Ok(()));
        assert_eq!(
            p.set_value(ParameterValue::Bool(false)),
            Err(Error::ValueTypeMismatch)
        );
    }
}
