//! DOM scratch attached to the existing native enter/leave frames.

use super::control::build::ConditionalFrame;
use super::{
    DomBinding, DomChanges, DomChild, DomChildren, DomDependency, DomExpressionFacts, DomFacts,
    DomNode, DomRejection, DomRoot, DomRootKind, DomText, DomUnsupported, PropertyRole, ValueKind,
};
use alloc::vec::Vec;
use vize_l0::{Span, id::NodeId, side_table::SideTable};
use vize_l2::{
    expr::ExprRef,
    file::FileArtifact,
    op::{Namespace, Op},
    walk::NodeRef,
};

mod binding;
mod control;
mod dependencies;
mod file;

pub(in crate::decision) struct DomBuilder<'facts, 'owner, 'arena, F> {
    expressions: &'facts F,
    file: Option<&'owner FileArtifact<'arena>>,
    frames: Vec<(NodeId, DomFrame<'owner, 'arena>)>,
    facts: DomFacts<'owner, 'arena>,
    root_count: usize,
}

struct DomFrame<'owner, 'arena> {
    node: DomNode<'owner, 'arena>,
    children: Vec<DomChild>,
    binding_names: Vec<&'arena str>,
    conditional: Option<ConditionalFrame<'owner, 'arena>>,
    normalize_class: bool,
    normalize_style: bool,
    has_text: bool,
}

impl<'facts, 'owner, 'arena, F: DomExpressionFacts> DomBuilder<'facts, 'owner, 'arena, F> {
    pub(in crate::decision) fn new(
        expressions: &'facts F,
        root_count: usize,
        file: Option<&'owner FileArtifact<'arena>>,
    ) -> Self {
        Self {
            expressions,
            file,
            frames: Vec::new(),
            root_count,
            facts: DomFacts {
                root: DomRoot {
                    kind: DomRootKind::Empty,
                    children: Vec::new(),
                },
                nodes: SideTable::new(),
                bindings: SideTable::new(),
                controls: SideTable::new(),
                dependencies: Vec::new(),
                file_expressions: SideTable::new(),
                unsupported: Vec::new(),
            },
        }
    }

    pub(in crate::decision) fn enter(
        &mut self,
        id: NodeId,
        op: &'owner Op<'arena>,
        owner_span: Option<Span>,
    ) -> Result<(), super::super::DecisionBuildError> {
        let span = NodeRef::Op(op).span();
        let branch_root = if let Some((_, frame)) = self.frames.last_mut()
            && let Some(conditional) = &mut frame.conditional
        {
            conditional.child(id, op, owner_span)?
        } else {
            false
        };
        let block_eligible = branch_root
            || (self.frames.is_empty() && self.root_count == 1 && matches!(op, Op::Element(_)));
        let conditional = self.prepare_conditional(id, op);
        let text =
            match op {
                Op::Text(_) => Some(false),
                Op::Interpolation(interpolation) => Some(
                    self.value(id, interpolation.expression)
                        .is_some_and(ValueKind::is_dynamic),
                ),
                Op::Element(element) => {
                    if element.namespace != Namespace::Html {
                        self.reject(id, span, DomUnsupported::Namespace);
                    }
                    if element.attributes.iter().any(|attribute| {
                        matches!(attribute.name, "class" | "style" | "key" | "ref")
                    }) {
                        self.reject(id, span, DomUnsupported::SpecialAttribute);
                    }
                    None
                }
                Op::Comment(_) => None,
                Op::If(_) => None,
                Op::Component(_) | Op::For(_) | Op::Slot(_) => {
                    self.reject(id, span, DomUnsupported::Operation);
                    None
                }
            };
        if !self
            .frames
            .last()
            .is_some_and(|(_, frame)| frame.conditional.is_some())
        {
            let groups =
                self.frames
                    .last_mut()
                    .map_or(&mut self.facts.root.children, |(_, frame)| {
                        frame.has_text |= text.is_some();
                        &mut frame.children
                    });
            if let Some(dynamic) = text {
                if let Some(DomChild::Text(group)) = groups.last_mut() {
                    group.nodes.push(id);
                    group.dynamic |= dynamic;
                } else {
                    let nodes = alloc::vec![id];
                    groups.push(DomChild::Text(DomText { nodes, dynamic }));
                }
            } else {
                groups.push(DomChild::Node(id));
            }
        }
        if matches!(op, Op::Interpolation(_)) {
            self.demand(DomDependency::DisplayValue);
        } else if matches!(op, Op::Comment(_)) {
            self.demand(DomDependency::CommentValue);
        }
        self.frames.push((
            id,
            DomFrame {
                node: DomNode {
                    op,
                    block_eligible,
                    children: DomChildren::Empty,
                    changes: DomChanges::default(),
                    dynamic_property_bindings: Vec::new(),
                },
                children: Vec::new(),
                binding_names: Vec::new(),
                conditional,
                normalize_class: false,
                normalize_style: false,
                has_text: false,
            },
        ));
        Ok(())
    }

    pub(in crate::decision) fn leave(
        &mut self,
        id: NodeId,
    ) -> Result<(), super::super::DecisionBuildError> {
        let Some((frame_id, mut frame)) = self.frames.pop() else {
            return Err(super::super::DecisionBuildError::InvalidTraversal { node: id });
        };
        if frame_id != id {
            return Err(super::super::DecisionBuildError::InvalidTraversal { node: id });
        }
        if let Some(conditional) = frame.conditional.take() {
            self.complete_conditional(id, NodeRef::Op(frame.node.op).span(), conditional)?;
        }
        frame.node.children = if frame.children.is_empty() {
            DomChildren::Empty
        } else if frame.children.len() == 1
            && matches!(frame.children.first(), Some(DomChild::Text(_)))
        {
            match frame.children.pop() {
                Some(DomChild::Text(text)) => {
                    frame.node.changes.text = text.dynamic;
                    DomChildren::Text(text)
                }
                _ => DomChildren::Empty,
            }
        } else {
            DomChildren::Array(frame.children)
        };
        self.complete_dependencies(
            &frame.node,
            frame.has_text,
            frame.normalize_class,
            frame.normalize_style,
        );
        if self.frames.last().is_some_and(|(_, frame)| {
            frame
                .conditional
                .as_ref()
                .is_some_and(|conditional| conditional.closes_first_branch(id))
        }) {
            self.demand(DomDependency::ConditionalPlaceholder);
        }
        self.facts.nodes.insert(id, frame.node);
        Ok(())
    }

    pub(in crate::decision) fn finish(mut self) -> DomFacts<'owner, 'arena> {
        self.facts.root.kind = match self.facts.root.children.as_slice() {
            [] => DomRootKind::Empty,
            [DomChild::Node(id)] => {
                if let Some(node) = self.facts.nodes.get_mut(*id) {
                    node.block_eligible = matches!(node.op, Op::Element(_));
                }
                DomRootKind::Direct
            }
            [_] => DomRootKind::Direct,
            children => {
                let mut non_comments = 0;
                let mut text_values = false;
                for child in children {
                    text_values |= matches!(child, DomChild::Text(_));
                    if !matches!(child, DomChild::Node(id) if self.facts.nodes.get(*id)
                        .is_some_and(|node| matches!(node.op, Op::Comment(_))))
                    {
                        non_comments += 1;
                    }
                }
                if text_values {
                    self.demand(DomDependency::TextValue);
                }
                self.demand(DomDependency::FragmentValue);
                self.demand(DomDependency::BlockBoundary);
                self.demand(DomDependency::NativeElementBlock);
                DomRootKind::Fragment {
                    single_non_comment: non_comments == 1,
                }
            }
        };
        self.facts
    }

    fn value(&mut self, id: NodeId, expression: ExprRef<'arena>) -> Option<ValueKind> {
        let span = expression.span();
        if let ExprRef::Js(js) = expression {
            if self.file.is_some() {
                return self.file_value(id, js);
            }
            if js.ast.is_literal() {
                return Some(ValueKind::LiteralConstant);
            }
            if self.expressions.is_context_only(js) {
                return Some(ValueKind::ContextDependent);
            }
        }
        self.reject(id, span, DomUnsupported::Expression);
        None
    }

    fn reject(&mut self, id: NodeId, span: Span, reason: DomUnsupported) {
        self.facts.unsupported.push(DomRejection {
            node: id,
            span,
            reason,
        });
    }

    fn demand(&mut self, role: DomDependency) {
        if !self.facts.dependencies.contains(&role) {
            self.facts.dependencies.push(role);
        }
    }
}
