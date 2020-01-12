use crate::{Error, GetSetOp, Parameter, ParameterFrame, MAX_PARAMS_PER_OP};
use byteorder::{ByteOrder, LittleEndian};

#[derive(Debug, Clone)]
pub struct Frame<T: AsRef<[u8]>> {
    buffer: T,
}

mod field {
    use crate::wire::field::*;

    pub const PREAMBLE: Field = 0..4;
    pub const OP: usize = 4;
    pub const COUNT: usize = 5;
    pub const PARAMS: Rest = 6..;
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
    pub fn num_parameters(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[field::COUNT]
    }

    #[inline]
    pub fn param_at(&self, index: usize) -> Result<Parameter, Error> {
        if index >= usize::from(self.num_parameters()) {
            return Err(Error::WireIndexOutOfBounds);
        }
        let data = self.buffer.as_ref();
        let mut offset = field::PARAMS.start;
        for _ in 0..index {
            let f = ParameterFrame::new_checked(&data[offset..])?;
            offset += ParameterFrame::<&[u8]>::buffer_len(f.value_type_id().wire_size());
        }

        // Cursor now at index
        Parameter::parse(&ParameterFrame::new_unchecked(&data[offset..]))
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
    pub fn set_num_parameters(&mut self, value: u8) {
        let data = self.buffer.as_mut();
        data[field::COUNT] = value;
    }

    #[inline]
    pub fn set_param_at(&mut self, index: usize, value: Parameter) -> Result<(), Error> {
        if index >= MAX_PARAMS_PER_OP {
            return Err(Error::WireIndexOutOfBounds);
        }
        let data = self.buffer.as_mut();
        let mut offset = field::PARAMS.start;
        for _ in 0..index {
            let f = ParameterFrame::new_checked(&data[offset..])?;
            offset += ParameterFrame::<&[u8]>::buffer_len(f.value_type_id().wire_size());
        }

        // Cursor now at index
        let mut f = ParameterFrame::new_checked(&mut data[offset..])?;
        value.emit(&mut f);
        Ok(())
    }
}

impl<T: AsRef<[u8]>> AsRef<[u8]> for Frame<T> {
    fn as_ref(&self) -> &[u8] {
        self.buffer.as_ref()
    }
}
