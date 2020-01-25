use crate::{Error, Parameter, ParameterPacket, MAX_PARAMS_PER_OP};

#[derive(Debug, Clone)]
pub struct Packet<T: AsRef<[u8]>> {
    buffer: T,
}

mod field {
    use crate::wire::field::*;

    pub const COUNT: usize = 0;
    pub const PARAMS: Rest = 1..;
}

impl<T: AsRef<[u8]>> Packet<T> {
    pub fn new_unchecked(buffer: T) -> Packet<T> {
        Packet { buffer }
    }

    pub fn new_checked(buffer: T) -> Result<Packet<T>, Error> {
        let packet = Self::new_unchecked(buffer);
        packet.check_len()?;
        Ok(packet)
    }

    pub fn check_len(&self) -> Result<(), Error> {
        let len = self.buffer.as_ref().len();
        if len < field::PARAMS.start {
            Err(Error::WireTruncated)
        } else {
            Ok(())
        }
    }

    pub fn into_inner(self) -> T {
        self.buffer
    }

    pub fn header_len() -> usize {
        field::PARAMS.start
    }

    pub fn buffer_len(payload_len: usize) -> usize {
        field::PARAMS.start + payload_len
    }

    #[inline]
    pub fn count(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[field::COUNT]
    }

    #[inline]
    pub fn parameter_at(&self, index: usize) -> Result<Parameter, Error> {
        if index >= usize::from(self.count()) {
            return Err(Error::WireIndexOutOfBounds);
        }
        let data = self.buffer.as_ref();
        let mut offset = field::PARAMS.start;
        for _ in 0..index {
            let f = ParameterPacket::new_checked(&data[offset..])?;
            offset += ParameterPacket::<&[u8]>::buffer_len(f.value_type_id().wire_size());
        }

        // Cursor now at index
        Parameter::parse(&ParameterPacket::new_unchecked(&data[offset..]))
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> Packet<T> {
    #[inline]
    pub fn set_count(&mut self, value: u8) {
        let data = self.buffer.as_mut();
        data[field::COUNT] = value;
    }

    #[inline]
    pub fn set_parameter_at(&mut self, index: usize, value: Parameter) -> Result<(), Error> {
        if index >= MAX_PARAMS_PER_OP {
            return Err(Error::WireIndexOutOfBounds);
        }
        let data = self.buffer.as_mut();
        let mut offset = field::PARAMS.start;
        for _ in 0..index {
            let f = ParameterPacket::new_checked(&data[offset..])?;
            offset += ParameterPacket::<&[u8]>::buffer_len(f.value_type_id().wire_size());
        }

        // Cursor now at index
        let mut f = ParameterPacket::new_checked(&mut data[offset..])?;
        value.emit(&mut f);
        Ok(())
    }
}

impl<T: AsRef<[u8]>> AsRef<[u8]> for Packet<T> {
    fn as_ref(&self) -> &[u8] {
        self.buffer.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Parameter, ParameterFlags, ParameterId, ParameterValue};
    use core::convert::TryInto;
    use pretty_assertions::assert_eq;

    static BYTES: [u8; 134] = [
        7, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 11, 0, 0, 0,
        0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 2, 1, 0, 0, 0, 0, 0, 0, 0,
        0, 13, 0, 0, 0, 0, 0, 0, 0, 3, 171, 0, 0, 0, 0, 0, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 0, 5,
        210, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15, 0, 0, 0, 0, 0, 0, 0, 6, 46, 251, 255, 255, 0, 0,
        0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 9, 182, 243, 157, 191,
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
    fn construct() {
        let mut bytes = [0xFF; 134];
        let mut p = Packet::new_unchecked(&mut bytes[..]);
        assert_eq!(p.check_len(), Ok(()));
        p.set_count(PARAMS.len().try_into().unwrap());
        for (index, param) in PARAMS.iter().enumerate() {
            assert_eq!(p.set_parameter_at(index, *param), Ok(()));
        }
        assert_eq!(&p.into_inner()[..], &BYTES[..]);
    }

    #[test]
    fn deconstruct() {
        let p = Packet::new_checked(&BYTES[..]).unwrap();
        assert_eq!(p.count(), PARAMS.len().try_into().unwrap());
        for index in 0..PARAMS.len() {
            assert_eq!(p.parameter_at(index), Ok(PARAMS[index]));
        }
    }
}
