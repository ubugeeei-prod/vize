//! Borrowed, native L2-node decisions beside a sealed canonical artifact.
//!
//! Every id comes from the artifact's single enter/leave walk. No flat
//! program, expression parse, legacy facts or second numbering walk is
//! needed. Placement stays inline until its independent analysis lands.

use alloc::vec::Vec;

use vize_l0::{id::NodeId, side_table::SideTable};
use vize_l2::{
    artifact::Artifact,
    op::{BindingOp, Op},
    walk::{NodeEvent, NodeRef},
};
use vize_l3::{
    decision::{
        ControlKind, ControlRegion, DecisionTables, NodeDecision, StaticLevel,
        policy::{BindingOwner, BindingRole, TargetPolicy},
    },
    placement::Placement,
};

/// Compute conservative shared facts for every canonical L2 node.
///
/// Neutral meaning never changes with the selected target. Output meaning
/// filters compile-time cloak markers and SSR native-element events only.
/// Authored comments, components and control ops stay dynamic. Slot-content
/// grouping, hoist/cache eligibility and production selection are unfinished.
pub fn build_decisions(
    artifact: &Artifact<'_>,
    policy: TargetPolicy,
) -> Result<DecisionTables, DecisionBuildError> {
    let mut builder = Builder {
        policy,
        frames: Vec::new(),
        nodes: SideTable::new(),
        controls: SideTable::new(),
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
    if let Some(frame) = builder.frames.last() {
        return Err(DecisionBuildError::InvalidTraversal { node: frame.id });
    }
    if builder.nodes.len() != artifact.node_count() as usize {
        return Err(DecisionBuildError::NodeCountMismatch {
            expected: artifact.node_count(),
            actual: builder.nodes.len(),
        });
    }
    Ok(DecisionTables {
        policy,
        nodes: builder.nodes,
        controls: builder.controls,
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
}

struct Builder {
    policy: TargetPolicy,
    frames: Vec<Frame>,
    nodes: SideTable<NodeDecision>,
    controls: SideTable<ControlRegion>,
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

impl Builder {
    fn visit(&mut self, event: NodeEvent<'_, '_>) -> Result<(), DecisionBuildError> {
        match event {
            NodeEvent::Enter {
                id, node, parent, ..
            } => {
                if parent != self.frames.last().map(|frame| frame.id) {
                    return Err(DecisionBuildError::InvalidTraversal { node: id });
                }
                match node {
                    NodeRef::Op(op) => self.enter_op(id, op),
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

    fn enter_op(&mut self, id: NodeId, op: &Op<'_>) {
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
            self.controls.insert(
                id,
                ControlRegion {
                    kind,
                    parent: control,
                },
            );
            Some(id)
        } else {
            control
        };
        self.frames.push(Frame {
            id,
            levels,
            owner,
            control,
            child_control,
            dynamic_bindings: Vec::new(),
        });
    }

    fn binding(&mut self, id: NodeId, binding: &BindingOp<'_>) -> Result<(), DecisionBuildError> {
        let frame = self
            .frames
            .last_mut()
            .ok_or(DecisionBuildError::InvalidTraversal { node: id })?;
        let owner = frame
            .owner
            .ok_or(DecisionBuildError::InvalidTraversal { node: id })?;
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
        self.nodes.insert(
            id,
            NodeDecision {
                static_level: levels.neutral,
                output_level: levels.output,
                dynamic_bindings: Vec::new(),
                placement: Placement::Inline,
                control: frame.child_control,
            },
        );
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
        self.nodes.insert(
            id,
            NodeDecision {
                static_level: frame.levels.neutral,
                output_level: frame.levels.output,
                dynamic_bindings: frame.dynamic_bindings,
                placement: Placement::Inline,
                control: frame.control,
            },
        );
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
}

#[derive(Clone, Copy)]
struct Levels {
    neutral: StaticLevel,
    output: StaticLevel,
}

impl Levels {
    const STATIC: Self = Self {
        neutral: StaticLevel::Static,
        output: StaticLevel::Static,
    };
    const DYNAMIC: Self = Self {
        neutral: StaticLevel::Dynamic,
        output: StaticLevel::Dynamic,
    };
    const DYNAMIC_TEXT: Self = Self {
        neutral: StaticLevel::DynamicText,
        output: StaticLevel::DynamicText,
    };

    fn join(self, other: Self) -> Self {
        Self {
            neutral: join(self.neutral, other.neutral),
            output: join(self.output, other.output),
        }
    }

    fn nested(self) -> Self {
        Self {
            neutral: nested(self.neutral),
            output: nested(self.output),
        }
    }
}

fn join(left: StaticLevel, right: StaticLevel) -> StaticLevel {
    match (left, right) {
        (StaticLevel::Dynamic, _) | (_, StaticLevel::Dynamic) => StaticLevel::Dynamic,
        (StaticLevel::DynamicText, _) | (_, StaticLevel::DynamicText) => StaticLevel::DynamicText,
        (StaticLevel::Static, StaticLevel::Static) => StaticLevel::Static,
    }
}

fn nested(level: StaticLevel) -> StaticLevel {
    match level {
        StaticLevel::DynamicText => StaticLevel::Dynamic,
        StaticLevel::Static | StaticLevel::Dynamic => level,
    }
}
