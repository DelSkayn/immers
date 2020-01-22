pub use immers_core::*;
pub use immers_derive::*;

#[derive(Patchable)]
#[patchable(derive(Debug, Clone))]
pub struct Test {
    a: u16,
    b: u32,
    c: u8,
}
