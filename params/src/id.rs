// TODO - NonZero?
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Id(u32);

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
