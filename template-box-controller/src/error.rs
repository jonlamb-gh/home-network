#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Error {
    ParamsError(params::Error),
    SmoltcpError(smoltcp::Error),
    Capacity,
    Duplicate,
    PermissionDenied,
    NotFound,
}

impl From<params::Error> for Error {
    fn from(e: params::Error) -> Self {
        Error::ParamsError(e)
    }
}

impl From<smoltcp::Error> for Error {
    fn from(e: smoltcp::Error) -> Self {
        Error::SmoltcpError(e)
    }
}
