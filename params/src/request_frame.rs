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

    pub const OP: usize = 0;
    pub const COUNT: usize = 1;
    pub const IDS: Rest = 2..;
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
