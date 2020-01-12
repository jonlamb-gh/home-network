use crate::{Error, GetSetOp, ParameterId, MAX_PARAMS_PER_OP};
use byteorder::{ByteOrder, LittleEndian};
use core::mem;
use static_assertions::assert_eq_size;

assert_eq_size!(u32, ParameterId);

#[derive(Debug, Clone)]
pub struct Frame<T: AsRef<[u8]>> {
    buffer: T,
}

mod field {
    use crate::wire::field::*;

    pub const PREAMBLE: Field = 0..4;
    pub const OP: usize = 4;
    pub const COUNT: usize = 5;
    pub const IDS: Rest = 6..;
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
        if len < field::IDS.start {
            Err(Error::WireTruncated)
        } else {
            Ok(())
        }
    }

    pub fn into_inner(self) -> T {
        self.buffer
    }

    pub fn header_len() -> usize {
        field::IDS.start
    }

    pub fn buffer_len(payload_len: usize) -> usize {
        field::IDS.start + payload_len
    }

    #[inline]
    pub fn preamble(&self) -> u32 {
        let data = self.buffer.as_ref();
        LittleEndian::read_u32(&data[field::PREAMBLE])
    }

    #[inline]
    pub fn op(&self) -> GetSetOp {
        let data = self.buffer.as_ref();
        GetSetOp::from(data[field::OP])
    }

    #[inline]
    pub fn num_ids(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[field::COUNT]
    }

    #[inline]
    pub fn id_at(&self, index: usize) -> Result<ParameterId, Error> {
        if index >= usize::from(self.num_ids()) {
            return Err(Error::WireIndexOutOfBounds);
        }
        let data = self.buffer.as_ref();
        let offset = index * mem::size_of::<ParameterId>();
        Ok(ParameterId::from(LittleEndian::read_u32(
            &data[field::IDS.start + offset..],
        )))
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> Frame<T> {
    #[inline]
    pub fn set_preamble(&mut self, value: u32) {
        let data = self.buffer.as_mut();
        LittleEndian::write_u32(&mut data[field::PREAMBLE], value);
    }

    #[inline]
    pub fn set_op(&mut self, value: GetSetOp) {
        let data = self.buffer.as_mut();
        data[field::OP] = value.as_u8();
    }

    #[inline]
    pub fn set_num_ids(&mut self, value: u8) {
        let data = self.buffer.as_mut();
        data[field::COUNT] = value;
    }

    #[inline]
    pub fn set_id_at(&mut self, index: usize, value: ParameterId) -> Result<(), Error> {
        if index >= MAX_PARAMS_PER_OP {
            return Err(Error::WireIndexOutOfBounds);
        }
        let data = self.buffer.as_mut();
        let offset = index * mem::size_of::<ParameterId>();
        LittleEndian::write_u32(&mut data[field::IDS.start + offset..], value.into());
        Ok(())
    }
}

impl<T: AsRef<[u8]>> AsRef<[u8]> for Frame<T> {
    fn as_ref(&self) -> &[u8] {
        self.buffer.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ParameterFlags, Parameter, PREAMBLE_WORD, ParameterValue};
    use core::convert::TryInto;

    static LIST_ALL_REQ_BYTES: [u8; 6] = [
        0xAB, 0xCD, 0xEF, 0xFF,
        0x00,
        0x00,
    ];

    static GET_REQ_BYTES: [u8; 34] = [
        0xAB, 0xCD, 0xEF, 0xFF,
        0x01,
        0x07,
        0x0A, 0x00, 0x00, 0x00,
        0x0B, 0x00, 0x00, 0x00,
        0x0C, 0x00, 0x00, 0x00,
        0x0D, 0x00, 0x00, 0x00,
        0x0E, 0x00, 0x00, 0x00,
        0x0F, 0x00, 0x00, 0x00,
        0x10, 0x00, 0x00, 0x00,
    ];

    static SET_REQ_BYTES: [u8; 34] = [
        0xAB, 0xCD, 0xEF, 0xFF,
        0x02,
        0x07,
        0x0A, 0x00, 0x00, 0x00,
        0x0B, 0x00, 0x00, 0x00,
        0x0C, 0x00, 0x00, 0x00,
        0x0D, 0x00, 0x00, 0x00,
        0x0E, 0x00, 0x00, 0x00,
        0x0F, 0x00, 0x00, 0x00,
        0x10, 0x00, 0x00, 0x00,
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
    fn header_len() {
        assert_eq!(Frame::<&[u8]>::header_len(), 6);
        assert_eq!(Frame::<&[u8]>::buffer_len(22), 6 + 22);
    }

    #[test]
    fn construct_list_all() {
        let mut bytes = [0xFF; 6];
        let mut f = Frame::new_unchecked(&mut bytes);
        assert_eq!(f.check_len(), Ok(()));
        f.set_preamble(PREAMBLE_WORD);
        f.set_op(GetSetOp::ListAll);
        f.set_num_ids(0);
        assert_eq!(&f.into_inner()[..], &LIST_ALL_REQ_BYTES[..]);
    }

    #[test]
    fn deconstruct_list_all() {
        let f = Frame::new_checked(&LIST_ALL_REQ_BYTES[..]).unwrap();
        assert_eq!(f.preamble(), PREAMBLE_WORD);
        assert_eq!(f.op(), GetSetOp::ListAll);
        assert_eq!(f.num_ids(), 0);
    }

    #[test]
    fn construct_get() {
        let mut bytes = [0xFF; 34];
        let mut f = Frame::new_unchecked(&mut bytes[..]);
        assert_eq!(f.check_len(), Ok(()));
        f.set_preamble(PREAMBLE_WORD);
        f.set_op(GetSetOp::Get);
        f.set_num_ids(PARAMS.len().try_into().unwrap());
        for (index, p) in PARAMS.iter().enumerate() {
            assert_eq!(f.set_id_at(index, p.id()), Ok(()));
        }
        assert_eq!(&f.into_inner()[..], &GET_REQ_BYTES[..]);
    }

    #[test]
    fn deconstruct_get() {
        let f = Frame::new_checked(&GET_REQ_BYTES[..]).unwrap();
        assert_eq!(f.preamble(), PREAMBLE_WORD);
        assert_eq!(f.op(), GetSetOp::Get);
        assert_eq!(f.num_ids(), PARAMS.len().try_into().unwrap());
        for index in 0..PARAMS.len() {
            assert_eq!(f.id_at(index), Ok(PARAMS[index].id()));
        }
    }

    #[test]
    fn construct_set() {
        let mut bytes = [0xFF; 34];
        let mut f = Frame::new_unchecked(&mut bytes[..]);
        assert_eq!(f.check_len(), Ok(()));
        f.set_preamble(PREAMBLE_WORD);
        f.set_op(GetSetOp::Set);
        f.set_num_ids(PARAMS.len().try_into().unwrap());
        for (index, p) in PARAMS.iter().enumerate() {
            assert_eq!(f.set_id_at(index, p.id()), Ok(()));
        }
        assert_eq!(&f.into_inner()[..], &SET_REQ_BYTES[..]);
    }
}
