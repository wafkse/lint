//! Enforces structure-owned invariant rationale comments.

use std::collections::BTreeSet;

use clippy_utils::diagnostics::span_lint;
use rustc_ast::ast::{Crate, Item, ItemKind};
use rustc_lint::{EarlyContext, EarlyLintPass};

use crate::source_comments::MarkerIndex;

rustc_session::declare_lint! {
    /// The `struct_invariant_note` lint ties invariant notes to opaque structs.
    ///
    /// A struct with at least one non-public field must have exactly one
    /// adjacent ordinary comment containing `NOTE(invariant):`. The marker is
    /// rejected everywhere else.
    pub STRUCT_INVARIANT_NOTE,
    Deny,
    "opaque structures require one structure-owned invariant rationale"
}

/// Lint state for invariant marker ownership.
#[derive(Default)]
pub struct StructInvariantNote {
    /// Source marker index built while authored files are visited.
    markers: MarkerIndex,

    /// Marker start positions accepted by qualifying structures.
    accepted_markers: BTreeSet<u32>,
}

rustc_session::impl_lint_pass!(StructInvariantNote => [STRUCT_INVARIANT_NOTE]);
impl EarlyLintPass for StructInvariantNote {
    fn check_item(&mut self, context: &EarlyContext<'_>, item: &Item) {
        self.markers.scan_file(context, item.span);

        if item.span.from_expansion() {
            return;
        }
        let ItemKind::Struct(ident, _, data) = &item.kind else {
            return;
        };
        let has_non_public_field = data.fields().iter().any(|field| !field.vis.kind.is_pub());
        let marker_list = MarkerIndex::adjacent(context, ident.span, "NOTE(invariant):");

        match (has_non_public_field, marker_list.as_slice()) {
            (true, [marker]) => {
                self.accepted_markers.insert(marker.span.lo().0);
            }
            (true, []) => span_lint(
                context,
                STRUCT_INVARIANT_NOTE,
                ident.span,
                "struct with non-public fields requires one adjacent `NOTE(invariant):` comment",
            ),
            (true, _) => span_lint(
                context,
                STRUCT_INVARIANT_NOTE,
                ident.span,
                "struct with non-public fields requires exactly one adjacent `NOTE(invariant):` comment",
            ),
            (false, _) => (),
        }
    }

    fn check_crate_post(&mut self, context: &EarlyContext<'_>, _krate: &Crate) {
        for marker in self.markers.invariant_markers() {
            if !self.accepted_markers.contains(&marker.span.lo().0) {
                span_lint(
                    context,
                    STRUCT_INVARIANT_NOTE,
                    marker.span,
                    "`NOTE(invariant):` is only allowed as the single adjacent rationale for a struct with non-public fields",
                );
            }
        }
    }
}
