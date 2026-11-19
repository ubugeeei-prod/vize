//! Inline interruption evidence; no diagnostic allocation during unwinding.

use super::{FileIssueKind, TemplateIssue};
use vize_l0::Span;

#[derive(Debug, Clone, Copy)]
pub(crate) enum TemplateWalk {
    Idle,
    Complete,
    Pending(Span),
    Interrupted(Span),
}

impl TemplateWalk {
    pub(crate) fn enter(&mut self, span: Span) -> Self {
        let previous = *self;
        if !matches!(self, Self::Interrupted(_)) {
            *self = Self::Pending(span);
        }
        previous
    }

    pub(crate) fn complete(&mut self, previous: Self) {
        if matches!(self, Self::Pending(_)) {
            *self = match previous {
                Self::Pending(span) => Self::Pending(span),
                _ => Self::Complete,
            };
        }
    }

    pub(crate) fn interrupt(&mut self) {
        if let Self::Pending(span) = *self {
            *self = Self::Interrupted(span);
        }
    }

    pub(crate) fn is_complete(self) -> bool {
        matches!(self, Self::Idle | Self::Complete)
    }

    pub(crate) fn interruption(self) -> Option<TemplateIssue> {
        match self {
            Self::Interrupted(span) => Some(TemplateIssue {
                node: None,
                span,
                kind: FileIssueKind::InterruptedTemplate,
            }),
            _ => None,
        }
    }
}
