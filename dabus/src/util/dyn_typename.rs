use std::any::{Any, type_name};

pub trait TypeNamed {
    fn type_name(&self) -> &'static str;
}

impl<T: Any + 'static> TypeNamed for T {
    fn type_name(&self) -> &'static str {
        type_name::<T>()
    }
}