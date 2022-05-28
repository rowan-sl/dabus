use std::{any::TypeId, fmt::Debug};

use crate::util::GeneralRequirements;

/// value must be Debug, just to make things easy
pub struct DynVar {
    val: Box<dyn GeneralRequirements + Sync + Send + 'static>,
}

impl DynVar {
    #[must_use]
    pub fn new<T: GeneralRequirements + Sync + Send + 'static>(x: T) -> Self {
        Self { val: Box::new(x) }
    }

    #[must_use]
    pub fn to_raw(self) -> Box<dyn GeneralRequirements> {
        self.val
    }

    #[must_use]
    pub fn from_raw(val: Box<dyn GeneralRequirements + Sync + Send + 'static>) -> Self {
        Self { val }
    }

    #[must_use]
    pub fn type_name(&self) -> &'static str {
        (*self.val).type_name()
    }

    #[must_use]
    pub fn as_ref<T: GeneralRequirements>(&self) -> Option<&T> {
        (*self.val).as_any().downcast_ref()
    }

    #[must_use]
    pub fn as_mut<T: GeneralRequirements>(&mut self) -> Option<&mut T> {
        (*self.val).mut_any().downcast_mut()
    }

    pub fn try_to<T: GeneralRequirements>(self) -> Result<T, Self> {
        if (*self.val).as_any().type_id() == TypeId::of::<T>() {
            Ok(unsafe { *self.val.to_any().downcast().unwrap_unchecked() })
        } else {
            Err(self)
        }
    }

    #[must_use]
    pub unsafe fn as_ref_unchecked<T: GeneralRequirements>(&self) -> &T {
        (*self.val).as_any().downcast_ref_unchecked()
    }

    #[must_use]
    pub unsafe fn as_mut_unchecked<T: GeneralRequirements>(&mut self) -> &mut T {
        (*self.val).mut_any().downcast_mut_unchecked()
    }

    #[must_use]
    pub unsafe fn try_to_unchecked<T: GeneralRequirements>(self) -> T {
        *self.val.to_any().downcast_unchecked()
    }

    #[must_use]
    pub fn is<T: GeneralRequirements>(&self) -> bool {
        (*self.val).as_any().type_id() == TypeId::of::<T>()
    }

    #[must_use]
    pub fn clone_as<T: GeneralRequirements + Clone + Sync + Send + 'static>(&self) -> Option<Self> {
        Some(Self {
            val: Box::new(self.as_ref::<T>()?.clone()),
        })
    }

    #[must_use]
    pub unsafe fn clone_as_unchecked<T: GeneralRequirements + Clone + Sync + Send + 'static>(
        &self,
    ) -> Self {
        Self {
            val: Box::new(self.as_ref_unchecked::<T>().clone()),
        }
    }
}

impl Debug for DynVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynVar")
            .field("val", self.val.as_dbg())
            .finish()
    }
}
