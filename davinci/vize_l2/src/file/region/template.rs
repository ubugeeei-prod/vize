//! Original root cursor and private ordinary HTML header/body construction.

use super::RootRegion;
use crate::artifact::ComponentFactory;
use crate::file::template::TemplateWalk;
use crate::file::{FileBuilder, FileIssueKind, ScopeId, ScriptUnitId, TemplateIssue};
use crate::lang::js::file::template::{
    TemplateRouteState, TemplateRouteIssue, TemplateRouteIssueKind,
};
use vize_l0::id::NodeId;
use vize_l1::markup::{ChildView, ChildrenView, NativeTemplateComponent, RootText, SurfaceChild};

mod attribute_value;
mod body;
mod conditional;
mod for_body;
mod for_head;
#[cfg(test)]
mod for_tests;
mod handler;
mod interpolation;
mod model;
mod pair;
mod text_value;
mod visibility;
pub(crate) use visibility::TemplateVisibility;
pub struct FileTemplateWalk<'s, 'a> {
    selected: &'s NativeTemplateComponent<'a>,
    root: RootRegion<'s, 'a, TemplateVisibility>,
    state: &'s mut TemplateRouteState,
    previous: TemplateWalk,
    cursor: usize,
    expected: usize,
    armed: bool,
}
impl<'s, 'a> FileTemplateWalk<'s, 'a> {
    pub(crate) fn new(
        selected: &'s NativeTemplateComponent<'a>,
        builder: &'s mut FileBuilder<'a>,
        state: &'s mut TemplateRouteState,
        setup: Option<ScriptUnitId>,
    ) -> Result<Self, TemplateRouteIssue> {
        let span = selected.component().block().span();
        let policy = TemplateVisibility::checked(selected, &builder.facts, setup);
        let scope = if let Some(setup) = setup {
            let Some(unit) = builder.facts.units.iter().find(|unit| unit.id == setup) else {
                return Err(TemplateRouteIssue {
                    span,
                    kind: TemplateRouteIssueKind::MissingProgram,
                });
            };
            unit.scope
        } else {
            ScopeId(0)
        };
        // No neutral factory is exposed before this actual whole-root guard.
        *state = TemplateRouteState::Walking;
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
    pub fn child(&mut self, child: ChildView<'_, 'a>) -> Result<NodeId, TemplateRouteIssue> {
        if !matches!(self.state, TemplateRouteState::Walking)
            || self.root.facts.template_walk.interruption().is_some()
        {
            return self.reject(TemplateRouteIssueKind::Interrupted);
        }
        if !core::ptr::eq(child.component(), self.selected.component())
            || child.parent_element().is_some()
            || child.ordinal() != self.cursor
        {
            return self.reject(TemplateRouteIssueKind::InvalidEvent);
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
    /// Consume a possible adjacent pair through the same original root iterator.
    /// Ordinary events and every existing direct caller retain `child` behavior.
    pub fn child_with_tail<'o>(
        &mut self,
        child: ChildView<'o, 'a>,
        tail: &mut ChildrenView<'o, 'a>,
    ) -> Result<NodeId, TemplateRouteIssue> {
        if self.cursor != 0
            || self.expected != 2
            || !matches!(child.surface(), SurfaceChild::Element(_))
        {
            return self.child(child);
        }
        if !matches!(self.state, TemplateRouteState::Walking)
            || self.root.facts.template_walk.interruption().is_some()
        {
            return self.reject(TemplateRouteIssueKind::Interrupted);
        }
        if !core::ptr::eq(child.component(), self.selected.component())
            || child.parent_element().is_some()
            || child.ordinal() != self.cursor
        {
            return self.reject(TemplateRouteIssueKind::InvalidEvent);
        }
        let selected = self.selected;
        let span = selected.component().block().span();
        let result = self.root.with_walk(span, |root| {
            vize_l0::ensure_sufficient_stack(|| pair::construct(selected, child, tail, root))
        });
        match result {
            Ok((node, consumed)) => {
                self.cursor += consumed;
                Ok(node)
            }
            Err(kind) => self.reject(kind),
        }
    }
    /// Consume the selected original root cursor through its sealed L1 text
    /// receipt. Omitted source text advances custody without creating an op.
    pub fn root_text(
        &mut self,
        receipt: &RootText<'a>,
    ) -> Result<Option<NodeId>, TemplateRouteIssue> {
        if !matches!(self.state, TemplateRouteState::Walking)
            || self.root.facts.template_walk.interruption().is_some()
        {
            return self.reject(TemplateRouteIssueKind::Interrupted);
        }
        let Some(view) = receipt.admitted_for_root_at(self.selected, self.cursor) else {
            return self.reject(TemplateRouteIssueKind::InvalidEvent);
        };
        let result = match view.receipt().content() {
            Some(content) => self
                .root
                .text(content, view.receipt().span())
                .map(Some)
                .map_err(TemplateRouteIssueKind::Artifact),
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
        child: ChildView<'_, 'a>,
        input: crate::lang::js::InterpolationInput<'a>,
    ) -> Result<NodeId, TemplateRouteIssue> {
        let check = if !matches!(self.state, TemplateRouteState::Walking)
            || self.root.facts.template_walk.interruption().is_some()
        {
            Err(TemplateRouteIssueKind::Interrupted)
        } else if !core::ptr::eq(child.component(), self.selected.component())
            || child.parent_element().is_some()
            || child.ordinal() != self.cursor
        {
            Err(TemplateRouteIssueKind::InvalidEvent)
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

    /// Prepare at this actual original event and retain the complete result
    /// before any source-policy projection or canonical mint. This root-only
    /// source profile confers no native target output eligibility.
    pub fn root_text_value(
        &mut self,
        child: ChildView<'_, 'a>,
    ) -> Result<NodeId, TemplateRouteIssue> {
        if !matches!(self.state, TemplateRouteState::Walking)
            || self.root.facts.template_walk.interruption().is_some()
        {
            return self.reject(TemplateRouteIssueKind::Interrupted);
        }
        // The bounded row is already normally owned: a repeated observation
        // must not prepare the same original child or replace it a second time.
        if !self.root.facts.file_text_values.is_empty() {
            return self.reject(TemplateRouteIssueKind::InvalidEvent);
        }
        let selected = self.selected;
        let span = selected.component().block().span();
        let cursor = self.cursor;
        let expected = self.expected;
        let result = self.root.with_walk(span, |root| {
            text_value::construct(selected, child, root, cursor, expected)
        });
        match result {
            Ok(node) => {
                self.cursor += 1;
                Ok(node)
            }
            Err(kind) => self.reject(kind),
        }
    }

    fn reject<T>(&mut self, kind: TemplateRouteIssueKind) -> Result<T, TemplateRouteIssue> {
        if let TemplateRouteState::Refused(issue) = *self.state {
            return Err(issue);
        }
        let span = match kind {
            TemplateRouteIssueKind::Handler { span, .. }
            | TemplateRouteIssueKind::For { span, .. }
            | TemplateRouteIssueKind::TextValuePreparation { span, .. }
            | TemplateRouteIssueKind::InterpolationPreparation { span, .. } => span,
            _ => self.selected.component().block().span(),
        };
        let issue = TemplateRouteIssue { span, kind };
        *self.state = TemplateRouteState::Refused(issue);
        self.root.facts.template_issues.push(TemplateIssue {
            node: None,
            span,
            kind: FileIssueKind::UnsupportedSyntax,
        });
        Err(issue)
    }
    pub fn complete(mut self) -> Result<(), TemplateRouteIssue> {
        if !matches!(self.state, TemplateRouteState::Walking)
            || self.root.facts.template_walk.interruption().is_some()
        {
            return self.reject(TemplateRouteIssueKind::Interrupted);
        }
        if self.cursor != self.expected {
            return self.reject(TemplateRouteIssueKind::IncompleteChildren);
        }
        self.root.facts.template_walk.complete(self.previous);
        if self.root.facts.template_walk.interruption().is_some() {
            return self.reject(TemplateRouteIssueKind::Interrupted);
        }
        *self.state = TemplateRouteState::Complete;
        self.armed = false;
        Ok(())
    }
}

#[cfg(test)]
mod interruption_tests;
impl Drop for FileTemplateWalk<'_, '_> {
    fn drop(&mut self) {
        if self.armed {
            if matches!(self.state, TemplateRouteState::Walking) {
                *self.state = TemplateRouteState::Interrupted;
            }
            self.root.facts.template_walk.interrupt();
        }
    }
}
