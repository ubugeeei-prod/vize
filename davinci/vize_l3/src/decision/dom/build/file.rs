//! Admit the actual private factory resolution, without another AST walk.

use super::{DomBuilder, DomExpressionFacts, DomUnsupported, ValueKind};
use crate::decision::dom::file::{DomFileExpression, matches_expression};
use crate::decision::dom::vue::policy::FileReads;
use crate::decision::dom::vue::{VueReadKind, VueRenderExpression, VueRenderRead};
use alloc::vec::Vec;
use vize_l0::id::NodeId;
use vize_l2::{expr::JsExpr, file::Namespace};

mod native;

impl<'owner, 'arena, F: DomExpressionFacts, R: FileReads<'owner, 'arena>>
    DomBuilder<'_, 'owner, 'arena, F, R>
{
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
        if let Some(rejected) = native::refusal(file, node, expression, R::NATIVE_SETUP) {
            self.reject(rejected.node, rejected.span, rejected.reason);
            return None;
        }
        let table = resolution.table()?;
        // These identities were resolved by the actual minting factory. Check
        // recorded declarations only, rather than rerunning scope/name lookup.
        let mut reads = Vec::new();
        for occurrence in table.occurrences() {
            let Some(binding) = resolution.binding(occurrence.binding).filter(|binding| {
                resolution.accepts(*binding)
                    && binding.declaration().is_some_and(|declaration| {
                        declaration.namespace == Namespace::Value
                            && declaration.name == occurrence.name
                    })
            }) else {
                self.reject(node, expression.span, DomUnsupported::FileBinding);
                return None;
            };
            if R::RECORD {
                let Some(kind) = self.reads.classify(occurrence, binding) else {
                    self.reject(node, expression.span, DomUnsupported::VueReadAccess);
                    return None;
                };
                reads.push(VueRenderRead {
                    occurrence,
                    binding,
                    kind,
                });
            }
        }
        // A direct identifier has no call/member descendants whose behavior
        // could vary despite immutable reads. Complete read lists alone do not
        // prove purity or constant evaluation of a compound expression.
        let constant_read = expression.ast.is_identifier_reference()
            && matches!(reads.as_slice(), [read] if read.kind == VueReadKind::SetupConst);
        let value = if expression.ast.is_literal() || constant_read {
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
        if R::RECORD {
            self.facts.vue_expressions.insert(
                node,
                VueRenderExpression {
                    resolution,
                    reads,
                    value,
                },
            );
        }
        Some(value)
    }
}
