//! A host-syntax loop: a retained head, one body region and an optional
//! `@empty` region, checked as they open.

use super::{ArtifactError, Builder, Frame, Owner, RegionBuilder, check};
use crate::expr::{ExprRef, JsExpr, OpaqueExpr, OpaqueReason};
use crate::op::{ForBinding, ForOp, HostFor, Op, Region};
use crate::scope::{ScopeBinding, ScopeFacts, ScopeOrigin, ScopeTag};
use oxc_ast::ast::{BindingIdentifier, ForStatementLeft};
use oxc_span::GetSpan;
use vize_l0::{Box, Span, id::NodeId};

/// The authored head of a host loop, as the host parser produced it.
#[derive(Debug, Clone, Copy)]
pub struct HostForHead<'a> {
    /// The per-iteration declaration.
    pub left: &'a ForStatementLeft<'a>,
    /// The authored text of `left`.
    pub left_text: &'a str,
    /// The iterated collection.
    pub collection: &'a JsExpr<'a>,
    /// The `index` binding, when authored.
    pub index: Option<&'a BindingIdentifier<'a>>,
    /// The `key` expression, when authored.
    pub key: Option<ExprRef<'a>>,
    /// `for await`.
    pub r#await: bool,
}

/// The regions of one host loop: `body` exactly once, then `empty` at most once.
pub struct HostForBuilder<'s, 'a> {
    builder: &'s mut Builder<'a>,
    node: NodeId,
}

impl<'a> HostForBuilder<'_, 'a> {
    /// Build the repeated region.
    pub fn body(
        &mut self,
        children: impl FnOnce(&mut RegionBuilder<'_, 'a>, NodeId),
    ) -> Result<(), ArtifactError> {
        self.builder.host_region(self.node, false, children)
    }

    /// Build the region rendered when the collection yields no item.
    pub fn empty(
        &mut self,
        children: impl FnOnce(&mut RegionBuilder<'_, 'a>, NodeId),
    ) -> Result<(), ArtifactError> {
        self.builder.host_region(self.node, true, children)
    }
}

fn span_of(span: oxc_span::Span) -> Span {
    Span::new(span.start, span.end)
}

impl<'a> Builder<'a> {
    pub(super) fn host_for(
        &mut self,
        head: HostForHead<'a>,
        span: Span,
        regions: impl FnOnce(&mut HostForBuilder<'_, 'a>, NodeId),
    ) -> Result<NodeId, ArtifactError> {
        let id = self.prepare(span)?;
        let left = span_of(head.left.span());
        check::owned_span(self.parts.source, id, left, Some(span))?;
        if self
            .parts
            .source
            .get(left.start as usize..left.end as usize)
            != Some(head.left_text)
        {
            return Err(ArtifactError::InvalidHostFor { node: id });
        }
        let collection = ExprRef::Js(head.collection);
        check::expression(self.parts.source, id, collection, span)?;
        if let Some(key) = head.key {
            check::expression(self.parts.source, id, key, span)?;
        }
        let declared = match head.left {
            ForStatementLeft::VariableDeclaration(declaration) => declaration
                .declarations
                .iter()
                .flat_map(|declarator| declarator.id.get_binding_identifiers())
                .collect(),
            _ => alloc::vec::Vec::new(),
        };
        let mut bindings = alloc::vec::Vec::with_capacity(declared.len() + 1);
        for identifier in declared.into_iter().chain(head.index) {
            let authored = span_of(identifier.span);
            check::owned_span(self.parts.source, id, authored, Some(span))?;
            bindings.push(ScopeBinding {
                name: vize_l0::String::from(identifier.name.as_str()),
                origin: ScopeOrigin::Authored { span: authored },
            });
        }
        let tag = ScopeTag::from_index(
            u32::try_from(self.parts.scopes.len()).map_err(|_| ArtifactError::NodeLimit)?,
        );
        let value = self.allocator.alloc(OpaqueExpr {
            reason: OpaqueReason::ForValue,
            source: head.left_text,
            span: left,
        });
        self.mint();
        let _ = self.parts.scopes.insert(id, ScopeFacts { tag, bindings });
        self.frames.push(Frame {
            id,
            owner: Owner::HostFor {
                binding: ForBinding {
                    source: collection,
                    value: ExprRef::Opaque(value),
                    key: None,
                    index: None,
                },
                host: HostFor {
                    left: head.left,
                    left_text: head.left_text,
                    index: head.index,
                    key: head.key,
                    r#await: head.r#await,
                    empty: None,
                },
                body: None,
            },
            span,
            bindings: vize_l0::Vec::new_in(&self.allocator),
            children_started: false,
            ops: vize_l0::Vec::new_in(&self.allocator),
        });
        let depth = self.frames.len();
        regions(
            &mut HostForBuilder {
                builder: self,
                node: id,
            },
            id,
        );
        let finished = self.frames.len() == depth
            && matches!(
                self.frames.last(),
                Some(Frame { id: top, owner: Owner::HostFor { body: Some(_), .. }, .. }) if *top == id
            );
        if !finished {
            return Err(ArtifactError::UnfinishedOwner { node: id });
        }
        let frame = self
            .frames
            .pop()
            .ok_or(ArtifactError::UnfinishedOwner { node: id })?;
        let Owner::HostFor {
            binding,
            host,
            body: Some(region),
        } = frame.owner
        else {
            return Err(ArtifactError::UnfinishedOwner { node: id });
        };
        let host = Some(Box::new_in(host, &self.allocator));
        self.push(Op::For(Box::new_in(
            ForOp {
                binding,
                region,
                host,
                span,
            },
            &self.allocator,
        )));
        Ok(id)
    }

    fn host_region(
        &mut self,
        node: NodeId,
        empty: bool,
        children: impl FnOnce(&mut RegionBuilder<'_, 'a>, NodeId),
    ) -> Result<(), ArtifactError> {
        let frame = self
            .frames
            .last()
            .ok_or(ArtifactError::UnfinishedOwner { node })?;
        let in_order = match &frame.owner {
            Owner::HostFor { host, body, .. } if frame.id == node => {
                if empty {
                    body.is_some() && host.empty.is_none()
                } else {
                    body.is_none()
                }
            }
            _ => return Err(ArtifactError::UnfinishedOwner { node: frame.id }),
        };
        if !in_order {
            return Err(ArtifactError::InvalidHostFor { node });
        }
        let span = frame.span;
        self.frames.push(Frame {
            id: node,
            owner: Owner::HostRegion,
            span,
            bindings: vize_l0::Vec::new_in(&self.allocator),
            children_started: false,
            ops: vize_l0::Vec::new_in(&self.allocator),
        });
        let depth = self.frames.len();
        children(&mut RegionBuilder { builder: self }, node);
        if self.frames.len() != depth
            || !matches!(
                self.frames.last(),
                Some(Frame {
                    owner: Owner::HostRegion,
                    ..
                })
            )
        {
            return Err(ArtifactError::UnfinishedOwner { node });
        }
        let frame = self
            .frames
            .pop()
            .ok_or(ArtifactError::UnfinishedOwner { node })?;
        let region = Region { ops: frame.ops };
        if let Some(Frame {
            owner: Owner::HostFor { host, body, .. },
            ..
        }) = self.frames.last_mut()
        {
            if empty {
                host.empty = Some(region);
            } else {
                *body = Some(region);
            }
        }
        Ok(())
    }
}
