use crate::{
    Error, GetSetFlags, GetSetNodeId, GetSetOp, GetSetPayloadType, GetSetVersion, PREAMBLE_WORD,
};
use byteorder::{ByteOrder, LittleEndian};
use core::fmt;

#[derive(Debug, Clone)]
pub struct Frame<T: AsRef<[u8]>> {
    buffer: T,
}

mod field {
    use crate::wire::field::*;

    pub const PREAMBLE: Field = 0..4;
    pub const NODE_ID: Field = 4..8;
    pub const FLAGS: Field = 8..12;
    pub const VERSION: usize = 12;
    pub const OP: usize = 13;
    pub const PAYLOAD_TYPE: usize = 14;
    pub const PAYLOAD_SIZE: Field = 15..17;
    pub const PAYLOAD: Rest = 17..;
}

impl<T: AsRef<[u8]>> Frame<T> {
    pub fn new_unchecked(buffer: T) -> Frame<T> {
        Frame { buffer }
    }

    pub fn new_checked(buffer: T) -> Result<Frame<T>, Error> {
        let packet = Self::new_unchecked(buffer);
        packet.check_len()?;
        packet.check_preamble()?;
        Ok(packet)
    }

    pub fn check_len(&self) -> Result<(), Error> {
        let len = self.buffer.as_ref().len();
        if len < field::PAYLOAD.start {
            Err(Error::WireTruncated)
        } else {
            Ok(())
        }
    }

    pub fn check_preamble(&self) -> Result<(), Error> {
        if self.preamble() != PREAMBLE_WORD {
            Err(Error::WirePreamble)
        } else {
            Ok(())
        }
    }

    pub fn into_inner(self) -> T {
        self.buffer
    }

    pub fn header_len() -> usize {
        field::PAYLOAD.start
    }

    pub fn buffer_len(payload_len: usize) -> usize {
        field::PAYLOAD.start + payload_len
    }

    #[inline]
    pub fn preamble(&self) -> u32 {
        let data = self.buffer.as_ref();
        LittleEndian::read_u32(&data[field::PREAMBLE])
    }

    #[inline]
    pub fn node_id(&self) -> GetSetNodeId {
        let data = self.buffer.as_ref();
        LittleEndian::read_u32(&data[field::NODE_ID])
    }

    #[inline]
    pub fn flags(&self) -> GetSetFlags {
        let data = self.buffer.as_ref();
        LittleEndian::read_u32(&data[field::FLAGS])
    }

    #[inline]
    pub fn version(&self) -> GetSetVersion {
        let data = self.buffer.as_ref();
        data[field::VERSION]
    }

    #[inline]
    pub fn op(&self) -> GetSetOp {
        let data = self.buffer.as_ref();
        GetSetOp::from(data[field::OP])
    }

    #[inline]
    pub fn payload_type(&self) -> GetSetPayloadType {
        let data = self.buffer.as_ref();
        GetSetPayloadType::from(data[field::PAYLOAD_TYPE])
    }

    #[inline]
    pub fn payload_size(&self) -> u16 {
        let data = self.buffer.as_ref();
        LittleEndian::read_u16(&data[field::PAYLOAD_SIZE])
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> Frame<&'a T> {
    #[inline]
    pub fn payload(&self) -> &'a [u8] {
        let data = self.buffer.as_ref();
        &data[field::PAYLOAD]
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> Frame<T> {
    #[inline]
    pub fn set_preamble(&mut self, value: u32) {
        let data = self.buffer.as_mut();
        LittleEndian::write_u32(&mut data[field::PREAMBLE], value);
    }

    #[inline]
    pub fn set_node_id(&mut self, value: GetSetNodeId) {
        let data = self.buffer.as_mut();
        LittleEndian::write_u32(&mut data[field::NODE_ID], value);
    }

    #[inline]
    pub fn set_flags(&mut self, value: GetSetFlags) {
        let data = self.buffer.as_mut();
        LittleEndian::write_u32(&mut data[field::FLAGS], value);
    }

    #[inline]
    pub fn set_version(&mut self, value: GetSetVersion) {
        let data = self.buffer.as_mut();
        data[field::VERSION] = value;
    }

    #[inline]
    pub fn set_op(&mut self, value: GetSetOp) {
        let data = self.buffer.as_mut();
        data[field::OP] = value.as_u8();
    }

    #[inline]
    pub fn set_payload_type(&mut self, value: GetSetPayloadType) {
        let data = self.buffer.as_mut();
        data[field::PAYLOAD_TYPE] = value.as_u8();
    }

    #[inline]
    pub fn set_payload_size(&mut self, value: u16) {
        let data = self.buffer.as_mut();
        LittleEndian::write_u16(&mut data[field::PAYLOAD_SIZE], value);
    }

    #[inline]
    pub fn payload_mut(&mut self) -> &mut [u8] {
        let data = self.buffer.as_mut();
        &mut data[field::PAYLOAD]
    }
}

impl<T: AsRef<[u8]>> AsRef<[u8]> for Frame<T> {
    fn as_ref(&self) -> &[u8] {
        self.buffer.as_ref()
    }
}

impl<T: AsRef<[u8]>> fmt::Display for Frame<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "GetSetFrame {{ pa: 0x{:X} nid: 0x{:X} f: 0x{:X} v: {} op: {} pt: {} ps: {}}}",
            self.preamble(),
            self.node_id(),
            self.flags(),
            self.version(),
            self.op(),
            self.payload_type(),
            self.payload_size(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::TypeId;
    use crate::{ParameterPacket, MAX_PARAMS_PER_OP, PREAMBLE_WORD};
    use core::convert::TryInto;
    use pretty_assertions::assert_eq;

    static FRAME_BYTES: [u8; 46] = [
        0xAB, 0xCD, 0xEF, 0xFF, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x01,
        0x1D, 0x00, 0x07, 0x0A, 0x00, 0x00, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x00, 0x00,
        0x0D, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00,
        0x00,
    ];

    static PAYLOAD_BYTES: [u8; 29] = [
        0x07, 0x0A, 0x00, 0x00, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x0D, 0x00,
        0x00, 0x00, 0x0E, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn header_len() {
        assert_eq!(Frame::<&[u8]>::header_len(), 17);
        assert_eq!(Frame::<&[u8]>::buffer_len(22), 17 + 22);
    }

    #[test]
    fn max_fits_in_udp_datagram() {
        let value_size = TypeId::U32.wire_size();
        let f_size = ParameterPacket::<&[u8]>::buffer_len(value_size);
        assert!(MAX_PARAMS_PER_OP * f_size <= 1500);
    }

    #[test]
    fn construct() {
        let mut bytes = [0xFF; 46];
        let mut f = Frame::new_unchecked(&mut bytes[..]);
        assert_eq!(f.check_len(), Ok(()));
        f.set_preamble(PREAMBLE_WORD);
        f.set_node_id(0x01);
        f.set_flags(0);
        f.set_version(1);
        f.set_op(GetSetOp::Get);
        f.set_payload_type(GetSetPayloadType::ParameterIdListPacket);
        f.set_payload_size(PAYLOAD_BYTES.len().try_into().unwrap());
        f.payload_mut().copy_from_slice(&PAYLOAD_BYTES[..]);
        assert_eq!(f.check_preamble(), Ok(()));
        assert_eq!(&f.into_inner()[..], &FRAME_BYTES[..]);
    }

    #[test]
    fn deconstruct() {
        let f = Frame::new_checked(&FRAME_BYTES[..]).unwrap();
        assert_eq!(f.preamble(), PREAMBLE_WORD);
        assert_eq!(f.node_id(), 0x01);
        assert_eq!(f.flags(), 0);
        assert_eq!(f.version(), 1);
        assert_eq!(f.op(), GetSetOp::Get);
        assert_eq!(f.payload_type(), GetSetPayloadType::ParameterIdListPacket);
        assert_eq!(f.payload_size(), PAYLOAD_BYTES.len().try_into().unwrap());
        assert_eq!(f.payload(), &PAYLOAD_BYTES[..]);
    }
}
