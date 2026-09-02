//! Enforces blank lines between structurally distinct statement groups.

use clippy_utils::diagnostics::span_lint;
use clippy_utils::in_automatically_derived;
use rustc_hir::{Block, PatKind, Stmt, StmtKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_span::Span;

rustc_session::declare_lint! {
    /// The `statement_group_spacing` lint separates distinct statement groups.
    ///
    /// ### Example
    ///
    /// ```rust
    /// let input = parse()?;
    /// let output = validate(input)?;
    ///
    /// emit(output);
    /// ```
    ///
    /// ### Explanation
    ///
    /// Simple bindings may form one derivation group. Expression statements and
    /// local items form their own groups. Structural bindings such as struct,
    /// tuple, slice, reference, or `let else` patterns stand alone.
    pub STATEMENT_GROUP_SPACING,
    Deny,
    "distinct statement groups must be separated by a blank line"
}

rustc_session::declare_lint_pass!(StatementGroupSpacing => [STATEMENT_GROUP_SPACING]);

/// Structural class used to determine statement paragraph boundaries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StatementGroup {
    /// Consecutive identifier or wildcard bindings.
    SimpleBinding,

    /// A pattern binding that exposes or validates structure.
    StructuralBinding,

    /// An expression statement with or without a semicolon.
    Expression,

    /// A local item declaration.
    Item,
}

impl StatementGroup {
    /// Classify one authored statement by its source-level role.
    fn of(statement: &Stmt<'_>) -> Self {
        match statement.kind {
            StmtKind::Let(local) => match (local.els.is_some(), local.pat.kind) {
                (false, PatKind::Binding(_, _, _, None) | PatKind::Wild) => Self::SimpleBinding,
                _ => Self::StructuralBinding,
            },
            StmtKind::Expr(_) | StmtKind::Semi(_) => Self::Expression,
            StmtKind::Item(_) => Self::Item,
        }
    }

    /// Return whether adjacency requires an empty source line.
    const fn requires_separator(self, following: Self) -> bool {
        matches!(self, Self::StructuralBinding)
            || matches!(following, Self::StructuralBinding)
            || !matches!(
                (self, following),
                (Self::SimpleBinding, Self::SimpleBinding)
                    | (Self::Expression, Self::Expression)
                    | (Self::Item, Self::Item)
            )
    }
}

impl StatementGroupSpacing {
    /// Return whether authored source contains an empty physical line between spans.
    fn has_blank_line(context: &LateContext<'_>, preceding: Span, following: Span) -> bool {
        let gap = preceding.between(following);
        let source_map = context.sess().source_map();
        let Ok(source) = source_map.span_to_snippet(gap) else {
            return true;
        };
        let line_list = source.split('\n').collect::<Vec<_>>();

        match line_list.as_slice() {
            [_, _] | [_] | [] => false,
            [_, middle @ .., _] => middle.iter().any(|line| line.trim().is_empty()),
        }
    }

    /// Check one authored adjacency between structural statement groups.
    fn check_adjacency(
        context: &LateContext<'_>,
        preceding_span: Span,
        preceding_group: StatementGroup,
        following_span: Span,
        following_group: StatementGroup,
    ) {
        let separated = Self::has_blank_line(context, preceding_span, following_span);

        if preceding_group.requires_separator(following_group) && !separated {
            span_lint(
                context,
                STATEMENT_GROUP_SPACING,
                following_span.shrink_to_lo(),
                "separate this statement group from the preceding group with a blank line",
            );
        }
    }
}

impl<'tcx> LateLintPass<'tcx> for StatementGroupSpacing {
    fn check_block(&mut self, context: &LateContext<'tcx>, block: &'tcx Block<'tcx>) {
        if block.span.from_expansion() || in_automatically_derived(context.tcx, block.hir_id) {
            return;
        }

        for pair in block.stmts.windows(2) {
            let [preceding, following] = pair else {
                continue;
            };

            if preceding.span.from_expansion() || following.span.from_expansion() {
                continue;
            }

            let preceding_group = StatementGroup::of(preceding);
            let following_group = StatementGroup::of(following);

            Self::check_adjacency(
                context,
                preceding.span,
                preceding_group,
                following.span,
                following_group,
            );
        }

        if let (Some(preceding), Some(following)) = (block.stmts.last(), block.expr)
            && !preceding.span.from_expansion()
            && !following.span.from_expansion()
        {
            Self::check_adjacency(
                context,
                preceding.span,
                StatementGroup::of(preceding),
                following.span,
                StatementGroup::Expression,
            );
        }
    }
}
