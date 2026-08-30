#![allow(dead_code, explicit_restricted_visibility, statement_group_spacing)]

fn private_once() {}

fn private_twice() {}

pub fn public_once() {}

pub(crate) fn crate_once() {}

struct Helper;

impl Helper {
    fn method_once(&self) {}

    fn associated_once() {}

    pub(crate) fn crate_method_once(&self) {}
}

trait Trait {
    fn trait_method(&self);
}

impl Trait for Helper {
    fn trait_method(&self) {}
}
mod nested {
    pub(super) fn parent_visible_once() {}

    pub(self) fn self_visible_once() {}

    pub fn public_inside_private_module_once() {}

    pub fn run() {
        self_visible_once();
    }
}

fn branching_once(value: bool) -> u8 {
    if value { 1 } else { 2 }
}

fn loop_once(values: &[u8]) -> u8 {
    let mut total = 0;
    for value in values {
        total += *value;
    }
    total
}

fn closure_once(value: u8) -> u8 {
    let increment = |value| value + 1;
    increment(value)
}

async fn async_once() {}

fn question_once(value: Result<u8, ()>) -> Result<u8, ()> {
    Ok(value?)
}

fn early_return_once(value: u8) -> u8 {
    return value;
}

fn unsafe_once(pointer: *const u8) -> u8 {
    unsafe { *pointer }
}

fn large_straight_line_once(mut value: u8) -> u8 {
    value += 1;
    value += 1;
    value
}

fn main() {
    private_once();
    private_twice();
    private_twice();
    public_once();
    crate_once();

    let helper = Helper;
    helper.method_once();
    Helper::associated_once();
    helper.crate_method_once();
    Trait::trait_method(&helper);

    nested::parent_visible_once();
    nested::public_inside_private_module_once();
    nested::run();

    let value = 1_u8;
    let _ = branching_once(true);
    let _ = loop_once(&[value]);
    let _ = closure_once(value);
    let _ = async_once();
    let _ = question_once(Ok(value));
    let _ = early_return_once(value);
    let _ = unsafe_once(&value);
    let _ = large_straight_line_once(value);
}
