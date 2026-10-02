//! Borrowed, native L2-node decisions beside a sealed canonical artifact.
//!
//! Every id comes from the artifact's single enter/leave walk. No flat
//! program, expression parse, legacy facts or second numbering walk is
//! needed. Placement stays inline until its independent analysis lands.

use alloc::vec::Vec;

mod levels;
use super::dom::{DomExpressionFacts, LiteralExpressions, build::DomBuilder};
use levels::Levels;

use super::{
    ControlKind, ControlRegion, DecisionTables, NativeAnalysis, NodeDecision, StaticLevel,
    policy::{BindingOwner, BindingRole, TargetPolicy},
};
use crate::placement::Placement;
use vize_l0::{Span, id::NodeId, side_table::SideTable};
use vize_l2::{
    artifact::Artifact,
    op::{BindingOp, Op},
    walk::{NodeEvent, NodeRef},
};

/// Compute conservative shared facts for every canonical L2 node.
///
/// Neutral meaning never changes with the selected target. Output meaning
/// filters compile-time cloak markers and SSR native-element events only.
/// Authored comments, components and control ops stay dynamic. Slot-content
/// grouping, hoist/cache eligibility and production selection are unfinished.
pub fn build_decisions<'owner, 'arena>(
    artifact: &'owner Artifact<'arena>,
    policy: TargetPolicy,
) -> Result<NativeAnalysis<'owner, 'arena>, DecisionBuildError> {
    build_with(artifact, policy, &LiteralExpressions)
}

/// Apply explicit native expression-binding semantics in the same DOM walk.
pub fn build_dom_decisions<'owner, 'arena>(
    artifact: &'owner Artifact<'arena>,
    expressions: &impl DomExpressionFacts,
) -> Result<NativeAnalysis<'owner, 'arena>, DecisionBuildError> {
    build_with(artifact, TargetPolicy::Dom, expressions)
}

fn build_with<'owner, 'arena>(
    artifact: &'owner Artifact<'arena>,
    policy: TargetPolicy,
    expressions: &impl DomExpressionFacts,
) -> Result<NativeAnalysis<'owner, 'arena>, DecisionBuildError> {
    let mut builder = Builder {
        policy,
        node_count: artifact.node_count(),
        frames: Vec::new(),
        nodes: SideTable::new(),
        controls: SideTable::new(),
        dom: (policy == TargetPolicy::Dom)
            .then(|| DomBuilder::new(expressions, artifact.root().ops.len())),
    };
    let mut failure = None;
    artifact
        .visit_events(&mut |event| {
            if failure.is_none() {
                failure = builder.visit(event).err();
            }
        })
        .map_err(|_| DecisionBuildError::NodeLimit)?;
    if let Some(error) = failure {
        return Err(error);
    }
    let dom = builder.dom.take().map(DomBuilder::finish);
    Ok(NativeAnalysis {
        artifact,
        tables: builder.finish()?,
        dom,
    })
}

/// A canonical walk failed to yield complete, correctly nested decisions.
/// Partial scratch tables are never returned as completed analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionBuildError {
    /// The shared L2 walk exhausted its stage-local id space.
    NodeLimit,
    /// A node's enter/leave or attached owner violated the shared walk.
    InvalidTraversal { node: NodeId },
    /// The result does not account for the sealed owner's node count.
    NodeCountMismatch { expected: u32, actual: usize },
    /// A supplied key is outside the sealed owner's dense node range.
    InvalidNode { node: NodeId },
    /// A supplied node would replace a decision instead of adding one row.
    DuplicateNode { node: NodeId },
    /// A control owner appeared more than once in the shared walk.
    DuplicateControl { node: NodeId },
}

struct Builder<'facts, 'owner, 'arena, F> {
    policy: TargetPolicy,
    node_count: u32,
    frames: Vec<Frame>,
    nodes: SideTable<NodeDecision>,
    controls: SideTable<ControlRegion>,
    dom: Option<DomBuilder<'facts, 'owner, 'arena, F>>,
}

/// One open region op, released at its matching leave event.
struct Frame {
    id: NodeId,
    levels: Levels,
    owner: Option<BindingOwner>,
    control: Option<NodeId>,
    child_control: Option<NodeId>,
    dynamic_bindings: Vec<NodeId>,
}

impl<'owner, 'arena, F: DomExpressionFacts> Builder<'_, 'owner, 'arena, F> {
    fn finish(self) -> Result<DecisionTables, DecisionBuildError> {
        if let Some(frame) = self.frames.last() {
            return Err(DecisionBuildError::InvalidTraversal { node: frame.id });
        }
        if self.nodes.len() != self.node_count as usize {
            return Err(DecisionBuildError::NodeCountMismatch {
                expected: self.node_count,
                actual: self.nodes.len(),
            });
        }
        Ok(DecisionTables {
            policy: self.policy,
            nodes: self.nodes,
            controls: self.controls,
        })
    }

    fn visit(&mut self, event: NodeEvent<'owner, 'arena>) -> Result<(), DecisionBuildError> {
        match event {
            NodeEvent::Enter {
                id,
                node,
                parent,
                owner_span,
            } => {
                if parent != self.frames.last().map(|frame| frame.id) {
                    return Err(DecisionBuildError::InvalidTraversal { node: id });
                }
                match node {
                    NodeRef::Op(op) => self.enter_op(id, op, owner_span)?,
                    NodeRef::Binding(binding) => self.binding(id, binding)?,
                }
            }
            NodeEvent::Leave {
                id,
                node: NodeRef::Op(_),
                ..
            } => self.leave_op(id)?,
            NodeEvent::Leave {
                node: NodeRef::Binding(_),
                ..
            } => {}
        }
        Ok(())
    }

    fn enter_op(
        &mut self,
        id: NodeId,
        op: &'owner Op<'arena>,
        owner_span: Option<Span>,
    ) -> Result<(), DecisionBuildError> {
        let control = self.frames.last().and_then(|frame| frame.child_control);
        let (levels, owner, kind) = match op {
            Op::Element(_) => (Levels::STATIC, Some(BindingOwner::Element), None),
            Op::Component(_) => (Levels::DYNAMIC, Some(BindingOwner::Component), None),
            Op::Text(_) => (Levels::STATIC, None, None),
            Op::Interpolation(_) => (Levels::DYNAMIC_TEXT, None, None),
            Op::Comment(_) => (Levels::DYNAMIC, None, None),
            Op::If(_) => (Levels::DYNAMIC, None, Some(ControlKind::Conditional)),
            Op::For(_) => (Levels::DYNAMIC, None, Some(ControlKind::Loop)),
            Op::Slot(_) => (
                Levels::DYNAMIC,
                Some(BindingOwner::Slot),
                Some(ControlKind::Slot),
            ),
        };
        let child_control = if let Some(kind) = kind {
            self.insert_control(
                id,
                ControlRegion {
                    kind,
                    parent: control,
                },
            )?;
            Some(id)
        } else {
            control
        };
        if let Some(dom) = &mut self.dom {
            dom.enter(id, op, owner_span)?;
        }
        self.frames.push(Frame {
            id,
            levels,
            owner,
            control,
            child_control,
            dynamic_bindings: Vec::new(),
        });
        Ok(())
    }

    fn binding(
        &mut self,
        id: NodeId,
        binding: &'owner BindingOp<'arena>,
    ) -> Result<(), DecisionBuildError> {
        let frame = self
            .frames
            .last_mut()
            .ok_or(DecisionBuildError::InvalidTraversal { node: id })?;
        let owner = frame
            .owner
            .ok_or(DecisionBuildError::InvalidTraversal { node: id })?;
        if let Some(dom) = &mut self.dom {
            dom.binding(id, binding);
        }
        let role = match binding {
            BindingOp::On(_) => BindingRole::Event,
            BindingOp::VueCloak(_) => BindingRole::Cloak,
            BindingOp::Bind(_)
            | BindingOp::Model(_)
            | BindingOp::SlotContent(_)
            | BindingOp::VueDirective(_)
            | BindingOp::VueCssBind(_)
            | BindingOp::VueSync(_)
            | BindingOp::VueSlotScope(_)
            | BindingOp::VueOnce(_)
            | BindingOp::VueMemo(_)
            | BindingOp::VueShow(_)
            | BindingOp::VueHtml(_)
            | BindingOp::VueText(_) => BindingRole::Other,
        };
        let levels = Levels {
            neutral: if role == BindingRole::Cloak {
                StaticLevel::Static
            } else {
                StaticLevel::Dynamic
            },
            output: if self.policy.binding_is_dynamic(owner, role) {
                StaticLevel::Dynamic
            } else {
                StaticLevel::Static
            },
        };
        if levels.output == StaticLevel::Dynamic {
            frame.dynamic_bindings.push(id);
        }
        frame.levels = frame.levels.join(levels);
        let control = frame.child_control;
        self.insert_node(
            id,
            NodeDecision {
                static_level: levels.neutral,
                output_level: levels.output,
                dynamic_bindings: Vec::new(),
                placement: Placement::Inline,
                control,
            },
        )?;
        Ok(())
    }

    fn leave_op(&mut self, id: NodeId) -> Result<(), DecisionBuildError> {
        let frame = self
            .frames
            .pop()
            .ok_or(DecisionBuildError::InvalidTraversal { node: id })?;
        if frame.id != id {
            return Err(DecisionBuildError::InvalidTraversal { node: id });
        }
        if let Some(dom) = &mut self.dom {
            dom.leave(id)?;
        }
        self.insert_node(
            id,
            NodeDecision {
                static_level: frame.levels.neutral,
                output_level: frame.levels.output,
                dynamic_bindings: frame.dynamic_bindings,
                placement: Placement::Inline,
                control: frame.control,
            },
        )?;
        if let Some(parent) = self.frames.last_mut() {
            // Direct interpolation yields DynamicText for its element;
            // text under a nested element makes the ancestor Dynamic.
            let levels = if frame.owner == Some(BindingOwner::Element) {
                frame.levels.nested()
            } else {
                frame.levels
            };
            parent.levels = parent.levels.join(levels);
        }
        Ok(())
    }

    fn check_key(&self, id: NodeId) -> Result<(), DecisionBuildError> {
        if id.index() >= self.node_count {
            return Err(DecisionBuildError::InvalidNode { node: id });
        }
        Ok(())
    }

    fn insert_node(&mut self, id: NodeId, row: NodeDecision) -> Result<(), DecisionBuildError> {
        self.check_key(id)?;
        if self.nodes.insert(id, row).is_some() {
            return Err(DecisionBuildError::DuplicateNode { node: id });
        }
        Ok(())
    }

    fn insert_control(&mut self, id: NodeId, row: ControlRegion) -> Result<(), DecisionBuildError> {
        self.check_key(id)?;
        if self.controls.insert(id, row).is_some() {
            return Err(DecisionBuildError::DuplicateControl { node: id });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
