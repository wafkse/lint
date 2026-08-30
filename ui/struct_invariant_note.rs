#![allow(dead_code)]

struct Missing {
    value: u8,
}

// NOTE(invariant): The private field is validated before construction.
struct Valid {
    value: u8,
}

// NOTE(invariant): Public storage does not enforce a representation invariant.
struct FullyPublic {
    pub value: u8,
}

struct Misplaced {
    // NOTE(invariant): A field comment does not own the structure invariant.
    value: u8,
}

// NOTE(invariant): First rationale.
// NOTE(invariant): Second rationale.
struct Duplicate {
    value: u8,
}

fn main() {}
