use crate::value::TypeId;
use crate::{Error, ParameterFlags, ParameterId, ParameterValue};
use byteorder::{ByteOrder, LittleEndian};
use static_assertions::assert_eq_size;

assert_eq_size!(u32, ParameterId);
assert_eq_size!(u8, TypeId);
//assert_eq_size!(u64, local_time_ms);

#[derive(Debug, Clone)]
pub struct Frame<T: AsRef<[u8]>> {
    buffer: T,
}

mod field {
    use crate::wire::field::*;

    pub const TIME: Field = 0..8;
    pub const ID: Field = 8..12;
    pub const FLAGS: Field = 12..16;
    pub const VALUE_TYPE_ID: usize = 16;
    pub const VALUE: Rest = 17..;
}

impl<T: AsRef<[u8]>> Frame<T> {
    pub fn new_unchecked(buffer: T) -> Frame<T> {
        Frame { buffer }
    }

    pub fn new_checked(buffer: T) -> Result<Frame<T>, Error> {
        let packet = Self::new_unchecked(buffer);
        packet.check_len()?;
        Ok(packet)
    }

    pub fn check_len(&self) -> Result<(), Error> {
        let len = self.buffer.as_ref().len();
        if len < field::VALUE.start {
            Err(Error::WireTruncated)
        } else {
            Ok(())
        }
    }

    pub fn into_inner(self) -> T {
        self.buffer
    }

    pub fn header_len() -> usize {
        field::VALUE.start
    }

    pub fn buffer_len(payload_len: usize) -> usize {
        field::VALUE.start + payload_len
    }

    #[inline]
    pub fn local_time_ms(&self) -> u64 {
        let data = self.buffer.as_ref();
        LittleEndian::read_u64(&data[field::TIME])
    }

    #[inline]
    pub fn id(&self) -> ParameterId {
        let data = self.buffer.as_ref();
        ParameterId::from(LittleEndian::read_u32(&data[field::ID]))
    }

    #[inline]
    pub fn flags(&self) -> ParameterFlags {
        let data = self.buffer.as_ref();
        ParameterFlags::from(LittleEndian::read_u32(&data[field::FLAGS]))
    }

    #[inline]
    pub(crate) fn value_type_id(&self) -> TypeId {
        let data = self.buffer.as_ref();
        TypeId::from(data[field::VALUE_TYPE_ID])
    }

    #[inline]
    pub fn value(&self) -> ParameterValue {
        let data = self.buffer.as_ref();
        match data[field::VALUE_TYPE_ID] {
            0 => ParameterValue::None,
            1 => ParameterValue::Notification,
            2 => ParameterValue::Bool(data[field::VALUE.start] != 0),
            3 => ParameterValue::U8(data[field::VALUE.start]),
            4 => ParameterValue::U32(LittleEndian::read_u32(&data[field::VALUE])),
            5 => ParameterValue::F32(LittleEndian::read_f32(&data[field::VALUE])),
            _ => ParameterValue::None,
        }
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> Frame<T> {
    #[inline]
    pub fn set_local_time_ms(&mut self, value: u64) {
        let data = self.buffer.as_mut();
        LittleEndian::write_u64(&mut data[field::TIME], value)
    }

    #[inline]
    pub fn set_id(&mut self, value: ParameterId) {
        let data = self.buffer.as_mut();
        LittleEndian::write_u32(&mut data[field::ID], value.into())
    }

    #[inline]
    pub fn set_flags(&mut self, value: ParameterFlags) {
        let data = self.buffer.as_mut();
        LittleEndian::write_u32(&mut data[field::FLAGS], value.into())
    }

    #[inline]
    pub fn set_value(&mut self, value: ParameterValue) {
        let data = self.buffer.as_mut();
        data[field::VALUE_TYPE_ID] = TypeId::from(value).as_u8();
        match value {
            ParameterValue::None => (),
            ParameterValue::Notification => (),
            ParameterValue::Bool(inner) => {
                data[field::VALUE.start] = inner as u8;
            }
            ParameterValue::U8(inner) => {
                data[field::VALUE.start] = inner;
            }
            ParameterValue::U32(inner) => LittleEndian::write_u32(&mut data[field::VALUE], inner),
            ParameterValue::F32(inner) => LittleEndian::write_f32(&mut data[field::VALUE], inner),
        }
    }
}

impl<T: AsRef<[u8]>> AsRef<[u8]> for Frame<T> {
    fn as_ref(&self) -> &[u8] {
        self.buffer.as_ref()
    }
}
