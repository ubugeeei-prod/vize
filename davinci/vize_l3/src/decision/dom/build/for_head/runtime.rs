//! Mutable root For eligibility on the existing authentic rows and frames.

use super::super::{DomBuilder, DomChild, DomChildren, DomExpressionFacts, DomFrame};
use crate::decision::dom::{
    DomDependency, DomFileForHead, DomUnsupported,
    vue::{VueReadKind, policy::FileReads},
};
use oxc_ast::ast::Expression;
use vize_l0::id::NodeId;
use vize_l2::{op::Op, walk::NodeRef};

impl<'owner, 'arena, F: DomExpressionFacts, R: FileReads<'owner, 'arena>>
    DomBuilder<'_, 'owner, 'arena, F, R>
{
    pub(super) fn prepare_for_runtime(
        &mut self,
        node: NodeId,
        row: &DomFileForHead<'owner, 'arena>,
    ) -> bool {
        if !R::ORIGINAL_FOR
            || row.collection_read().map(|read| read.kind) != Some(VueReadKind::SetupLet)
        {
            // Preserve old generic/constant Operation refusals and positive
            // collection custody; the stable constant branch is not implemented.
            return false;
        }
        let span = row.original().span;
        if !self.frames.is_empty() || self.root_count != 1 || row.resolution().key().is_some() {
            self.reject(node, span, DomUnsupported::ForShape);
            return false;
        }
        let occurrence = row.resolution().collection_occurrence();
        let Expression::Identifier(identifier) = row.resolution().input().collection() else {
            self.reject(node, span, DomUnsupported::ForCollectionRuntime);
            return false;
        };
        let authored = row.resolution().collection_authored_span();
        let source = row.collection().file().artifact().source();
        // The policy holds the actual selected setup and checks same-File/unit/
        // direct-declaration membership. PrimitiveLiteral is not a numeric type.
        if self.reads.classify(occurrence, row.collection()) != Some(VueReadKind::SetupLet)
            || identifier.name.as_str() != occurrence.name
            || source.get(authored.start as usize..authored.end as usize) != Some(occurrence.name)
        {
            self.reject(node, authored, DomUnsupported::ForCollectionRuntime);
            return false;
        }
        let alias = row.resolution().value_declaration();
        let authored = alias.authored_span();
        if source.get(authored.start as usize..authored.end as usize) != Some(alias.fact().name()) {
            self.reject(node, authored, DomUnsupported::ForShape);
            return false;
        }
        // Pinned transform order precedes the existing carrier's closure.
        self.demand(DomDependency::CollectionIteration);
        self.demand(DomDependency::FragmentValue);
        self.demand(DomDependency::BlockBoundary);
        self.demand(DomDependency::NativeElementBlock);
        true
    }

    pub(in crate::decision::dom::build) fn in_original_for_body(&self) -> bool {
        R::ORIGINAL_FOR
            && self.frames.first().is_some_and(|(_, frame)| {
                frame.node.block_eligible && matches!(frame.node.op, Op::OriginalFor(_))
            })
    }

    pub(in crate::decision::dom::build) fn prepare_for_body(
        &mut self,
        node: NodeId,
        op: &Op<'arena>,
    ) -> bool {
        if !self.in_original_for_body() {
            return false;
        }
        let carrier = self
            .frames
            .last()
            .is_some_and(|(_, frame)| matches!(frame.node.op, Op::OriginalFor(_)));
        match op {
            Op::Element(element) if carrier && element.attributes.is_empty() => true,
            Op::Text(_) if !carrier => false,
            _ => {
                self.reject(node, NodeRef::Op(op).span(), DomUnsupported::ForBody);
                false
            }
        }
    }

    pub(in crate::decision::dom::build) fn complete_for_body(
        &mut self,
        node: NodeId,
        frame: &mut DomFrame<'owner, 'arena>,
    ) {
        if !frame.node.block_eligible || !matches!(frame.node.op, Op::OriginalFor(_)) {
            return;
        }
        // The direct Element already closed on this same walk; this is one
        // table lookup, never another body or child traversal.
        let valid = match frame.children.as_slice() {
            [DomChild::Node(body)] => self.facts.node(*body).is_some_and(|fact| {
                fact.block_eligible
                    && matches!(fact.op(), Op::Element(element) if element.attributes.is_empty())
                    && matches!(
                        &fact.children,
                        DomChildren::Empty
                            | DomChildren::Text(crate::decision::dom::DomText {
                                dynamic: false,
                                ..
                            })
                    )
            }),
            _ => false,
        };
        if !valid {
            frame.node.block_eligible = false;
            self.reject(
                node,
                NodeRef::Op(frame.node.op).span(),
                DomUnsupported::ForBody,
            );
        }
    }
}
