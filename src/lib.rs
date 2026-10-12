#![feature(rustc_private)]
#![warn(unused_extern_crates)]

//! Personal Rust lints.

extern crate rustc_ast;
extern crate rustc_data_structures;
extern crate rustc_hir;
extern crate rustc_lexer;
extern crate rustc_lint;
extern crate rustc_session;
extern crate rustc_span;

mod explicit_restricted_visibility;
mod no_numeric_field_access;
mod no_self_field_access;
mod private_single_call_fn;
mod self_pattern_rest;
mod source_comments;
mod statement_group_spacing;
mod struct_invariant_note;

use explicit_restricted_visibility::{
    EXPLICIT_RESTRICTED_VISIBILITY, ExplicitRestrictedVisibility,
};
use no_numeric_field_access::{NO_NUMERIC_FIELD_ACCESS, NoNumericFieldAccess};
use no_self_field_access::{NO_SELF_FIELD_ACCESS, NoSelfFieldAccess};
use private_single_call_fn::{PRIVATE_SINGLE_CALL_FN, PrivateSingleCallFn};
use rustc_lint::LintStore;
use rustc_session::Session;
use self_pattern_rest::{SELF_PATTERN_REST, SelfPatternRest};
use statement_group_spacing::{STATEMENT_GROUP_SPACING, StatementGroupSpacing};
use struct_invariant_note::{STRUCT_INVARIANT_NOTE, StructInvariantNote};

dylint_linting::dylint_library!();

/// Registers all personal lints with Dylint.
#[unsafe(no_mangle)]
pub fn register_lints(_session: &Session, lint_store: &mut LintStore) {
    lint_store.register_lints(&[
        EXPLICIT_RESTRICTED_VISIBILITY,
        NO_NUMERIC_FIELD_ACCESS,
        NO_SELF_FIELD_ACCESS,
        PRIVATE_SINGLE_CALL_FN,
        SELF_PATTERN_REST,
        STATEMENT_GROUP_SPACING,
        STRUCT_INVARIANT_NOTE,
    ]);
    lint_store.register_early_pass(|| Box::new(ExplicitRestrictedVisibility));
    lint_store.register_early_pass(|| Box::new(StructInvariantNote::default()));
    lint_store.register_late_pass(|_| Box::new(NoNumericFieldAccess));
    lint_store.register_late_pass(|_| Box::new(NoSelfFieldAccess));
    lint_store.register_late_pass(|_| Box::new(SelfPatternRest));
    lint_store.register_late_pass(|_| Box::new(PrivateSingleCallFn::default()));
    lint_store.register_late_pass(|_| Box::new(StatementGroupSpacing));
}

#[test]
fn ui() {
    let mut test = dylint_testing::ui::Test::src_base(env!("CARGO_PKG_NAME"), "ui");

    test.rustc_flags(["--edition=2024"]);
    test.run();
}
