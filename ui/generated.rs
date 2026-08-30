#![allow(dead_code)]

#[derive(Clone, Debug)]
// NOTE(invariant): The wrapped value is intentionally opaque for the fixture.
struct Tuple(u8);

#[derive(Clone, Debug)]
// NOTE(invariant): The field is intentionally opaque for the fixture.
struct Named {
    value: u8,
}

fn main() {}
