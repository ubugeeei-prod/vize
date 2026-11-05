use super::{DESTRUCTURE, GET_SCRIPT_SETUP, GET_SETUP, RULE};
use crate::diagnostic::LintDiagnostic;
use crate::rules::script::ScriptLintResult;
use oxc_ast::ast::{ChainElement, Expression};
use oxc_span::Span;
use vize_l0::{CompactString, FxHashMap};

mod bind;
mod verify;
mod visit;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Props(bool),
    Prop(bool),
    Other,
    DefineProps,
    WithDefaults,
    VueFactory,
}

pub(super) struct Binding {
    kind: Kind,
    scope: u32,
}

pub(super) struct ObjectFrame {
    pub(super) is_vue: bool,
    pub(super) depth: u32,
    pub(super) key: Option<CompactString>,
}

pub(super) struct Visitor<'a> {
    pub(super) offset: usize,
    pub(super) result: &'a mut ScriptLintResult,
    pub(super) fn_depth: u32,
    pub(super) scope_id: u32,
    pub(super) force_vue: bool,
    pub(super) frames: Vec<FxHashMap<CompactString, Binding>>,
    pub(super) objects: Vec<ObjectFrame>,
}

impl Visitor<'_> {
    pub(super) fn insert(&mut self, name: &str, kind: Kind) {
        if let Some(frame) = self.frames.last_mut() {
            frame.insert(
                CompactString::from(name),
                Binding {
                    kind,
                    scope: self.scope_id,
                },
            );
        }
    }

    fn lookup(&self, name: &str) -> Option<&Binding> {
        self.frames.iter().rev().find_map(|frame| frame.get(name))
    }

    pub(super) fn peel<'b>(expr: &'b Expression<'b>) -> &'b Expression<'b> {
        match expr {
            Expression::ParenthesizedExpression(inner) => Self::peel(&inner.expression),
            Expression::TSAsExpression(inner) => Self::peel(&inner.expression),
            Expression::TSSatisfiesExpression(inner) => Self::peel(&inner.expression),
            Expression::TSNonNullExpression(inner) => Self::peel(&inner.expression),
            other => other,
        }
    }

    /// Root identifier of a member chain, and whether a property was followed.
    /// A call stops the walk so `p?.foo?.()` stays a call for the call visitor.
    pub(super) fn member_root<'b>(expr: &'b Expression<'b>) -> (&'b Expression<'b>, bool) {
        let mut current = Self::peel(expr);
        let mut via_member = false;
        loop {
            current = match current {
                Expression::StaticMemberExpression(member) => {
                    via_member = true;
                    Self::peel(&member.object)
                }
                Expression::ComputedMemberExpression(member) => {
                    via_member = true;
                    Self::peel(&member.object)
                }
                Expression::PrivateFieldExpression(field) => {
                    via_member = true;
                    Self::peel(&field.object)
                }
                Expression::ChainExpression(chain) => {
                    return Self::chain_root(&chain.expression, via_member);
                }
                other => return (other, via_member),
            };
        }
    }

    fn chain_root<'b>(
        element: &'b ChainElement<'b>,
        via_member: bool,
    ) -> (&'b Expression<'b>, bool) {
        match element {
            ChainElement::CallExpression(call) => (&call.callee, via_member),
            ChainElement::TSNonNullExpression(inner) => {
                let (root, nested) = Self::member_root(&inner.expression);
                (root, via_member || nested)
            }
            ChainElement::StaticMemberExpression(member) => {
                let (root, _) = Self::member_root(&member.object);
                (root, true)
            }
            ChainElement::ComputedMemberExpression(member) => {
                let (root, _) = Self::member_root(&member.object);
                (root, true)
            }
            ChainElement::PrivateFieldExpression(field) => {
                let (root, _) = Self::member_root(&field.object);
                (root, true)
            }
        }
    }

    pub(super) fn prop_scope(&self, name: &str, via_member: bool) -> Option<bool> {
        let binding = self.lookup(name)?;
        if binding.scope != self.scope_id {
            return None;
        }
        match binding.kind {
            Kind::Prop(script_setup) => Some(script_setup),
            Kind::Props(script_setup) if via_member => Some(script_setup),
            _ => None,
        }
    }

    pub(super) fn container_scope(&self, name: &str) -> Option<bool> {
        let binding = self.lookup(name)?;
        if binding.scope != self.scope_id {
            return None;
        }
        match binding.kind {
            Kind::Props(script_setup) => Some(script_setup),
            _ => None,
        }
    }

    pub(super) fn report_span(&mut self, span: Span, message: &str) {
        self.result.add_diagnostic(LintDiagnostic::warn(
            RULE,
            message,
            self.offset as u32 + span.start,
            self.offset as u32 + span.end,
        ));
    }

    pub(super) fn report_get(&mut self, span: Span, script_setup: bool) {
        self.report_span(
            span,
            if script_setup {
                GET_SCRIPT_SETUP
            } else {
                GET_SETUP
            },
        );
    }

    pub(super) fn report_destructure(&mut self, span: Span) {
        self.report_span(span, DESTRUCTURE);
    }
}
