// TODO - refactor this impl
// reduce the types, add a str/string type?

use crate::Error;
use core::fmt;
use core::str;

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum Value {
    None,
    // TODO - not sure unidirectional Notification fits here
    Notification,
    Bool(bool),
    U8(u8),
    I8(i8),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    F32(f32),
}

impl Default for Value {
    fn default() -> Self {
        Value::None
    }
}

impl Value {
    pub fn type_id(&self) -> TypeId {
        TypeId::from(*self)
    }

    pub fn as_bool(&self) -> bool {
        match *self {
            Value::Bool(v) => v,
            _ => panic!("Value type mismatch"),
        }
    }

    pub fn as_u8(&self) -> u8 {
        match *self {
            Value::U8(v) => v,
            _ => panic!("Value type mismatch"),
        }
    }

    pub fn as_i8(&self) -> i8 {
        match *self {
            Value::I8(v) => v,
            _ => panic!("Value type mismatch"),
        }
    }

    pub fn as_u32(&self) -> u32 {
        match *self {
            Value::U32(v) => v,
            _ => panic!("Value type mismatch"),
        }
    }

    pub fn as_i32(&self) -> i32 {
        match *self {
            Value::I32(v) => v,
            _ => panic!("Value type mismatch"),
        }
    }

    pub fn as_u64(&self) -> u64 {
        match *self {
            Value::U64(v) => v,
            _ => panic!("Value type mismatch"),
        }
    }

    pub fn as_i64(&self) -> i64 {
        match *self {
            Value::I64(v) => v,
            _ => panic!("Value type mismatch"),
        }
    }

    pub fn as_f32(&self) -> f32 {
        match *self {
            Value::F32(v) => v,
            _ => panic!("Value type mismatch"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::None => write!(f, "None"),
            Value::Notification => write!(f, "Notification"),
            Value::Bool(v) => write!(f, "Bool({})", v),
            Value::U8(v) => write!(f, "U8({})", v),
            Value::I8(v) => write!(f, "I8({})", v),
            Value::U32(v) => write!(f, "U32({})", v),
            Value::I32(v) => write!(f, "I32({})", v),
            Value::U64(v) => write!(f, "U64({})", v),
            Value::I64(v) => write!(f, "I64({})", v),
            Value::F32(v) => write!(f, "F32({})", v),
        }
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum TypeId {
    None = 0,
    Notification = 1,
    Bool = 2,
    U8 = 3,
    I8 = 4,
    U32 = 5,
    I32 = 6,
    U64 = 7,
    I64 = 8,
    F32 = 9,
}

impl str::FromStr for TypeId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "None" || s == "none" {
            Ok(TypeId::None)
        } else if s == "Notification" || s == "notif" {
            Ok(TypeId::Notification)
        } else if s == "Bool" || s == "bool" {
            Ok(TypeId::Bool)
        } else if s == "U8" || s == "u8" {
            Ok(TypeId::U8)
        } else if s == "I8" || s == "i8" {
            Ok(TypeId::I8)
        } else if s == "U32" || s == "u32" {
            Ok(TypeId::U32)
        } else if s == "I32" || s == "i32" {
            Ok(TypeId::I32)
        } else if s == "U64" || s == "u64" {
            Ok(TypeId::U64)
        } else if s == "I64" || s == "i64" {
            Ok(TypeId::I64)
        } else if s == "F32" || s == "f32" {
            Ok(TypeId::F32)
        } else {
            Err(Error::ParseValue)
        }
    }
}

impl TypeId {
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    // Size of the value field on the ire
    pub fn wire_size(&self) -> usize {
        match *self {
            TypeId::None => 0,
            TypeId::Notification => 0,
            TypeId::Bool => 1,
            TypeId::U8 => 1,
            TypeId::I8 => 1,
            TypeId::U32 => 4,
            TypeId::I32 => 4,
            TypeId::U64 => 8,
            TypeId::I64 => 8,
            TypeId::F32 => 4,
        }
    }
}

impl From<u8> for TypeId {
    fn from(v: u8) -> TypeId {
        match v {
            0 => TypeId::None,
            1 => TypeId::Notification,
            2 => TypeId::Bool,
            3 => TypeId::U8,
            4 => TypeId::I8,
            5 => TypeId::U32,
            6 => TypeId::I32,
            7 => TypeId::U64,
            8 => TypeId::I64,
            9 => TypeId::F32,
            _ => TypeId::None,
        }
    }
}

impl From<Value> for TypeId {
    fn from(v: Value) -> TypeId {
        match v {
            Value::None => TypeId::None,
            Value::Notification => TypeId::Notification,
            Value::Bool(_) => TypeId::Bool,
            Value::U8(_) => TypeId::U8,
            Value::I8(_) => TypeId::I8,
            Value::U32(_) => TypeId::U32,
            Value::I32(_) => TypeId::I32,
            Value::U64(_) => TypeId::U64,
            Value::I64(_) => TypeId::I64,
            Value::F32(_) => TypeId::F32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::*;
    use core::mem;
    use pretty_assertions::assert_eq;

    #[test]
    fn wire_size() {
        assert_eq!(TypeId::from(Value::None).wire_size(), mem::size_of::<()>());
        assert_eq!(
            TypeId::from(Value::Notification).wire_size(),
            mem::size_of::<()>()
        );
        assert_eq!(
            TypeId::from(Value::Bool(true)).wire_size(),
            mem::size_of::<u8>()
        );
        assert_eq!(
            TypeId::from(Value::U8(123)).wire_size(),
            mem::size_of::<u8>()
        );
        assert_eq!(
            TypeId::from(Value::I8(-123)).wire_size(),
            mem::size_of::<i8>()
        );
        assert_eq!(
            TypeId::from(Value::U32(123)).wire_size(),
            mem::size_of::<u32>()
        );
        assert_eq!(
            TypeId::from(Value::I32(-123)).wire_size(),
            mem::size_of::<i32>()
        );
        assert_eq!(
            TypeId::from(Value::U64(123)).wire_size(),
            mem::size_of::<u64>()
        );
        assert_eq!(
            TypeId::from(Value::I64(-123)).wire_size(),
            mem::size_of::<i64>()
        );
        assert_eq!(
            TypeId::from(Value::F32(-1.234)).wire_size(),
            mem::size_of::<f32>()
        );
    }

    #[test]
    fn inner_types() {
        let val = Value::Bool(true);
        assert_eq!(true, val.as_bool());

        let val = Value::U8(123);
        assert_eq!(123, val.as_u8());

        let val = Value::I8(-123);
        assert_eq!(-123, val.as_i8());

        let val = Value::U32(12345);
        assert_eq!(12345, val.as_u32());

        let val = Value::I32(-123);
        assert_eq!(-123, val.as_i32());

        let val = Value::U64(12345);
        assert_eq!(12345, val.as_u64());

        let val = Value::I64(-123);
        assert_eq!(-123, val.as_i64());

        let val = Value::F32(-1.23);
        assert_relative_eq!(-1.23, val.as_f32());
    }

    #[test]
    fn type_ids() {
        let val = Value::None;
        assert_eq!(val.type_id(), TypeId::None);
        assert_eq!(TypeId::from(val.type_id().as_u8()), TypeId::None);

        let val = Value::Notification;
        assert_eq!(val.type_id(), TypeId::Notification);
        assert_eq!(TypeId::from(val.type_id().as_u8()), TypeId::Notification);

        let val = Value::Bool(true);
        assert_eq!(val.type_id(), TypeId::Bool);
        assert_eq!(TypeId::from(val.type_id().as_u8()), TypeId::Bool);

        let val = Value::U8(123);
        assert_eq!(val.type_id(), TypeId::U8);
        assert_eq!(TypeId::from(val.type_id().as_u8()), TypeId::U8);

        let val = Value::I8(-123);
        assert_eq!(val.type_id(), TypeId::I8);
        assert_eq!(TypeId::from(val.type_id().as_u8()), TypeId::I8);

        let val = Value::U32(12345);
        assert_eq!(val.type_id(), TypeId::U32);
        assert_eq!(TypeId::from(val.type_id().as_u8()), TypeId::U32);

        let val = Value::I32(-123);
        assert_eq!(val.type_id(), TypeId::I32);
        assert_eq!(TypeId::from(val.type_id().as_u8()), TypeId::I32);

        let val = Value::U64(12345);
        assert_eq!(val.type_id(), TypeId::U64);
        assert_eq!(TypeId::from(val.type_id().as_u8()), TypeId::U64);

        let val = Value::I64(-123);
        assert_eq!(val.type_id(), TypeId::I64);
        assert_eq!(TypeId::from(val.type_id().as_u8()), TypeId::I64);

        let val = Value::F32(-1.23);
        assert_eq!(val.type_id(), TypeId::F32);
        assert_eq!(TypeId::from(val.type_id().as_u8()), TypeId::F32);
    }
}
