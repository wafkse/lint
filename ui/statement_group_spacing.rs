#![allow(dead_code)]

fn effect() {}

fn simple_to_expression() {
    let first = 1_u8;
    let second = 2_u8;
    effect();
    effect();
}

fn expression_to_simple() {
    effect();
    effect();
    let first = 1_u8;
    let second = 2_u8;
}

fn structural_neighbors() {
    let first = 1_u8;
    let (left, right) = (2_u8, 3_u8);
    let last = 4_u8;

    let _ = (first, left, right, last);
}

fn structural_pair() {
    let (first, second) = (1_u8, 2_u8);
    let [third, fourth] = [3_u8, 4_u8];

    let _ = (first, second, third, fourth);
}

fn let_else_is_structural() {
    let input = Some(1_u8);
    let Some(value) = input else {
        return;
    };
    let output = value;

    let _ = output;
}

fn comment_is_not_blank() {
    let value = 1_u8;
    // The comment does not create an empty physical line.
    effect();

    let _ = value;
}

fn allowed_groups() {
    let first = 1_u8;
    let second = 2_u8;

    effect();
    effect();

    let third = 3_u8;
    let fourth = 4_u8;

    let (left, right) = (first, second);

    effect();

    fn local_one() {}
    fn local_two() {}

    let fifth = third + fourth + left + right;
    let _ = fifth;
}

fn simple_to_tail() -> u8 {
    let value = 1_u8;
    value
}

fn allowed_tail() -> u8 {
    let value = 1_u8;

    value
}

macro_rules! generated_binding {
    () => {
        let _generated = 1_u8;
    };
}

fn expansion_is_ignored() {
    let before = 1_u8;
    generated_binding!();
    let after = 2_u8;

    let _ = (before, after);
}

fn main() {}

trait AutomaticallyDerivedFixture {
    fn generated();
}

#[automatically_derived]
impl AutomaticallyDerivedFixture for () {
    fn generated() {
        let (left, right) = (1_u8, 2_u8);
        let sum = left + right;
        let _ = sum;
    }
}
