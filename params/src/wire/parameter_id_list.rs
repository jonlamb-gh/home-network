use crate::{Error, ParameterId, MAX_PARAMS_PER_OP};
use byteorder::{ByteOrder, LittleEndian};
use core::mem;
use static_assertions::assert_eq_size;

assert_eq_size!(u32, ParameterId);

#[derive(Debug, Clone)]
pub struct Packet<T: AsRef<[u8]>> {
    buffer: T,
}

mod field {
    use crate::wire::field::*;

    pub const COUNT: usize = 0;
    pub const IDS: Rest = 1..;
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
    pub fn count(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[field::COUNT]
    }

    #[inline]
    pub fn id_at(&self, index: usize) -> Result<ParameterId, Error> {
        if index >= usize::from(self.count()) {
            return Err(Error::WireIndexOutOfBounds);
        }
        let data = self.buffer.as_ref();
        let offset = index * mem::size_of::<ParameterId>();
        Ok(ParameterId::from(LittleEndian::read_u32(
            &data[field::IDS.start + offset..],
        )))
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> Packet<T> {
    #[inline]
    pub fn set_count(&mut self, value: u8) {
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

impl<T: AsRef<[u8]>> AsRef<[u8]> for Packet<T> {
    fn as_ref(&self) -> &[u8] {
        self.buffer.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ParameterId;
    use core::convert::TryInto;

    static BYTES: [u8; 29] = [
        0x07, 0x0A, 0x00, 0x00, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x0D, 0x00,
        0x00, 0x00, 0x0E, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00,
    ];

    static PARAM_IDS: [ParameterId; 7] = [
        ParameterId::new(0x0A),
        ParameterId::new(0x0B),
        ParameterId::new(0x0C),
        ParameterId::new(0x0D),
        ParameterId::new(0x0E),
        ParameterId::new(0x0F),
        ParameterId::new(0x10),
    ];

    #[test]
    fn construct() {
        let mut bytes = [0xFF; 29];
        let mut p = Packet::new_unchecked(&mut bytes[..]);
        assert_eq!(p.check_len(), Ok(()));
        p.set_count(PARAM_IDS.len().try_into().unwrap());
        for (index, id) in PARAM_IDS.iter().enumerate() {
            assert_eq!(p.set_id_at(index, *id), Ok(()));
        }
        assert_eq!(&p.into_inner()[..], &BYTES[..]);
    }

    #[test]
    fn deconstruct() {
        let p = Packet::new_checked(&BYTES[..]).unwrap();
        assert_eq!(p.count(), PARAM_IDS.len().try_into().unwrap());
        for index in 0..PARAM_IDS.len() {
            assert_eq!(p.id_at(index), Ok(PARAM_IDS[index]));
        }
    }
}
