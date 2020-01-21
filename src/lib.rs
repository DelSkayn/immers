use immers_core::*;
use immers_derive::*;

#[derive(Patchable)]
pub struct Test {
    foo: u32,
    bar: i16,
    baz: Option<u16>,
}

#[derive(Patchable)]
pub struct Foo(u32, i32, f32);
