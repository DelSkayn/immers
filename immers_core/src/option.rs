use crate::*;
use std::fmt;

#[derive(Clone)]
pub enum OptionPatch<T: Patchable> {
    /// Change from `Some(A)` to `Some(B)` where `B` is `T::Patch` applied to `A`
    SomeChange(T::Patch),
    /// Change from `None` to `Some(T)`
    SomeCreate(T),
    /// Change from `Some(_)` to `None`
    NoneCreate,
}

#[derive(Debug, Clone)]
pub enum OptionPatchError<T: Patchable> {
    SomePatchMismatch,
    SomeCreateMismatch,
    WithinSome(T::Error),
    NoneCreateMismatch,
}

impl<T: Patchable> fmt::Display for OptionPatchError<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OptionPatchError::SomePatchMismatch => write!(
                fmt,
                "got patch for member of variant `Option::Some` but the value was not this variant"
            ),
            OptionPatchError::SomeCreateMismatch => write!(
                fmt,
                "got a creation patch for member variant `Option::Some` but the value was already this variant"
            ),
            OptionPatchError::WithinSome(ref x) => write!(fmt, "within `Option::Some` => {}", x),
            OptionPatchError::NoneCreateMismatch => write!(
                fmt,
                "got a creation patch for member variant `Option::None` but the value was already this variant"
            ),
        }
    }
}

impl<T: Patchable + Clone> Patchable for Option<T> {
    type Patch = OptionPatch<T>;
    type Error = OptionPatchError<T>;

    fn produce(&self, other: &Self) -> Option<Self::Patch> {
        if let Some(ref s) = *self {
            if let Some(ref o) = *other {
                s.produce(o).map(OptionPatch::SomeChange)
            } else {
                Some(OptionPatch::NoneCreate)
            }
        } else {
            if let Some(ref o) = *other {
                Some(OptionPatch::SomeCreate(o.clone()))
            } else {
                None
            }
        }
    }

    fn apply(&mut self, patch: Self::Patch) -> Result<(), OptionPatchError<T>> {
        match patch {
            OptionPatch::SomeChange(x) => {
                if let Some(s) = self.as_mut() {
                    s.apply(x).map_err(OptionPatchError::WithinSome)
                } else {
                    Err(OptionPatchError::SomePatchMismatch)
                }
            }
            OptionPatch::SomeCreate(x) => {
                if self.is_none() {
                    *self = Some(x);
                    Ok(())
                } else {
                    Err(OptionPatchError::SomeCreateMismatch)
                }
            }
            OptionPatch::NoneCreate => {
                if self.is_none() {
                    Err(OptionPatchError::NoneCreateMismatch)
                } else {
                    *self = None;
                    Ok(())
                }
            }
        }
    }
}
