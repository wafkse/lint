#![allow(dead_code)]

pub(crate) fn crate_shorthand() {}

pub(in crate) fn bare_crate_path() {}

// NOTE(rationale): This fixture intentionally exercises the crate-root escape.
pub(crate) fn justified_crate_root() {}

// NOTE(invariant): The restricted field is part of an opaque representation.
struct Fields {
    pub(in crate) rejected: u8,

    // NOTE(rationale): The fixture exercises a justified field visibility.
    pub(crate) justified: u8,
}

mod outer {
    pub mod inner {
        pub(super) fn super_shorthand() {}

        pub(in super) fn bare_super_path() {}

        pub(in crate::outer) fn explicit_ancestor_path() {}

        // NOTE(rationale): The fixture exercises a justified parent visibility.
        pub(super) fn justified_parent() {}
    }
}

fn main() {}
