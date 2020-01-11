#![no_std]

pub use crate::error::Error;
pub use crate::flags::Flags;
pub use crate::frame::Frame as ParameterFrame;
pub use crate::id::Id as ParameterId;
pub use crate::parameter::Parameter;
pub use crate::value::Value as ParameterValue;

// TODO - GetSet is a frame with id, payload size, etc

mod error;
mod flags;
mod frame;
mod id;
mod parameter;
mod value;
mod wire;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
