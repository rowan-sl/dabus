use std::any::Any;

/// This trait allows downcasting `dyn Trait` to `dyn Any`,
/// which is neccesary when using the `downcast` functions of Any with another trait
///
/// # Examples
///
/// ```rust
/// use std::any::Any;
/// use dabus::extras::AsAny;
///
/// // you have this struct
/// struct Foo;
///
/// // and want to use it as `dyn Bar`
/// trait Bar: AsAny {
///     fn do_something(&self);
/// }
///
/// impl Bar for Foo {
///     fn do_something(&self) {
///         // some work is done here probably
///     }
/// }
/// # fn main() {
/// // but how do you do that?
/// let dyn_bar: Box<dyn Bar> = Box::new(Foo);
///
/// // use it as that trait
/// dyn_bar.do_something();
///
/// // using AsAny, you can cast dyn Bar to dyn Any, and then call .downcast() on it
/// let foo_again: Foo = *dyn_bar.to_any().downcast().unwrap();
/// // tada
/// # }
/// ```
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
