#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum Value {
    None,
    // TODO - not sure unidirectional Notification fits here
    Notification,
    Bool(bool),
    U8(u8),
    U32(u32),
    F32(f32),
}

impl Default for Value {
    fn default() -> Self {
        Value::None
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
    F32 = 5,
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
            5 => TypeId::F32,
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
            Value::F32(_) => TypeId::F32,
        }
    }
}
