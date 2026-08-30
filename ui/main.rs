#![allow(dead_code)]

// NOTE(invariant): The field is intentionally opaque for the fixture.
struct Named {
    value: u8,
}

// NOTE(invariant): The wrapped value is intentionally opaque for the fixture.
struct Wrapper(u8);

fn main() {
    let tuple = (1_u8, 2_u8);
    let _ = tuple.0;

    let wrapper = Wrapper(3);
    let _ = wrapper.0;

    let named = Named { value: 4 };
    let _ = named.value;

    let array = [5_u8];
    let _ = array[0];
}
