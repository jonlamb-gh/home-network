#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum Value {
    None,
    // TODO - not sure unidirectional Notification fits here
    Notification,
    U8(u8),
    U32(u32),
    F32(f32),
}

impl Default for Value {
    fn default() -> Self {
        Value::None
    }
}

// Value prefixed with u8 type ID on the wire
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub(crate) enum TypeId {
    None = 0,
    Notification = 1,
    U8 = 2,
    U32 = 3,
    F32 = 4,
}

impl TypeId {
    pub(crate) fn as_u8(&self) -> u8 {
        *self as u8
    }
}

impl From<Value> for TypeId {
    fn from(v: Value) -> TypeId {
        match v {
            Value::None => TypeId::None,
            Value::Notification => TypeId::Notification,
            Value::U8(_) => TypeId::U8,
            Value::U32(_) => TypeId::U32,
            Value::F32(_) => TypeId::F32,
        }
    }
}
