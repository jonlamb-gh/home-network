use crate::value::TypeId;
use crate::{Error, ParameterFlags, ParameterId, ParameterValue};
use byteorder::{ByteOrder, LittleEndian};
use static_assertions::assert_eq_size;

assert_eq_size!(u32, ParameterId);
assert_eq_size!(u8, TypeId);
//assert_eq_size!(u64, local_time_ms);

#[derive(Debug, Clone)]
pub struct Packet<T: AsRef<[u8]>> {
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
        match TypeId::from(data[field::VALUE_TYPE_ID]) {
            TypeId::None => ParameterValue::None,
            TypeId::Notification => ParameterValue::Notification,
            TypeId::Bool => ParameterValue::Bool(data[field::VALUE.start] != 0),
            TypeId::U8 => ParameterValue::U8(data[field::VALUE.start]),
            TypeId::I8 => ParameterValue::I8(data[field::VALUE.start] as _),
            TypeId::U32 => ParameterValue::U32(LittleEndian::read_u32(&data[field::VALUE])),
            TypeId::I32 => ParameterValue::I32(LittleEndian::read_i32(&data[field::VALUE])),
            TypeId::U64 => ParameterValue::U64(LittleEndian::read_u64(&data[field::VALUE])),
            TypeId::I64 => ParameterValue::I64(LittleEndian::read_i64(&data[field::VALUE])),
            TypeId::F32 => ParameterValue::F32(LittleEndian::read_f32(&data[field::VALUE])),
        }
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> Packet<T> {
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
            ParameterValue::I8(inner) => {
                data[field::VALUE.start] = inner as _;
            }
            ParameterValue::U32(inner) => LittleEndian::write_u32(&mut data[field::VALUE], inner),
            ParameterValue::I32(inner) => LittleEndian::write_i32(&mut data[field::VALUE], inner),
            ParameterValue::U64(inner) => LittleEndian::write_u64(&mut data[field::VALUE], inner),
            ParameterValue::I64(inner) => LittleEndian::write_i64(&mut data[field::VALUE], inner),
            ParameterValue::F32(inner) => LittleEndian::write_f32(&mut data[field::VALUE], inner),
        }
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
    use approx::*;
    use pretty_assertions::assert_eq;

    static NONE_PARAM_BYTES: [u8; 17] = [
        0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];

    static NOTIF_PARAM_BYTES: [u8; 17] = [
        0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x01,
    ];

    static BOOL_PARAM_BYTES: [u8; 18] = [
        0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x02, 0x01,
    ];

    static U8_PARAM_BYTES: [u8; 18] = [
        0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x03, 0xBA,
    ];

    static I8_PARAM_BYTES: [u8; 18] = [
        0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x04, 0x0A,
    ];

    static U32_PARAM_BYTES: [u8; 21] = [
        0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x05, 0x00, 0xFF, 0x00, 0xFF,
    ];

    static I32_PARAM_BYTES: [u8; 21] = [
        0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x06, 0x2E, 0xFB, 0xFF, 0xFF,
    ];

    static U64_PARAM_BYTES: [u8; 25] = [
        0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x07, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0x00, 0x00, 0x00,
    ];

    static I64_PARAM_BYTES: [u8; 25] = [
        0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x08, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0x00, 0x00, 0x00,
    ];

    static F32_PARAM_BYTES: [u8; 21] = [
        0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x09, 0xB6, 0xF3, 0x9D, 0xBF,
    ];

    #[test]
    fn header_len() {
        assert_eq!(Packet::<&[u8]>::header_len(), 17);
        assert_eq!(Packet::<&[u8]>::buffer_len(22), 17 + 22);
    }

    #[test]
    fn construct_none() {
        let mut bytes = [0xFF; 17];
        let mut f = Packet::new_unchecked(&mut bytes);
        assert_eq!(f.check_len(), Ok(()));
        f.set_local_time_ms(255);
        f.set_id(0x0A_u32.into());
        f.set_flags(0_u32.into());
        f.set_value(ParameterValue::None);
        assert_eq!(&f.into_inner()[..], &NONE_PARAM_BYTES[..]);
    }

    #[test]
    fn deconstruct_none() {
        let f = Packet::new_checked(&NONE_PARAM_BYTES[..]).unwrap();
        assert_eq!(f.local_time_ms(), 255);
        assert_eq!(f.id(), 0x0A_u32.into());
        assert_eq!(f.flags(), 0_u32.into());
        assert_eq!(f.value(), ParameterValue::None);
    }

    #[test]
    fn construct_notif() {
        let mut bytes = [0xFF; 17];
        let mut f = Packet::new_unchecked(&mut bytes);
        assert_eq!(f.check_len(), Ok(()));
        f.set_local_time_ms(255);
        f.set_id(0x0A_u32.into());
        f.set_flags(0_u32.into());
        f.set_value(ParameterValue::Notification);
        assert_eq!(&f.into_inner()[..], &NOTIF_PARAM_BYTES[..]);
    }

    #[test]
    fn deconstruct_notif() {
        let f = Packet::new_checked(&NOTIF_PARAM_BYTES[..]).unwrap();
        assert_eq!(f.local_time_ms(), 255);
        assert_eq!(f.id(), 0x0A_u32.into());
        assert_eq!(f.flags(), 0_u32.into());
        assert_eq!(f.value(), ParameterValue::Notification);
    }

    #[test]
    fn construct_bool() {
        let mut bytes = [0xFF; 18];
        let mut f = Packet::new_unchecked(&mut bytes);
        assert_eq!(f.check_len(), Ok(()));
        f.set_local_time_ms(255);
        f.set_id(0x0A_u32.into());
        f.set_flags(0_u32.into());
        f.set_value(ParameterValue::Bool(true));
        assert_eq!(&f.into_inner()[..], &BOOL_PARAM_BYTES[..]);
    }

    #[test]
    fn deconstruct_bool() {
        let f = Packet::new_checked(&BOOL_PARAM_BYTES[..]).unwrap();
        assert_eq!(f.local_time_ms(), 255);
        assert_eq!(f.id(), 0x0A_u32.into());
        assert_eq!(f.flags(), 0_u32.into());
        assert_eq!(f.value(), ParameterValue::Bool(true));
    }

    #[test]
    fn construct_u8() {
        let mut bytes = [0xFF; 18];
        let mut f = Packet::new_unchecked(&mut bytes);
        assert_eq!(f.check_len(), Ok(()));
        f.set_local_time_ms(255);
        f.set_id(0x0A_u32.into());
        f.set_flags(0_u32.into());
        f.set_value(ParameterValue::U8(0xBA));
        assert_eq!(&f.into_inner()[..], &U8_PARAM_BYTES[..]);
    }

    #[test]
    fn deconstruct_u8() {
        let f = Packet::new_checked(&U8_PARAM_BYTES[..]).unwrap();
        assert_eq!(f.local_time_ms(), 255);
        assert_eq!(f.id(), 0x0A_u32.into());
        assert_eq!(f.flags(), 0_u32.into());
        assert_eq!(f.value(), ParameterValue::U8(0xBA));
    }

    #[test]
    fn construct_i8() {
        let mut bytes = [0xFF; 18];
        let mut f = Packet::new_unchecked(&mut bytes);
        assert_eq!(f.check_len(), Ok(()));
        f.set_local_time_ms(255);
        f.set_id(0x0A_u32.into());
        f.set_flags(0_u32.into());
        f.set_value(ParameterValue::I8(0x0A));
        assert_eq!(&f.into_inner()[..], &I8_PARAM_BYTES[..]);
    }

    #[test]
    fn deconstruct_i8() {
        let f = Packet::new_checked(&I8_PARAM_BYTES[..]).unwrap();
        assert_eq!(f.local_time_ms(), 255);
        assert_eq!(f.id(), 0x0A_u32.into());
        assert_eq!(f.flags(), 0_u32.into());
        assert_eq!(f.value(), ParameterValue::I8(0x0A));
    }

    #[test]
    fn construct_u32() {
        let mut bytes = [0xFF; 21];
        let mut f = Packet::new_unchecked(&mut bytes);
        assert_eq!(f.check_len(), Ok(()));
        f.set_local_time_ms(255);
        f.set_id(0x0A_u32.into());
        f.set_flags(0_u32.into());
        f.set_value(ParameterValue::U32(0xFF_00_FF_00));
        assert_eq!(&f.into_inner()[..], &U32_PARAM_BYTES[..]);
    }

    #[test]
    fn deconstruct_u32() {
        let f = Packet::new_checked(&U32_PARAM_BYTES[..]).unwrap();
        assert_eq!(f.local_time_ms(), 255);
        assert_eq!(f.id(), 0x0A_u32.into());
        assert_eq!(f.flags(), 0_u32.into());
        assert_eq!(f.value(), ParameterValue::U32(0xFF_00_FF_00));
    }

    #[test]
    fn construct_i32() {
        let mut bytes = [0xFF; 21];
        let mut f = Packet::new_unchecked(&mut bytes);
        assert_eq!(f.check_len(), Ok(()));
        f.set_local_time_ms(255);
        f.set_id(0x0A_u32.into());
        f.set_flags(0_u32.into());
        f.set_value(ParameterValue::I32(-1234));
        assert_eq!(&f.into_inner()[..], &I32_PARAM_BYTES[..]);
    }

    #[test]
    fn deconstruct_i32() {
        let f = Packet::new_checked(&I32_PARAM_BYTES[..]).unwrap();
        assert_eq!(f.local_time_ms(), 255);
        assert_eq!(f.id(), 0x0A_u32.into());
        assert_eq!(f.flags(), 0_u32.into());
        assert_eq!(f.value(), ParameterValue::I32(-1234));
    }

    #[test]
    fn construct_u64() {
        let mut bytes = [0xFF; 25];
        let mut f = Packet::new_unchecked(&mut bytes);
        assert_eq!(f.check_len(), Ok(()));
        f.set_local_time_ms(255);
        f.set_id(0x0A_u32.into());
        f.set_flags(0_u32.into());
        f.set_value(ParameterValue::U64(0xFF_00_FF_00));
        assert_eq!(&f.into_inner()[..], &U64_PARAM_BYTES[..]);
    }

    #[test]
    fn deconstruct_u64() {
        let f = Packet::new_checked(&U64_PARAM_BYTES[..]).unwrap();
        assert_eq!(f.local_time_ms(), 255);
        assert_eq!(f.id(), 0x0A_u32.into());
        assert_eq!(f.flags(), 0_u32.into());
        assert_eq!(f.value(), ParameterValue::U64(0xFF_00_FF_00));
    }

    #[test]
    fn construct_i64() {
        let mut bytes = [0xFF; 25];
        let mut f = Packet::new_unchecked(&mut bytes);
        assert_eq!(f.check_len(), Ok(()));
        f.set_local_time_ms(255);
        f.set_id(0x0A_u32.into());
        f.set_flags(0_u32.into());
        f.set_value(ParameterValue::I64(0xFF_00_FF_00));
        assert_eq!(&f.into_inner()[..], &I64_PARAM_BYTES[..]);
    }

    #[test]
    fn deconstruct_i64() {
        let f = Packet::new_checked(&I64_PARAM_BYTES[..]).unwrap();
        assert_eq!(f.local_time_ms(), 255);
        assert_eq!(f.id(), 0x0A_u32.into());
        assert_eq!(f.flags(), 0_u32.into());
        assert_eq!(f.value(), ParameterValue::I64(0xFF_00_FF_00));
    }

    #[test]
    fn construct_f32() {
        let mut bytes = [0xFF; 21];
        let mut f = Packet::new_unchecked(&mut bytes);
        assert_eq!(f.check_len(), Ok(()));
        f.set_local_time_ms(255);
        f.set_id(0x0A_u32.into());
        f.set_flags(0_u32.into());
        f.set_value(ParameterValue::F32(-1.234));
        assert_eq!(&f.into_inner()[..], &F32_PARAM_BYTES[..]);
    }

    #[test]
    fn deconstruct_f32() {
        let f = Packet::new_checked(&F32_PARAM_BYTES[..]).unwrap();
        assert_eq!(f.local_time_ms(), 255);
        assert_eq!(f.id(), 0x0A_u32.into());
        assert_eq!(f.flags(), 0_u32.into());
        match f.value() {
            ParameterValue::F32(val) => assert_relative_eq!(val, -1.234),
            _ => panic!("Unexpected value"),
        }
    }
}
