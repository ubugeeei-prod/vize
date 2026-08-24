//! The indentation frame stack of the folio parser: which containers
//! are open, which grouped position each is in, and how a finished frame
//! closes back into its owner. Split from [`parse`](super) when the slot
//! outlet gained its props surface (a third phased frame) pushed the
//! parser past the source budget.

use vize_davinci::folio::FolioError;
use vize_s0::cstr;

use super::super::owned::{
    FolioAttribute, FolioBinding, FolioBranch, FolioComponent, FolioElement, FolioFor, FolioIf,
    FolioModel, FolioOp, FolioSlot,
};
use super::Parser;
use super::line::err;

/// Which grouped position an open element/component frame is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Phase {
    Attrs,
    Bindings,
    Children,
}

/// One open container on the indentation stack.
#[derive(Debug)]
pub(super) enum Frame {
    Element(FolioElement, Phase),
    Component(FolioComponent, Phase),
    Model(FolioModel),
    If(FolioIf),
    Branch(FolioBranch),
    For(FolioFor),
    Slot(FolioSlot, Phase),
}

impl Parser {
    pub(super) fn push_attr(
        &mut self,
        attribute: FolioAttribute,
        line_no: usize,
    ) -> Result<(), FolioError> {
        match self.stack.last_mut() {
            Some(Frame::Element(element, phase)) => match phase {
                Phase::Attrs => {
                    element.attributes.push(attribute);
                    Ok(())
                }
                Phase::Bindings => Err(err(line_no, cstr!("attribute after a binding"))),
                Phase::Children => Err(err(line_no, cstr!("attribute after a child"))),
            },
            Some(Frame::Component(component, phase)) => match phase {
                Phase::Attrs => {
                    component.attributes.push(attribute);
                    Ok(())
                }
                Phase::Bindings => Err(err(line_no, cstr!("attribute after a binding"))),
                Phase::Children => Err(err(line_no, cstr!("attribute after a child"))),
            },
            Some(Frame::Slot(slot, phase)) => match phase {
                Phase::Attrs => {
                    slot.attributes.push(attribute);
                    Ok(())
                }
                Phase::Bindings => Err(err(line_no, cstr!("attribute after a binding"))),
                Phase::Children => Err(err(line_no, cstr!("attribute after a child"))),
            },
            Some(Frame::Model(model)) => {
                model.attributes.push(attribute);
                Ok(())
            }
            Some(Frame::If(_)) => Err(err(line_no, cstr!("expected `branch` under `ui.if`"))),
            None | Some(Frame::Branch(_) | Frame::For(_)) => {
                Err(err(line_no, cstr!("`attr` outside an element")))
            }
        }
    }

    /// Attach a completed leaf binding (`ui.slot-content` /
    /// `vue.directive`) to the owner `guard_binding` admitted.
    pub(super) fn push_leaf_binding(&mut self, binding: FolioBinding) {
        match self.stack.last_mut() {
            Some(Frame::Element(element, _)) => element.bindings.push(binding),
            Some(Frame::Component(component, _)) => component.bindings.push(binding),
            Some(Frame::Slot(slot, _)) => slot.bindings.push(binding),
            _ => unreachable!("guard_binding admitted the owner"),
        }
    }

    /// A binding line (`ui.bind` / `ui.on` / `ui.model` /
    /// `ui.slot-content` / `vue.directive`) needs an open element,
    /// component, or slot outlet whose children have not started.
    pub(super) fn guard_binding(&mut self, line_no: usize) -> Result<(), FolioError> {
        match self.stack.last_mut() {
            Some(Frame::Element(_, phase) | Frame::Component(_, phase) | Frame::Slot(_, phase)) => {
                if *phase == Phase::Children {
                    return Err(err(line_no, cstr!("binding after a child")));
                }
                *phase = Phase::Bindings;
                Ok(())
            }
            Some(Frame::Model(_)) => Err(err(line_no, cstr!("expected `attr` under `ui.model`"))),
            Some(Frame::If(_)) => Err(err(line_no, cstr!("expected `branch` under `ui.if`"))),
            None | Some(Frame::Branch(_) | Frame::For(_)) => {
                Err(err(line_no, cstr!("binding outside an element")))
            }
        }
    }

    /// A region-op line needs a child position: the root, an element or
    /// component body, a branch, a `ui.for` region, or a slot fallback.
    pub(super) fn guard_child(&self, line_no: usize) -> Result<(), FolioError> {
        match self.stack.last() {
            Some(Frame::If(_)) => Err(err(line_no, cstr!("expected `branch` under `ui.if`"))),
            Some(Frame::Model(_)) => Err(err(line_no, cstr!("expected `attr` under `ui.model`"))),
            None
            | Some(
                Frame::Element(..)
                | Frame::Component(..)
                | Frame::Branch(_)
                | Frame::For(_)
                | Frame::Slot(..),
            ) => Ok(()),
        }
    }

    /// Attach a finished op to the innermost open child position.
    pub(super) fn attach_op(&mut self, op: FolioOp) {
        match self.stack.last_mut() {
            None => self.root.push(op),
            Some(Frame::Element(element, phase)) => {
                *phase = Phase::Children;
                element.children.push(op);
            }
            Some(Frame::Component(component, phase)) => {
                *phase = Phase::Children;
                component.children.push(op);
            }
            Some(Frame::Branch(branch)) => branch.ops.push(op),
            Some(Frame::For(for_op)) => for_op.ops.push(op),
            Some(Frame::Slot(slot, phase)) => {
                *phase = Phase::Children;
                slot.fallback.push(op);
            }
            Some(Frame::If(_) | Frame::Model(_)) => {
                unreachable!("guard_child rejects these parents")
            }
        }
    }

    /// Close the innermost frame back into its owner.
    pub(super) fn close_top(&mut self) {
        let Some(frame) = self.stack.pop() else {
            return;
        };
        match frame {
            Frame::Element(element, _) => self.attach_op(FolioOp::Element(element)),
            Frame::Component(component, _) => self.attach_op(FolioOp::Component(component)),
            Frame::If(if_op) => self.attach_op(FolioOp::If(if_op)),
            Frame::For(for_op) => self.attach_op(FolioOp::For(for_op)),
            Frame::Slot(slot, _) => self.attach_op(FolioOp::Slot(slot)),
            Frame::Branch(branch) => match self.stack.last_mut() {
                Some(Frame::If(if_op)) => if_op.branches.push(branch),
                _ => unreachable!("branch frames only open under ui.if"),
            },
            Frame::Model(model) => match self.stack.last_mut() {
                Some(Frame::Element(element, _)) => {
                    element.bindings.push(FolioBinding::Model(model));
                }
                Some(Frame::Component(component, _)) => {
                    component.bindings.push(FolioBinding::Model(model));
                }
                Some(Frame::Slot(slot, _)) => {
                    slot.bindings.push(FolioBinding::Model(model));
                }
                _ => unreachable!("model frames only open under an element-like owner"),
            },
        }
    }
}
