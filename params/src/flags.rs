use bitfield::bitfield;
use static_assertions::assert_eq_size;

assert_eq_size!(u32, Flags);

bitfield! {
    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
    pub struct Flags(u32);
    u32;
    // TODO publish periodic (some bits for range 0-8 Hz?)
    // udp_periodic_tx
    pub read_only, set_read_only : 0;
}

impl Flags {
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
