use bitfield::bitfield;

bitfield! {
    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
    pub struct Flags(u32);
    u32;
    pub read_only, set_read_only : 0;
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
