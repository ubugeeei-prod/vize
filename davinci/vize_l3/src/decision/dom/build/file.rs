//! Admit the actual private factory resolution, without another AST walk.

use super::{DomBuilder, DomExpressionFacts, DomUnsupported, ValueKind};
use crate::decision::dom::file::{DomFileExpression, matches_expression};
use vize_l0::id::NodeId;
use vize_l2::{expr::JsExpr, file::Namespace};

impl<'owner, 'arena, F: DomExpressionFacts> DomBuilder<'_, 'owner, 'arena, F> {
    pub(super) fn file_value(
        &mut self,
        node: NodeId,
        expression: &JsExpr<'arena>,
    ) -> Option<ValueKind> {
        let file = self.file?;
        let Some(resolution) = file
            .expression(node)
            .filter(|row| matches_expression(*row, expression))
        else {
            self.reject(node, expression.span, DomUnsupported::FileExpression);
            return None;
        };
        if !resolution.scope().is_some_and(|scope| {
            file.scopes()
                .get(scope.index() as usize)
                .is_some_and(|record| record.id == scope)
        }) {
            self.reject(node, expression.span, DomUnsupported::FileScope);
            return None;
        };
        let table = resolution.table()?;
        // These identities were resolved by the actual minting factory. Check
        // recorded declarations only, rather than rerunning scope/name lookup.
        if !table.occurrences().iter().all(|occurrence| {
            resolution
                .binding(occurrence.binding)
                .is_some_and(|binding| {
                    resolution.accepts(binding)
                        && binding.declaration().is_some_and(|declaration| {
                            declaration.namespace == Namespace::Value
                                && declaration.name == occurrence.name
                        })
                })
        }) {
            self.reject(node, expression.span, DomUnsupported::FileBinding);
            return None;
        }
        let value = if expression.ast.is_literal() {
            ValueKind::LiteralConstant
        } else if !table.occurrences().is_empty() {
            ValueKind::FileDependent
        } else {
            self.reject(node, expression.span, DomUnsupported::Expression);
            return None;
        };
        self.facts
            .file_expressions
            .insert(node, DomFileExpression { resolution });
        Some(value)
    }
}
