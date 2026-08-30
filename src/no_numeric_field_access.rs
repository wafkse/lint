//! Rejects numeric field access expressions.

use clippy_utils::diagnostics::span_lint;
use rustc_hir::{Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass};

rustc_session::declare_lint! {
    /// The `no_numeric_field_access` lint rejects unnamed field access.
    ///
    /// ### Example
    ///
    /// ```rust
    /// let pair = (1, 2);
    /// let first = pair.0;
    /// ```
    ///
    /// ### Explanation
    ///
    /// Numeric field access couples code to tuple position.
    /// Destructure the value or use a type with named accessors instead.
    pub NO_NUMERIC_FIELD_ACCESS,
    Deny,
    "numeric field access is prohibited"
}

rustc_session::declare_lint_pass!(NoNumericFieldAccess => [NO_NUMERIC_FIELD_ACCESS]);

impl<'tcx> LateLintPass<'tcx> for NoNumericFieldAccess {
    fn check_expr(&mut self, context: &LateContext<'tcx>, expression: &'tcx Expr<'tcx>) {
        if expression.span.from_expansion() {
            return;
        }

        if let ExprKind::Field(_, field) = expression.kind
            && field.is_numeric()
        {
            span_lint(
                context,
                NO_NUMERIC_FIELD_ACCESS,
                field.span,
                "numeric field access is prohibited",
            );
        }
    }
}
