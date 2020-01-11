// TODO - NonZero?
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Id(pub u32);

impl From<u32> for Id {
    fn from(id: u32) -> Self {
        Id(id)
    }
}
