use crate::value::TypeId;
use crate::{Error, Flags, ParameterId, ParameterValue};
use byteorder::{ByteOrder, LittleEndian};
use static_assertions::{assert_eq_size, const_assert_eq};

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
    pub fn flags(&self) -> Flags {
        let data = self.buffer.as_ref();
        Flags::from(LittleEndian::read_u32(&data[field::FLAGS]))
    }

    #[inline]
    pub fn value(&self) -> ParameterValue {
        let data = self.buffer.as_ref();
        match data[field::VALUE_TYPE_ID] {
            0 => ParameterValue::None,
            1 => ParameterValue::Notification,
            2 => ParameterValue::U8(data[field::VALUE.start]),
            3 => ParameterValue::U32(LittleEndian::read_u32(&data[field::VALUE])),
            4 => ParameterValue::F32(LittleEndian::read_f32(&data[field::VALUE])),
            _ => ParameterValue::None,
        }
    }
}
