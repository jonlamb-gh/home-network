use core::fmt;
use heapless::consts::U64;
use typenum::Unsigned;

pub type MaxParamsPerOp = U64;
pub const MAX_PARAMS_PER_OP: usize = MaxParamsPerOp::USIZE;

pub const PREAMBLE_WORD: u32 = 0xFF_EF_CD_AB;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Op {
    ListAll = 0,
    Get = 1,
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
    fn from(v: u8) -> Op {
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
