//! Enforces explicit receiver destructuring before stored field use.

use clippy_utils::diagnostics::span_lint;
use clippy_utils::{expr_or_init, get_async_fn_body, is_self};
use rustc_hir::def::Res;
use rustc_hir::intravisit::{FnKind, Visitor, walk_stmt};
use rustc_hir::{Body, Expr, ExprKind, FnDecl, LetStmt, Pat, PatKind, QPath, Stmt, StmtKind, UnOp};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::Span;
use rustc_span::def_id::LocalDefId;
use rustc_span::symbol::kw;

rustc_session::declare_lint! {
    /// The `no_self_field_access` lint requires explicit receiver destructuring.
    ///
    /// ### Example
    ///
    /// ```rust
    /// struct Registers { rsi: u64 }
    ///
    /// impl Registers {
    ///     fn rsi(&self) -> &u64 {
    ///         &self.rsi
    ///     }
    /// }
    /// ```
    ///
    /// ### Explanation
    ///
    /// Receiver field access hides which representation fields a method uses.
    /// Methods that derive fields from `self` must destructure `self` in the first statement.
    pub NO_SELF_FIELD_ACCESS,
    Deny,
    "stored fields must be derived from a first-statement self destructure"
}

rustc_session::declare_lint_pass!(NoSelfFieldAccess => [NO_SELF_FIELD_ACCESS]);

impl NoSelfFieldAccess {
    #[inline]
    fn is_self_path(expression: &Expr<'_>) -> bool {
        match expression.kind {
            ExprKind::Path(QPath::Resolved(None, path)) => match (path.res, path.segments) {
                (Res::Local(_), [segment]) => segment.ident.name == kw::SelfLower,
                _ => false,
            },
            _ => false,
        }
    }

    #[inline]
    fn is_self_receiver(context: &LateContext<'_>, expression: &Expr<'_>) -> bool {
        let expression = expr_or_init(context, expression);

        match expression.kind {
            ExprKind::Path(_) => Self::is_self_path(expression),
            ExprKind::AddrOf(_, _, inner)
            | ExprKind::DropTemps(inner)
            | ExprKind::Unary(UnOp::Deref, inner) => Self::is_self_receiver(context, inner),
            _ => false,
        }
    }

    #[inline]
    fn is_self_pattern(pattern: &Pat<'_>) -> bool {
        match pattern.kind {
            PatKind::Struct(path, fields, _) => {
                Self::is_self_pattern_path(path) && !fields.is_empty()
            }
            PatKind::TupleStruct(path, fields, _) => {
                Self::is_self_pattern_path(path) && !fields.is_empty()
            }
            PatKind::Ref(inner, _, _) => Self::is_self_pattern(inner),
            _ => false,
        }
    }

    #[inline]
    fn is_self_pattern_path(path: QPath<'_>) -> bool {
        match path {
            QPath::Resolved(None, path) => {
                matches!(path.res, Res::SelfTyAlias { .. } | Res::SelfTyParam { .. })
            }
            _ => false,
        }
    }

    #[inline]
    fn is_direct_self_destructure(local: &LetStmt<'_>) -> bool {
        match local.init {
            Some(initializer) => {
                Self::is_self_path(initializer) && Self::is_self_pattern(local.pat)
            }
            None => false,
        }
    }

    #[inline]
    fn is_self_derived_destructure(context: &LateContext<'_>, local: &LetStmt<'_>) -> bool {
        match local.init {
            Some(initializer) => {
                Self::is_self_receiver(context, initializer) && Self::is_self_pattern(local.pat)
            }
            None => false,
        }
    }

    #[inline]
    fn first_self_destructure(expression: &Expr<'_>) -> Option<rustc_hir::HirId> {
        match expression.kind {
            ExprKind::Block(block, _) => match block.stmts.first() {
                Some(Stmt {
                    kind: StmtKind::Let(local),
                    ..
                }) => {
                    if Self::is_direct_self_destructure(local) {
                        Some(local.hir_id)
                    } else {
                        None
                    }
                }
                _ => None,
            },
            _ => None,
        }
    }
}

/// Finds receiver destructures that occur after the first method statement.
struct SelfDestructureVisitor<'a, 'tcx> {
    /// The active late lint context.
    context: &'a LateContext<'tcx>,

    /// The allowed first-statement destructure when one exists.
    first_destructure: Option<rustc_hir::HirId>,
}

impl<'tcx> Visitor<'tcx> for SelfDestructureVisitor<'_, 'tcx> {
    #[allow(clippy::single_match)]
    fn visit_stmt(&mut self, statement: &'tcx Stmt<'tcx>) {
        let Self {
            context,
            first_destructure,
        } = self;

        if !statement.span.from_expansion()
            && let StmtKind::Let(local) = statement.kind
            && NoSelfFieldAccess::is_self_derived_destructure(context, local)
        {
            let is_first = *first_destructure == Some(local.hir_id);

            if !is_first {
                span_lint(
                    *context,
                    NO_SELF_FIELD_ACCESS,
                    local.span,
                    "destructure `self` in the first statement of the method",
                );
            }
        }

        walk_stmt(self, statement);
    }
}

impl<'tcx> LateLintPass<'tcx> for NoSelfFieldAccess {
    #[inline]
    #[allow(clippy::collapsible_match, clippy::single_match)]
    fn check_expr(&mut self, context: &LateContext<'tcx>, expression: &'tcx Expr<'tcx>) {
        match (expression.span.from_expansion(), expression.kind) {
            (false, ExprKind::Field(receiver, field)) => {
                if Self::is_self_receiver(context, receiver) {
                    span_lint(
                        context,
                        NO_SELF_FIELD_ACCESS,
                        field.span,
                        "destructure `self` before accessing stored fields",
                    )
                }
            }
            (false, ExprKind::Match(receiver, arms, _)) => match (
                Self::is_self_receiver(context, receiver),
                arms.iter().any(|arm| Self::is_self_pattern(arm.pat)),
            ) {
                (true, true) => span_lint(
                    context,
                    NO_SELF_FIELD_ACCESS,
                    expression.span,
                    "derive stored fields from a first-statement `self` destructure",
                ),
                _ => {}
            },
            _ => {}
        }
    }

    #[inline]
    #[allow(clippy::single_match)]
    fn check_fn(
        &mut self,
        context: &LateContext<'tcx>,
        kind: FnKind<'tcx>,
        _declaration: &'tcx FnDecl<'tcx>,
        body: &'tcx Body<'tcx>,
        span: Span,
        _def_id: LocalDefId,
    ) {
        if span.from_expansion() {
            return;
        }
        let FnKind::Method(_, _) = kind else {
            return;
        };
        let Some(receiver) = body.params.first() else {
            return;
        };

        if !is_self(receiver) {
            return;
        }

        let expression = match kind.asyncness() {
            rustc_hir::IsAsync::Async(_) => get_async_fn_body(context.tcx, body),
            rustc_hir::IsAsync::NotAsync => Some(body.value),
        };

        if let Some(expression) = expression {
            let first_destructure = Self::first_self_destructure(expression);
            let mut visitor = SelfDestructureVisitor {
                context,
                first_destructure,
            };

            visitor.visit_expr(expression);
        }
    }
}
