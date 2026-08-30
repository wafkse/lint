//! Rejects private helper functions that are only used once.

use clippy_utils::diagnostics::span_lint_hir_and_then;
use clippy_utils::{is_from_proc_macro, is_in_test_function};
use rustc_data_structures::fx::{FxIndexMap, IndexEntry};
use rustc_hir::def::DefKind;
use rustc_hir::def_id::LocalDefId;
use rustc_hir::intravisit::{self, Visitor};
use rustc_hir::{BlockCheckMode, Expr, ExprKind, HirId, Node};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_session::impl_lint_pass;
use rustc_span::Span;

rustc_session::declare_lint! {
    /// The `private_single_call_fn` lint rejects private helpers used only once.
    ///
    /// Any explicit visibility such as `pub`, `pub(crate)`, or `pub(self)` is ignored.
    /// Trait methods are also ignored because their visibility is defined by the trait.
    /// Only straight-line bodies with at most six HIR expressions are considered.
    pub PRIVATE_SINGLE_CALL_FN,
    Deny,
    "private helper functions used only once are prohibited"
}

impl_lint_pass!(PrivateSingleCallFn => [PRIVATE_SINGLE_CALL_FN]);

const TRIVIAL_EXPRESSION_LIMIT: usize = 6;

#[derive(Default)]
struct BodyComplexity {
    expressions: usize,
    complex: bool,
}

impl BodyComplexity {
    fn is_trivial(&self) -> bool {
        !self.complex && self.expressions <= TRIVIAL_EXPRESSION_LIMIT
    }
}

impl<'hir> Visitor<'hir> for BodyComplexity {
    fn visit_expr(&mut self, expression: &'hir Expr<'hir>) {
        if self.complex {
            return;
        }

        self.expressions += 1;
        if self.expressions > TRIVIAL_EXPRESSION_LIMIT {
            self.complex = true;
            return;
        }

        if matches!(
            expression.kind,
            ExprKind::If(..)
                | ExprKind::Loop(..)
                | ExprKind::Match(..)
                | ExprKind::Closure(..)
                | ExprKind::Break(..)
                | ExprKind::Continue(..)
                | ExprKind::Ret(..)
                | ExprKind::Become(..)
                | ExprKind::InlineAsm(..)
                | ExprKind::Yield(..)
        ) || matches!(
            expression.kind,
            ExprKind::Block(block, _) if matches!(block.rules, BlockCheckMode::UnsafeBlock(_))
        ) {
            self.complex = true;
            return;
        }

        intravisit::walk_expr(self, expression);
    }
}

#[derive(Clone, Copy)]
enum Usage {
    Once { site: Span },
    Multiple,
}

#[derive(Default)]
pub struct PrivateSingleCallFn {
    usage: FxIndexMap<LocalDefId, Usage>,
}

impl PrivateSingleCallFn {
    fn record_usage(&mut self, def_id: LocalDefId, site: Span) {
        match self.usage.entry(def_id) {
            IndexEntry::Occupied(mut entry) => {
                if matches!(entry.get(), Usage::Once { .. }) {
                    entry.insert(Usage::Multiple);
                }
            }
            IndexEntry::Vacant(entry) => {
                entry.insert(Usage::Once { site });
            }
        }
    }

    fn referenced_function(context: &LateContext<'_>, expression: &Expr<'_>) -> Option<LocalDefId> {
        let def_id = match expression.kind {
            ExprKind::Path(ref path) => context.qpath_res(path, expression.hir_id).opt_def_id(),
            ExprKind::MethodCall(..) => context
                .typeck_results()
                .type_dependent_def_id(expression.hir_id),
            _ => None,
        }?;
        let def_id = def_id.as_local()?;

        if !matches!(context.tcx.def_kind(def_id), DefKind::Fn | DefKind::AssocFn)
            || !matches!(
                context.tcx.hir_node_by_def_id(def_id),
                Node::Item(_) | Node::ImplItem(_)
            )
        {
            return None;
        }

        Some(def_id)
    }

    fn has_trivial_body(context: &LateContext<'_>, def_id: LocalDefId) -> bool {
        let Some(body) = context.tcx.hir_maybe_body_owned_by(def_id) else {
            return false;
        };
        let mut complexity = BodyComplexity::default();

        complexity.visit_expr(body.value);
        complexity.is_trivial()
    }

    fn is_private_helper(
        context: &LateContext<'_>,
        def_id: LocalDefId,
        hir_id: HirId,
        span: Span,
    ) -> bool {
        if span.in_external_macro(context.sess().source_map())
            || context
                .tcx
                .hir_maybe_body_owned_by(def_id)
                .is_some_and(|body| is_in_test_function(context.tcx, body.value.hir_id))
        {
            return false;
        }
        match context.tcx.hir_node(hir_id) {
            Node::Item(item) => item.vis_span.is_empty() && !is_from_proc_macro(context, item),
            Node::ImplItem(item) => {
                item.vis_span().is_some_and(Span::is_empty) && !is_from_proc_macro(context, item)
            }
            _ => false,
        }
    }
}

impl<'tcx> LateLintPass<'tcx> for PrivateSingleCallFn {
    fn check_expr(&mut self, context: &LateContext<'tcx>, expression: &'tcx Expr<'tcx>) {
        if let Some(def_id) = Self::referenced_function(context, expression) {
            self.record_usage(def_id, expression.span);
        }
    }

    fn check_crate_post(&mut self, context: &LateContext<'tcx>) {
        for (&def_id, &usage) in &self.usage {
            let Usage::Once { site } = usage else {
                continue;
            };
            let hir_id = context.tcx.local_def_id_to_hir_id(def_id);
            let span = context.tcx.hir_span_with_body(hir_id);

            if !Self::is_private_helper(context, def_id, hir_id, span)
                || !Self::has_trivial_body(context, def_id)
            {
                continue;
            }

            span_lint_hir_and_then(
                context,
                PRIVATE_SINGLE_CALL_FN,
                hir_id,
                span,
                "this private helper is only used once",
                |diagnostic| {
                    diagnostic.span_note(site, "used here");
                },
            );
        }
    }
}
