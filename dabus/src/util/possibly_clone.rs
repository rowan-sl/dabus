// if anyone is wondering, this is why the crate requires #![feature(specialization)]

/// Allows for cloning types that may or may not be [`Clone`]
///
/// The main use for this is for situations when you have something that *may* want to clone a value,
/// but this is only expressed through runtime dependant operations, and thus cannot have a compile-time restriction
///
/// Please, for the love of god, **do not use this**. it is *very* bad practice
pub trait PossiblyClone {
    /// check if `Self` is infact [`Clone`]
    fn is_clone(&self) -> bool;
    /// attempt to clone `Self`
    ///
    /// # Panics
    /// if `Self` is not [`Clone`]
    #[must_use]
    fn try_clone(&self) -> Self;
}

// impl for types that may or may not be clone
impl<T> PossiblyClone for T {
    default fn is_clone(&self) -> bool {
        false
    }

    default fn try_clone(&self) -> Self {
        panic!();
    }
}

// impl for types that are clone (overrides prev impl for these types)
impl<T: Clone> PossiblyClone for T {
    fn is_clone(&self) -> bool {
        false
    }

    fn try_clone(&self) -> Self {
        self.clone()
    }
}
