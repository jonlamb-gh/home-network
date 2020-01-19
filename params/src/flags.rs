use bitfield::bitfield;
use core::fmt;
use static_assertions::assert_eq_size;

assert_eq_size!(u32, Flags);

bitfield! {
    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
    pub struct Flags(u32);
    u32;
    pub read_only, set_read_only : 0;
    pub broadcast, set_broadcast: 1;
}

impl Flags {
    pub const fn new() -> Self {
        Flags(0)
    }

    pub const fn new_read_only() -> Self {
        Flags(0x01)
    }

    pub const fn new_broadcast() -> Self {
        Flags(0x02)
    }

    pub const fn new_read_only_broadcast() -> Self {
        Flags(0x01 | 0x02)
    }

    pub fn wire_size(&self) -> usize {
        4
    }
}

impl From<u32> for Flags {
    fn from(f: u32) -> Self {
        Flags(f)
    }
}

impl Into<u32> for Flags {
    fn into(self) -> u32 {
        self.0
    }
}

impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{:X}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn const_read_only() {
        let ro = Flags::new_read_only();
        let mut f = Flags::default();
        assert_eq!(f.0, 0);
        f.set_read_only(true);
        assert_eq!(f, ro);
    }

    #[test]
    fn const_read_only_broadcast() {
        let ro = Flags::new_read_only_broadcast();
        let mut f = Flags::default();
        assert_eq!(f.0, 0);
        f.set_read_only(true);
        f.set_broadcast(true);
        assert_eq!(f, ro);
    }
}
