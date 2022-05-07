use std::any::Any;

pub trait AsAny: 'static {
    fn as_any(&self) -> &dyn Any;
    fn mut_any(&mut self) -> &mut dyn Any;
    fn to_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<T: 'static> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn mut_any(&mut self) -> &mut dyn Any {
        self
    }
    fn to_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}
