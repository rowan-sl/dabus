/// very very cursed
pub trait PossiblyClone {
    fn is_clone(&self) -> bool;
    fn try_clone(&self) -> Self;
}

impl<T> PossiblyClone for T {
    default fn is_clone(&self) -> bool {
        false
    }

    default fn try_clone(&self) -> Self {
        panic!();
    }
}

impl<T: Clone> PossiblyClone for T {
    fn is_clone(&self) -> bool {
        false
    }

    fn try_clone(&self) -> Self {
        self.clone()
    }
}
