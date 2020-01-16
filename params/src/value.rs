use core::fmt;

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum Value {
    None,
    // TODO - not sure unidirectional Notification fits here
    Notification,
    Bool(bool),
    U8(u8),
    U32(u32),
    I32(i32),
    F32(f32),
}

impl Default for Value {
    fn default() -> Self {
        Value::None
    }
}

impl Value {
    pub(crate) fn type_id(&self) -> TypeId {
        TypeId::from(*self)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::None => write!(f, "None"),
            Value::Notification => write!(f, "Notification"),
            Value::Bool(v) => write!(f, "Bool({})", v),
            Value::U8(v) => write!(f, "U8({})", v),
            Value::U32(v) => write!(f, "U32({})", v),
            Value::I32(v) => write!(f, "I32({})", v),
            Value::F32(v) => write!(f, "F32({})", v),
        }
    }
}

// TODO - cleanup this pattern
// Value prefixed with u8 type ID on the wire
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub(crate) enum TypeId {
    None = 0,
    Notification = 1,
    Bool = 2,
    U8 = 3,
    U32 = 4,
    I32 = 5,
    F32 = 6,
}

impl TypeId {
    pub(crate) fn as_u8(&self) -> u8 {
        *self as u8
    }

    // Size of the value field on the ire
    pub(crate) fn wire_size(&self) -> usize {
        match *self {
            TypeId::None => 0,
            TypeId::Notification => 0,
            TypeId::Bool => 1,
            TypeId::U8 => 1,
            TypeId::U32 => 4,
            TypeId::I32 => 4,
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
            4 => TypeId::U32,
            5 => TypeId::I32,
            6 => TypeId::F32,
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
            Value::U32(_) => TypeId::U32,
            Value::I32(_) => TypeId::I32,
            Value::F32(_) => TypeId::F32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem;

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
            TypeId::from(Value::U32(123)).wire_size(),
            mem::size_of::<u32>()
        );
        assert_eq!(
            TypeId::from(Value::I32(-123)).wire_size(),
            mem::size_of::<i32>()
        );
        assert_eq!(
            TypeId::from(Value::F32(-1.234)).wire_size(),
            mem::size_of::<f32>()
        );
    }
}
