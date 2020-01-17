use crate::{
    Error, GetSetFlags, GetSetFrame, GetSetNodeId, GetSetOp, GetSetPayloadType, Parameter,
    ParameterListPacket, PREAMBLE_WORD,
};
use core::fmt;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct RefResponse<P: AsRef<[Parameter]>> {
    node_id: GetSetNodeId,
    flags: GetSetFlags,
    op: GetSetOp,
    params: P,
}

impl<P: AsRef<[Parameter]>> RefResponse<P> {
    pub fn new(node_id: GetSetNodeId, flags: GetSetFlags, op: GetSetOp, params: P) -> Self {
        RefResponse {
            node_id,
            flags,
            op,
            params,
        }
    }

    pub fn op(&self) -> GetSetOp {
        self.op
    }

    pub fn wire_size(&self) -> usize {
        GetSetFrame::<&[u8]>::buffer_len(self.payload_wire_size())
    }

    fn payload_wire_size(&self) -> usize {
        ParameterListPacket::<&[u8]>::buffer_len(
            self.params.as_ref().iter().map(|p| p.wire_size()).sum(),
        )
    }

    // TODO - does this make sense to have?
    // pub fn parse<T: AsRef<[u8]> + ?Sized>(frame: &GetSetFrame<&T>) -> Result<Self, Error> {

    pub fn emit<T: AsRef<[u8]> + AsMut<[u8]>>(
        &self,
        frame: &mut GetSetFrame<T>,
    ) -> Result<(), Error> {
        frame.set_preamble(PREAMBLE_WORD);
        frame.set_node_id(self.node_id);
        frame.set_flags(self.flags);
        frame.set_version(1);
        frame.set_op(self.op);
        frame.set_payload_type(GetSetPayloadType::ParameterListPacket);
        frame.set_payload_size(self.payload_wire_size() as u16);
        let mut p = ParameterListPacket::new_unchecked(frame.payload_mut());
        p.set_count(self.params.as_ref().len() as _);
        for (index, param) in self.params.as_ref().iter().enumerate() {
            p.set_parameter_at(index, *param)?;
        }
        Ok(())
    }
}

impl<P: AsRef<[Parameter]>> fmt::Display for RefResponse<P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "RefResponse {{ op: {} }}", self.op())?;
        for p in self.params.as_ref().iter() {
            writeln!(f, "{}", p)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ParameterFlags, ParameterId, ParameterPacket, ParameterValue};
    use core::mem;

    #[test]
    fn wire_size() {
        let params = [
            Parameter::new_with_value(
                ParameterId::new(0x0A),
                ParameterFlags(0),
                ParameterValue::I32(-1234),
            ),
            Parameter::new_with_value(
                ParameterId::new(0x0B),
                ParameterFlags(0),
                ParameterValue::Bool(true),
            ),
        ];

        let resp = RefResponse::new(0, 0, GetSetOp::Set, &params[..]);
        assert_eq!(resp.op(), GetSetOp::Set);
        assert_eq!(resp.params.len(), 2);
        assert_eq!(
            resp.wire_size(),
            GetSetFrame::<&[u8]>::header_len()
                + ParameterListPacket::<&[u8]>::header_len()
                + ParameterPacket::<&[u8]>::header_len()
                + mem::size_of::<i32>()
                + ParameterPacket::<&[u8]>::header_len()
                + mem::size_of::<u8>()
        );
    }

    #[test]
    fn emit() {
        let p_a = Parameter::new_with_value(
            ParameterId::new(0x0A),
            ParameterFlags(0),
            ParameterValue::I32(-1234),
        );
        let p_b = Parameter::new_with_value(
            ParameterId::new(0x0B),
            ParameterFlags(0),
            ParameterValue::Bool(true),
        );
        let params = [p_a, p_b];

        let resp = RefResponse::new(0, 0, GetSetOp::Set, &params[..]);
        assert_eq!(resp.op(), GetSetOp::Set);

        let mut bytes = [0xFF; 64];
        let mut frame = GetSetFrame::new_unchecked(&mut bytes[..]);
        assert_eq!(resp.emit(&mut frame), Ok(()));
        assert_eq!(frame.check_len(), Ok(()));
        assert_eq!(frame.check_preamble(), Ok(()));
        assert_eq!(frame.op(), GetSetOp::Set);
        assert_eq!(frame.payload_size(), resp.payload_wire_size() as u16);
        let packet = ParameterListPacket::new_checked(frame.payload_mut()).unwrap();
        assert_eq!(packet.check_len(), Ok(()));
        assert_eq!(packet.count(), 2);
        assert_eq!(packet.parameter_at(0), Ok(p_a));
        assert_eq!(packet.parameter_at(1), Ok(p_b));
    }
}
