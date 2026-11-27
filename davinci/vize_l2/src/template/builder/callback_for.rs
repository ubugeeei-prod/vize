//! An array callback loop (`xs.map((item, index) => …)`): a `ui.for` whose
//! value and key aliases are the callback's parameters, iterated as `v-for`
//! iterates an array, and whose region is what the callback returns.

use super::{Builder, L2TemplateError, Owner, RegionBuilder, check};
use crate::expr::{ExprRef, JsExpr, OpaqueExpr, OpaqueReason};
use crate::op::ForBinding;
use crate::scope::{ScopeAnalysis, ScopeBinding, ScopeOrigin, ScopeTag};
use oxc_ast::ast::FormalParameters;
use oxc_span::GetSpan;
use vize_l0::{Span, id::NodeId};

/// The authored head of a callback loop, as the host parser produced it.
#[derive(Debug, Clone, Copy)]
pub struct CallbackForHead<'a> {
    /// The array the callback is called on.
    pub collection: &'a JsExpr<'a>,
    /// At most two plain parameters: the item, then its index.
    pub params: &'a FormalParameters<'a>,
}

impl<'a> Builder<'a> {
    pub(super) fn callback_for(
        &mut self,
        head: CallbackForHead<'a>,
        span: Span,
        children: impl FnOnce(&mut RegionBuilder<'_, 'a>, NodeId),
    ) -> Result<NodeId, L2TemplateError> {
        let id = self.prepare(span)?;
        let params = head.params;
        if params.rest.is_some()
            || params.items.len() > 2
            || params
                .items
                .iter()
                .any(|param| param.initializer.is_some() || !param.decorators.is_empty())
        {
            return Err(L2TemplateError::InvalidHostFor { node: id });
        }
        check::expression(self.parts.source, id, ExprRef::Js(head.collection), span)?;
        let mut aliases = [None, None];
        let mut bindings = alloc::vec::Vec::new();
        for (slot, param) in aliases.iter_mut().zip(&params.items) {
            let pattern = param.pattern.span();
            let pattern = Span::new(pattern.start, pattern.end);
            check::owned_span(self.parts.source, id, pattern, Some(span))?;
            let source = self
                .parts
                .source
                .get(pattern.start as usize..pattern.end as usize)
                .ok_or(L2TemplateError::InvalidSpan {
                    node: Some(id),
                    span: pattern,
                })?;
            *slot = Some(ExprRef::Opaque(self.allocator.alloc(OpaqueExpr {
                reason: OpaqueReason::ForValue,
                source,
                span: pattern,
            })));
            for identifier in param.pattern.get_binding_identifiers() {
                bindings.push(ScopeBinding {
                    name: vize_l0::String::from(identifier.name.as_str()),
                    origin: ScopeOrigin::Authored {
                        span: Span::new(identifier.span.start, identifier.span.end),
                    },
                });
            }
        }
        let [value, key] = aliases;
        // A parameterless callback binds nothing: an empty value at the list.
        let value = value.unwrap_or_else(|| {
            ExprRef::Opaque(self.allocator.alloc(OpaqueExpr {
                reason: OpaqueReason::ForValue,
                source: "",
                span: Span::new(head.collection.span.start, head.collection.span.start),
            }))
        });
        let tag = ScopeTag::from_index(
            u32::try_from(self.parts.scopes.len()).map_err(|_| L2TemplateError::NodeLimit)?,
        );
        let _ = self
            .parts
            .scopes
            .insert(id, ScopeAnalysis { tag, bindings });
        self.children(
            Owner::CallbackFor {
                binding: ForBinding {
                    source: ExprRef::Js(head.collection),
                    value,
                    key,
                    index: None,
                },
            },
            span,
            id,
            children,
        )?;
        Ok(id)
    }
}
