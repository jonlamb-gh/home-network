#![no_std]

pub use crate::error::Error;
pub use crate::flags::Flags as ParameterFlags;
pub use crate::getset::{MaxParamsPerOp, Op as GetSetOp, MAX_PARAMS_PER_OP};
pub use crate::id::Id as ParameterId;
pub use crate::parameter::Parameter;
pub use crate::parameter_frame::Frame as ParameterFrame;
pub use crate::request::Request;
pub use crate::request_frame::Frame as RequestFrame;
pub use crate::response::Response;
pub use crate::response_frame::Frame as ResponseFrame;
pub use crate::value::Value as ParameterValue;

mod error;
mod flags;
mod getset;
mod id;
mod parameter;
mod parameter_frame;
mod request;
mod request_frame;
mod response;
mod response_frame;
mod value;
mod wire;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
