#![no_std]

pub use crate::error::Error;
pub use crate::flags::Flags as ParameterFlags;
pub use crate::getset::{MaxParamsPerOp, Op as GetSetOp, MAX_PARAMS_PER_OP, PREAMBLE_WORD};
pub use crate::id::Id as ParameterId;
pub use crate::parameter::Parameter;
pub use crate::ref_response::RefResponse;
pub use crate::request::Request;
pub use crate::response::Response;
pub use crate::value::Value as ParameterValue;
pub use crate::wire::getset::Frame as GetSetFrame;
pub use crate::wire::parameter::Packet as ParameterPacket;
pub use crate::wire::parameter_id_list::Packet as ParameterIdListPacket;
pub use crate::wire::parameter_list::Packet as ParameterListPacket;

mod error;
mod flags;
mod getset;
mod id;
mod parameter;
mod ref_response;
mod request;
mod response;
mod value;
mod wire;
