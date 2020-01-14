#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Error {
    ParamsError(params::Error),
    Capacity,
    Duplicate,
    PermissionDenied,
}

impl From<params::Error> for Error {
    fn from(e: params::Error) -> Self {
        Error::ParamsError(e)
    }
}
