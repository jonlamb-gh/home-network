use core::fmt;
use heapless::consts::U64;
use typenum::Unsigned;

pub type MaxParamsPerOp = U64;
pub const MAX_PARAMS_PER_OP: usize = MaxParamsPerOp::USIZE;

pub const PREAMBLE_WORD: u32 = 0xFF_EF_CD_AB;

pub const NODE_ID_ANONYMOUS: NodeId = 0;

pub type NodeId = u32;
pub type Flags = u32;
pub type Version = u8;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Op {
    /// Request payload: None
    /// Response payload: ParameterListPacket
    ListAll = 0,

    /// Request payload: ParameterIdListPacket
    /// Response payload: ParameterListPacket
    Get = 1,

    /// Request payload: ParameterListPacket
    /// Response payload: ParameterListPacket
    Set = 2,
}

impl Default for Op {
    fn default() -> Self {
        Op::ListAll
    }
}

impl Op {
    pub(crate) fn as_u8(&self) -> u8 {
        *self as u8
    }
}

impl From<u8> for Op {
    fn from(v: u8) -> Self {
        match v {
            0 => Op::ListAll,
            1 => Op::Get,
            2 => Op::Set,
            _ => Op::ListAll,
        }
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", *self)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum PayloadType {
    None = 0,
    ParameterIdListPacket = 1,
    ParameterListPacket = 2,
}

impl Default for PayloadType {
    fn default() -> Self {
        PayloadType::None
    }
}

impl PayloadType {
    pub(crate) fn as_u8(&self) -> u8 {
        *self as u8
    }
}

impl From<u8> for PayloadType {
    fn from(v: u8) -> Self {
        match v {
            0 => PayloadType::None,
            1 => PayloadType::ParameterIdListPacket,
            2 => PayloadType::ParameterListPacket,
            _ => PayloadType::None,
        }
    }
}

impl fmt::Display for PayloadType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", *self)
    }
}
