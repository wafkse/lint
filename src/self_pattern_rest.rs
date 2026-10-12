//! Requires rest syntax for repeated edge wildcards in local `Self` tuple patterns.
//!
//! Existing rest patterns are excluded so the diagnostic never asks for a second
//! `..` or changes the positions of retained field bindings.

use clippy_utils::diagnostics::span_lint_and_help;
use rustc_hir::def::Res;
use rustc_hir::{LetStmt, Pat, PatKind, QPath};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::Span;

rustc_session::declare_lint! {
    /// The `self_pattern_rest` lint requires `..` for repeated edge wildcards.
    ///
    /// ### Example
    ///
    /// ```rust
    /// let Self(value, _, _) = self;
    /// // Prefer `let Self(value, ..) = self;`.
    /// ```
    ///
    /// ### Explanation
    ///
    /// Two or more plain wildcards at an edge can be replaced with rest syntax.
    /// Single wildcards, interior runs, and patterns already using `..` are ignored.
    pub SELF_PATTERN_REST,
    Deny,
    "repeated edge wildcards in Self patterns must use rest syntax"
}

rustc_session::declare_lint_pass!(SelfPatternRest => [SELF_PATTERN_REST]);

impl SelfPatternRest {
    /// Selects a replaceable edge without introducing multiple rest patterns.
    /// Leading wildcards win when both edges qualify.
    fn wildcard_span(pattern: &Pat<'_>) -> Option<Span> {
        if pattern.span.from_expansion() {
            return None;
        }

        match pattern.kind {
            PatKind::Ref(inner, _, _) => Self::wildcard_span(inner),
            PatKind::TupleStruct(QPath::Resolved(None, path), fields, rest)
                if matches!(path.res, Res::SelfCtor(_)) && rest.as_opt_usize().is_none() =>
            {
                let leading = fields
                    .iter()
                    .take_while(|field| matches!(field.kind, PatKind::Wild))
                    .count();
                let trailing = fields
                    .iter()
                    .rev()
                    .take_while(|field| matches!(field.kind, PatKind::Wild))
                    .count();

                let run = if leading >= 2 {
                    &fields[..leading]
                } else if trailing >= 2 {
                    &fields[fields.len() - trailing..]
                } else {
                    return None;
                };

                if run.iter().any(|field| field.span.from_expansion()) {
                    return None;
                }

                Some(run.first()?.span.to(run.last()?.span))
            }
            _ => None,
        }
    }
}

impl<'tcx> LateLintPass<'tcx> for SelfPatternRest {
    fn check_local(&mut self, context: &LateContext<'tcx>, local: &'tcx LetStmt<'tcx>) {
        if !local.span.from_expansion()
            && let Some(span) = Self::wildcard_span(local.pat)
        {
            span_lint_and_help(
                context,
                SELF_PATTERN_REST,
                span,
                "replace consecutive edge `_` patterns with `..`",
                None,
                "use `..` for this run and retain the other field patterns",
            );
        }
    }
}
