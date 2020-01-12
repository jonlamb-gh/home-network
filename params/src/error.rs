#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Error {
    WireTruncated,
    WireIndexOutOfBounds,
    Capacity,
}
