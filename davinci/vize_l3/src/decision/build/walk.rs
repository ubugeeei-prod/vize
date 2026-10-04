//! The unchanged canonical Enter/Binding/Leave visitor.

use super::{Builder, Frame, Levels};
use crate::decision::{
    ControlKind, ControlRegion, DecisionBuildError, DecisionTables, NodeDecision, StaticLevel,
    dom::DomExpressionFacts,
    dom::vue::policy::FileReads,
    policy::{BindingOwner, BindingRole},
};
use crate::placement::Placement;
use alloc::vec::Vec;
use vize_l0::{Span, id::NodeId};
use vize_l2::{
    op::{BindingOp, Op},
    walk::{NodeEvent, NodeRef},
};

impl<'owner, 'arena, F: DomExpressionFacts, R: FileReads<'owner, 'arena>>
    Builder<'_, 'owner, 'arena, F, R>
{
    pub(super) fn finish(self) -> Result<DecisionTables, DecisionBuildError> {
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

    pub(super) fn visit(
        &mut self,
        event: NodeEvent<'owner, 'arena>,
    ) -> Result<(), DecisionBuildError> {
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
            Op::For(_) | Op::OriginalFor(_) => (Levels::DYNAMIC, None, Some(ControlKind::Loop)),
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
        if let Some(ssr) = &mut self.ssr {
            ssr.enter(id, op, self.frames.is_empty());
        }
        if let Some(vapor) = &mut self.vapor {
            vapor.enter(id, op, self.frames.is_empty());
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
        if let Some(ssr) = &mut self.ssr {
            ssr.binding(id, binding, owner);
        }
        if let Some(vapor) = &mut self.vapor {
            vapor.binding(id, binding);
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
        if let Some(ssr) = &mut self.ssr {
            ssr.leave(id);
        }
        if let Some(vapor) = &mut self.vapor {
            vapor.leave(id);
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

    pub(super) fn insert_node(
        &mut self,
        id: NodeId,
        row: NodeDecision,
    ) -> Result<(), DecisionBuildError> {
        self.check_key(id)?;
        if self.nodes.insert(id, row).is_some() {
            return Err(DecisionBuildError::DuplicateNode { node: id });
        }
        Ok(())
    }

    pub(super) fn insert_control(
        &mut self,
        id: NodeId,
        row: ControlRegion,
    ) -> Result<(), DecisionBuildError> {
        self.check_key(id)?;
        if self.controls.insert(id, row).is_some() {
            return Err(DecisionBuildError::DuplicateControl { node: id });
        }
        Ok(())
    }
}
