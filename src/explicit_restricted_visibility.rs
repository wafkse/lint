//! Requires explicit paths for restricted visibility shorthands.

use clippy_utils::diagnostics::span_lint;
use rustc_ast::ast::{AssocItemKind, Item, ItemKind, Visibility, VisibilityKind};
use rustc_lint::{EarlyContext, EarlyLintPass};
use rustc_span::Span;
use rustc_span::symbol::kw;

use crate::source_comments::MarkerIndex;

rustc_session::declare_lint! {
    /// The `explicit_restricted_visibility` lint rejects crate-root and parent-only visibility.
    ///
    /// `pub(crate)`, `pub(super)`, `pub(in crate)`, and `pub(in super)` do not
    /// identify the owning subsystem. Restricted visibility must name a real
    /// ancestor module unless the declaration has an adjacent `NOTE(rationale):` comment.
    pub EXPLICIT_RESTRICTED_VISIBILITY,
    Deny,
    "restricted visibility must name an ancestor module or carry a rationale"
}

rustc_session::declare_lint_pass!(ExplicitRestrictedVisibility => [EXPLICIT_RESTRICTED_VISIBILITY]);

impl ExplicitRestrictedVisibility {
    fn check_visibility(context: &EarlyContext<'_>, visibility: &Visibility, target_span: Span) {
        let VisibilityKind::Restricted { path, .. } = &visibility.kind else {
            return;
        };

        let [only_segment] = &*path.segments else {
            return;
        };
        let target = only_segment.ident.name;
        let prohibited = target == kw::Crate || target == kw::Super;

        if !prohibited {
            return;
        }

        let rationale = MarkerIndex::adjacent(context, target_span, "NOTE(rationale):");

        if !rationale.is_empty() {
            return;
        }
        span_lint(
            context,
            EXPLICIT_RESTRICTED_VISIBILITY,
            visibility.span,
            "restricted visibility must name an explicit ancestor module",
        );
    }
}

impl EarlyLintPass for ExplicitRestrictedVisibility {
    fn check_item(&mut self, context: &EarlyContext<'_>, item: &Item) {
        if item.span.from_expansion() {
            return;
        }

        Self::check_visibility(context, &item.vis, item.vis.span);

        if let ItemKind::Struct(_, _, data) = &item.kind {
            for field in data.fields() {
                Self::check_visibility(context, &field.vis, field.vis.span);
            }
        }
    }

    fn check_impl_item(&mut self, context: &EarlyContext<'_>, item: &Item<AssocItemKind>) {
        if !item.span.from_expansion() {
            Self::check_visibility(context, &item.vis, item.vis.span);
        }
    }
}
