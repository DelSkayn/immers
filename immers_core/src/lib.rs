use std::{convert::Infallible, fmt};

mod option;
pub use option::{OptionPatch, OptionPatchError};

pub trait Patchable {
    type Patch: Clone;
    type Error: fmt::Display;

    /// Takes a value and produces a patches which when applied to the original object results in
    /// makes it equal to the taken value
    fn produce(&self, other: &Self) -> Option<Self::Patch>;

    /// Takes a patch and applies it to the current object.
    /// Returns an error if the patch cannot be applied to the current object.
    /// An example of invalid patch would be a patch which alters the value of a variant of the
    /// enum for a enum which is not at that value.
    fn apply(&mut self, patch: Self::Patch) -> Result<(), Self::Error>;
}

impl<T: Patchable> Patchable for Box<T> {
    type Patch = T::Patch;
    type Error = T::Error;

    fn produce(&self, other: &Self) -> Option<Self::Patch> {
        (**self).produce(other)
    }

    fn apply(&mut self, patch: Self::Patch) -> Result<(), Self::Error> {
        (**self).apply(patch)
    }
}

macro_rules! impl_patchable_primitive {
    ($x:ty) => {
        impl Patchable for $x {
            type Patch = Self;
            type Error = Infallible; //Make ! once it is stableized

            fn produce(&self, other: &Self) -> Option<Self::Patch> {
                if *self != *other {
                    Some(other.clone())
                } else {
                    None
                }
            }

            fn apply(&mut self, patch: Self::Patch) -> Result<(), Self::Error> {
                *self = patch;
                Ok(())
            }
        }
    };
}

macro_rules! impl_patchable_primitives{
    ($x:ty) => {
        impl_patchable_primitive!($x);
    };
    ($first:ty, $($rest:ty),+) => {
        impl_patchable_primitive!($first);
        impl_patchable_primitives!($($rest),*);
    };
}

impl_patchable_primitives!(
    u8,
    i8,
    u16,
    i16,
    u32,
    i32,
    f32,
    u64,
    i64,
    f64,
    usize,
    isize,
    String,
    bool,
    (),
    char
);

#[derive(Clone)]
enum TestPatch {
    Foo(u32),
    Bar(OptionPatch<u16>),
    Baz(i16),
}

enum TestPatchError {
    Foo(Infallible),
    Bar(OptionPatchError<u16>),
    Baz(Infallible),
}

impl fmt::Display for TestPatchError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TestPatchError::Foo(ref x) => write!(fmt, "within `foo` => {}", x),
            TestPatchError::Bar(ref x) => write!(fmt, "within `bar` => {}", x),
            TestPatchError::Baz(ref x) => write!(fmt, "within `baz` => {}", x),
        }
    }
}

#[derive(Clone)]
struct Test {
    foo: u32,
    bar: Option<u16>,
    baz: i16,
}

impl Patchable for Test {
    type Patch = Vec<TestPatch>;
    type Error = TestPatchError;

    fn produce(&self, other: &Self) -> Option<Self::Patch> {
        let mut patch = Vec::new();
        if let Some(x) = self.foo.produce(&other.foo) {
            patch.push(TestPatch::Foo(x));
        }
        if let Some(x) = self.bar.produce(&other.bar) {
            patch.push(TestPatch::Bar(x));
        }
        if let Some(x) = self.baz.produce(&other.baz) {
            patch.push(TestPatch::Baz(x));
        }
        if patch.len() == 0 {
            None
        } else {
            Some(patch)
        }
    }

    fn apply(&mut self, patch: Self::Patch) -> Result<(), Self::Error> {
        for v in patch {
            match v {
                TestPatch::Foo(x) => {
                    self.foo.apply(x).map_err(TestPatchError::Foo)?;
                }
                TestPatch::Bar(x) => {
                    self.bar.apply(x).map_err(TestPatchError::Bar)?;
                }
                TestPatch::Baz(x) => {
                    self.baz.apply(x).map_err(TestPatchError::Baz)?;
                }
            }
        }
        Ok(())
    }
}
