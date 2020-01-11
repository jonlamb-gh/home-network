use bitfield::bitfield;

bitfield! {
    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
    pub struct Flags(u32);
    u32;
}

impl From<u32> for Flags {
    fn from(f: u32) -> Self {
        Flags(f)
    }
}
