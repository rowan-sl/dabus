/// very very cursed
pub trait PossiblyClone {
    const IS_CLONE: bool;
    fn try_clone(&self) -> Self;
}

impl<T> PossiblyClone for T {
    default const IS_CLONE: bool = false;

    default fn try_clone(&self) -> Self {
        panic!();
    }
}

impl<T: Clone> PossiblyClone for T {
    const IS_CLONE: bool = true;

    fn try_clone(&self) -> Self {
        self.clone()
    }
}
