//! Original root cursor and private ordinary HTML header/body construction.

use super::{RootRegion, TemplatePolicy};
use crate::file::template::TemplateWalk;
use crate::file::{Declaration, FileBuilder, FileIssueKind, ScopeId, ScriptUnitId, TemplateIssue};
use crate::lang::js::file::native::{
    NativeRouteState, NativeTemplateIssue, NativeTemplateIssueKind,
};
use vize_l0::id::NodeId;
use vize_l1::markup::{NativeChild, NativeTemplateComponent};

mod body;

#[derive(Clone, Copy)]
pub(crate) struct NativeVisibility;
impl TemplatePolicy for NativeVisibility {
    fn visible(self, _: &Declaration) -> bool {
        false
    }
}
pub struct NativeTemplateWalk<'s, 'a> {
    selected: &'s NativeTemplateComponent<'a>,
    root: RootRegion<'s, 'a, NativeVisibility>,
    state: &'s mut NativeRouteState,
    previous: TemplateWalk,
    cursor: usize,
    expected: usize,
    armed: bool,
}
impl<'s, 'a> NativeTemplateWalk<'s, 'a> {
    pub(crate) fn new(
        selected: &'s NativeTemplateComponent<'a>,
        builder: &'s mut FileBuilder<'a>,
        state: &'s mut NativeRouteState,
        setup: Option<ScriptUnitId>,
    ) -> Result<Self, NativeTemplateIssue> {
        let span = selected.component().block().span();
        let scope = if let Some(setup) = setup {
            let Some(unit) = builder.facts.units.iter().find(|unit| unit.id == setup) else {
                return Err(NativeTemplateIssue {
                    span,
                    kind: NativeTemplateIssueKind::MissingProgram,
                });
            };
            unit.scope
        } else {
            ScopeId(0)
        };
        // No neutral factory is exposed before this actual whole-root guard.
        *state = NativeRouteState::Walking;
        let previous = builder.facts.template_walk.enter(span);
        let root = RootRegion::root(
            builder.canonical.region(),
            &mut builder.facts,
            scope,
            NativeVisibility,
            builder.source,
        );
        Ok(Self {
            selected,
            root,
            state,
            previous,
            cursor: 0,
            expected: selected.children().len(),
            armed: true,
        })
    }
    /// Original immutable owner borrow is disjoint from mutable File fields.
    #[must_use]
    pub fn selected(&self) -> &'s NativeTemplateComponent<'a> {
        self.selected
    }
    pub fn child(&mut self, child: NativeChild<'_, 'a>) -> Result<NodeId, NativeTemplateIssue> {
        if !matches!(self.state, NativeRouteState::Walking) {
            return self.reject(NativeTemplateIssueKind::Interrupted);
        }
        if !core::ptr::eq(child.component(), self.selected.component())
            || child.parent_element().is_some()
            || child.ordinal() != self.cursor
        {
            return self.reject(NativeTemplateIssueKind::InvalidEvent);
        }
        let result = body::construct(child, &mut self.root);
        match result {
            Ok(node) => {
                self.cursor += 1;
                Ok(node)
            }
            Err(kind) => self.reject(kind),
        }
    }
    fn reject<T>(&mut self, kind: NativeTemplateIssueKind) -> Result<T, NativeTemplateIssue> {
        if let NativeRouteState::Refused(issue) = *self.state {
            return Err(issue);
        }
        let span = self.selected.component().block().span();
        let issue = NativeTemplateIssue { span, kind };
        *self.state = NativeRouteState::Refused(issue);
        self.root.facts.template_issues.push(TemplateIssue {
            node: None,
            span,
            kind: FileIssueKind::UnsupportedSyntax,
        });
        Err(issue)
    }
    pub fn complete(mut self) -> Result<(), NativeTemplateIssue> {
        if !matches!(self.state, NativeRouteState::Walking) {
            return self.reject(NativeTemplateIssueKind::Interrupted);
        }
        if self.cursor != self.expected {
            return self.reject(NativeTemplateIssueKind::IncompleteChildren);
        }
        self.root.facts.template_walk.complete(self.previous);
        if self.root.facts.template_walk.interruption().is_some() {
            return self.reject(NativeTemplateIssueKind::Interrupted);
        }
        *self.state = NativeRouteState::Complete;
        self.armed = false;
        Ok(())
    }
}
impl Drop for NativeTemplateWalk<'_, '_> {
    fn drop(&mut self) {
        if self.armed {
            if matches!(self.state, NativeRouteState::Walking) {
                *self.state = NativeRouteState::Interrupted;
            }
            self.root.facts.template_walk.interrupt();
        }
    }
}
