//! Original root cursor and private ordinary HTML header/body construction.

use super::RootRegion;
use crate::artifact::ComponentFactory;
use crate::file::template::TemplateWalk;
use crate::file::{FileBuilder, FileIssueKind, ScopeId, ScriptUnitId, TemplateIssue};
use crate::lang::js::file::native::{
    NativeRouteState, NativeTemplateIssue, NativeTemplateIssueKind,
};
use vize_l0::id::NodeId;
use vize_l1::markup::{NativeChild, NativeRootText, NativeTemplateComponent};

mod attribute_value;
mod body;
mod for_body;
mod for_head;
#[cfg(test)]
mod for_tests;
mod handler;
mod interpolation;
mod visibility;
pub(crate) use visibility::NativeVisibility;
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
        let policy = NativeVisibility::checked(selected, &builder.facts, setup);
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
            policy,
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
        if !matches!(self.state, NativeRouteState::Walking)
            || self.root.facts.template_walk.interruption().is_some()
        {
            return self.reject(NativeTemplateIssueKind::Interrupted);
        }
        if !core::ptr::eq(child.component(), self.selected.component())
            || child.parent_element().is_some()
            || child.ordinal() != self.cursor
        {
            return self.reject(NativeTemplateIssueKind::InvalidEvent);
        }
        let selected = self.selected;
        let span = selected.component().block().span();
        // Includes original header observation and handler parsing/resolution,
        // before element_body's nested guard. A caught unwind is sticky even
        // when the caller keeps this walk and retries the same original child.
        let result = self
            .root
            .with_walk(span, |root| body::construct(selected, child, root));
        match result {
            Ok(node) => {
                self.cursor += 1;
                Ok(node)
            }
            Err(kind) => self.reject(kind),
        }
    }
    /// Consume the selected original root cursor through its sealed L1 text
    /// receipt. Omitted source text advances custody without creating an op.
    pub fn root_text(
        &mut self,
        receipt: &NativeRootText<'a>,
    ) -> Result<Option<NodeId>, NativeTemplateIssue> {
        if !matches!(self.state, NativeRouteState::Walking)
            || self.root.facts.template_walk.interruption().is_some()
        {
            return self.reject(NativeTemplateIssueKind::Interrupted);
        }
        let Some(view) = receipt.admitted_for_root_at(self.selected, self.cursor) else {
            return self.reject(NativeTemplateIssueKind::InvalidEvent);
        };
        let result = match view.receipt().content() {
            Some(content) => self
                .root
                .text(content, view.receipt().span())
                .map(Some)
                .map_err(NativeTemplateIssueKind::Artifact),
            None => Ok(None),
        };
        match result {
            Ok(node) => {
                self.cursor += 1;
                Ok(node)
            }
            Err(kind) => self.reject(kind),
        }
    }
    /// Consume a whole input at the actual original root child event.
    /// The private File stores its original observations before resolution;
    /// neither a raw expression nor a nested child can advance this cursor.
    pub fn root_interpolation(
        &mut self,
        child: NativeChild<'_, 'a>,
        input: crate::lang::js::NativeInterpolationInput<'a>,
    ) -> Result<NodeId, NativeTemplateIssue> {
        let check = if !matches!(self.state, NativeRouteState::Walking)
            || self.root.facts.template_walk.interruption().is_some()
        {
            Err(NativeTemplateIssueKind::Interrupted)
        } else if !core::ptr::eq(child.component(), self.selected.component())
            || child.parent_element().is_some()
            || child.ordinal() != self.cursor
        {
            Err(NativeTemplateIssueKind::InvalidEvent)
        } else {
            Ok(())
        };
        let selected = self.selected;
        let span = selected.component().block().span();
        let result = self.root.with_walk(span, |region| {
            interpolation::construct(selected, child, input, region, check)
        });
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
        let span = match kind {
            NativeTemplateIssueKind::Handler { span, .. }
            | NativeTemplateIssueKind::For { span, .. }
            | NativeTemplateIssueKind::InterpolationPreparation { span, .. } => span,
            _ => self.selected.component().block().span(),
        };
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
        if !matches!(self.state, NativeRouteState::Walking)
            || self.root.facts.template_walk.interruption().is_some()
        {
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

#[cfg(test)]
mod interruption;
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
