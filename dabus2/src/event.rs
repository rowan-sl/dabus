use std::marker::PhantomData;

/// type for declaring events.
///
/// example:
///
/// ```rust
/// use dabus2::event::EventDef;
///
/// static TEST_EVENT: &'static EventDef<unique_type::new!(), ()> = &EventDef::new();
/// ```
pub struct EventDef<Tag: unique_type::Unique, At, Rt=()/* if At is `()`, than this event is eligeble for lazy evaluation */> {
    _tag: PhantomData<*const Tag/* dropck */>,
    _at: PhantomData<*const At/* also dropck */>,
    _rt: PhantomData<*const Rt/* also dropck */>,
}

// its just because raw ptrs are not Sync or Send or Smth (but we dont actually have a raw ptr, we have a PhantomData)
unsafe impl<Tag: unique_type::Unique, At, Rt> Sync for EventDef<Tag, At, Rt> {}
unsafe impl<Tag: unique_type::Unique, At, Rt> Send for EventDef<Tag, At, Rt> {}

impl<Tag: unique_type::Unique, At, Rt> EventDef<Tag, At, Rt> {
    pub const fn new() -> Self {
        Self { _tag: PhantomData, _at: PhantomData, _rt: PhantomData }
    }
}
