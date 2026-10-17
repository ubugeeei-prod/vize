//! Demand-only lexical references from one original parser-owned template.
//!
//! This packet is separate from serialized Croquis and its legacy name-only
//! relation. A refused role or scope is never interpreted as a missing binding.
//! The caller must retain the original parser bundle; source equality alone is
//! not a substitute for that owner, just as for `FrozenElements`.

mod roles;
#[cfg(test)]
mod tests;
mod walk;

use vize_armature::parser::TemplateSyntaxFacts;
use vize_l0::{CompactString, Span};
use vize_relief::RootNode;

const MAX_DEPTH: usize = 64;
const MAX_UNITS: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateReferenceOwner {
    /// The original template scopes contain no declaration of this name.
    /// Component property lookup, including reserved instance APIs, is next.
    ComponentContext,
    /// A positively witnessed template declaration, or the selected handler's
    /// implicit `$event` parameter, which has no authored declaration token.
    TemplateLocal { declaration: Option<Span> },
    /// An enclosing binding goal was refused; absence proves nothing.
    Unknown,
}

/// One bare value-identifier lookup. Static member properties and `this`
/// members are different reference kinds and are never relabeled as locals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateReference {
    pub name: CompactString,
    pub span: Span,
    pub owner: TemplateReferenceOwner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateReferenceGapKind {
    /// Source identity or whole producer-capture completeness was not admitted.
    UnqualifiedSource,
    MissingSyntax,
    RefusedSyntax,
    InvalidCoordinates,
    UnsupportedNode,
    TraversalLimit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemplateReferenceGap {
    pub span: Span,
    pub kind: TemplateReferenceGapKind,
}

/// Positive facts and explicit coverage gaps. No public constructor accepts
/// caller-authored reference/owner pairs.
#[derive(Debug, Default)]
pub struct TemplateReferences {
    references: Vec<TemplateReference>,
    gaps: Vec<TemplateReferenceGap>,
}

impl TemplateReferences {
    pub fn references(&self) -> &[TemplateReference] {
        &self.references
    }

    pub fn gaps(&self) -> &[TemplateReferenceGap] {
        &self.gaps
    }
}

#[derive(Clone)]
struct Local {
    name: CompactString,
    span: Span,
}

struct Capture<'facts, 'source> {
    syntax: &'facts TemplateSyntaxFacts<'source>,
    source: &'source str,
    packet: TemplateReferences,
    locals: Vec<Local>,
    unknown_scope: bool,
    remaining: usize,
    traversal_refused: bool,
}

/// Consume actual retained roles without parsing, searching for names or
/// inventing context BindingIds. Bounds cover template nodes and emitted facts;
/// an exhausted traversal rolls back the entire packet's positive facts.
pub fn collect_template_references(
    root: &RootNode<'_>,
    syntax: &TemplateSyntaxFacts<'_>,
) -> TemplateReferences {
    let mut capture = Capture {
        syntax,
        source: root.source,
        packet: TemplateReferences::default(),
        locals: Vec::new(),
        unknown_scope: false,
        remaining: MAX_UNITS,
        traversal_refused: false,
    };
    if syntax.records_for(root.source).is_none() {
        capture.gap(
            Span::new(0, root.source.len() as u32),
            TemplateReferenceGapKind::UnqualifiedSource,
        );
    } else {
        capture.walk(&root.children, 0);
    }
    if capture.traversal_refused {
        capture.packet.references.clear();
    }
    capture
        .packet
        .references
        .sort_by_key(|reference| (reference.span.start, reference.span.end));
    capture.packet
}

impl Capture<'_, '_> {
    fn gap(&mut self, span: Span, kind: TemplateReferenceGapKind) {
        let gap = TemplateReferenceGap { span, kind };
        if !self.packet.gaps.contains(&gap) {
            self.packet.gaps.push(gap);
        }
    }

    fn spend(&mut self, units: usize, span: Span) -> bool {
        if let Some(remaining) = self.remaining.checked_sub(units) {
            self.remaining = remaining;
            true
        } else {
            self.traversal_refused = true;
            self.gap(span, TemplateReferenceGapKind::TraversalLimit);
            false
        }
    }
}

fn source_span(span: oxc_span::Span) -> Span {
    Span::new(span.start, span.end)
}
