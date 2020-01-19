use core::fmt;
use static_assertions::assert_eq_size;

assert_eq_size!(u32, Id);

// TODO - NonZero?
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Id(pub u32);

impl Id {
    pub const fn new(id: u32) -> Self {
        Id(id)
    }

    pub fn wire_size(&self) -> usize {
        4
    }
}

impl From<u32> for Id {
    fn from(id: u32) -> Self {
        Id(id)
    }
}

impl Into<u32> for Id {
    fn into(self) -> u32 {
        self.0
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
