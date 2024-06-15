use rmpv::Value;

use crate::api::{
    Buffer,
    Tabpage,
    Window
};

/// Trait to convert any type to rmpv::Value
pub trait AsValue {
    fn convert(&self) -> Value;
}

/// Macro to implement AsValue trait for a builtin type
macro_rules! impl_asvalue {
    ($arg:ty) => {
        impl AsValue for $arg {
            fn convert(&self) -> Value {
                Value::from(*self)
            }
        }
    };
}

/// Macro to implement AsValue trait for a tuple of builtin types
macro_rules! impl_asvalue_tuple {
    ($($arg:ty), +) => {
        impl AsValue for ($($arg), +) {
            fn convert(&self) -> Value {
                Value::from(0)
            }
        }
    };
}

// Implement AsValue for builtin types
impl_asvalue!(u64);
impl_asvalue!(i64);
impl_asvalue!(f64);
impl_asvalue!(bool);


// Implement AsValue for builtin tuples
impl_asvalue_tuple!(i64, i64);

impl AsValue for Value {
    fn convert(&self) -> Value {
        self.clone()
    }
}

impl AsValue for String {
    fn convert(&self) -> Value {
        Value::from(self.clone())
    }
}

impl AsValue for Vec<Value> {
    fn convert(&self) -> Value {
        Value::from(self.clone())
    }
}

impl AsValue for Vec<String> {
    fn convert(&self) -> Value {
        let v: Vec<Value> = self.iter().map(|x| Value::from(x.clone())).collect();
        Value::from(v)
    }
}

impl AsValue for Buffer {
    fn convert(&self) -> Value {
        self.data.clone()
    }
}

impl AsValue for Tabpage {
    fn convert(&self) -> Value {
        self.data.clone()
    }
}

impl AsValue for Window {
    fn convert(&self) -> Value {
        self.data.clone()
    }
}

impl AsValue for Vec<(Value, Value)> {
    fn convert(&self) -> Value {
        Value::Map(self.clone())
    }
}
