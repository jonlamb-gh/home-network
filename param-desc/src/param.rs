#![allow(unused_imports)]

use params::flags::{BCAST, CONST, RO};
use params::{Parameter, ParameterFlags, ParameterId, ParameterValue};

include! {concat!(env!("OUT_DIR"), "/param_gen.rs")}
