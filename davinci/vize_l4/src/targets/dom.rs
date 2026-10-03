//! Native virtual-DOM emission from one owner-bound L3 analysis.
//!
//! L3 supplies semantic eligibility, grouping, binding order and block facts.
//! This target encodes those facts into Vue calls, patch flags and property
//! arrays. It never parses an expression or constructs a JavaScript AST.
//! Unsupported surfaces return a typed error and no render fragment; the
//! caller retains the complete artifact, source and provenance diagnostics.

use vize_l0::{Span, ensure_sufficient_stack, id::NodeId};
use vize_l2::expr::ExprRef;
use vize_l2::op::Op;
use vize_l2::walk::NodeRef;
use vize_l3::decision::dom::{DomFacts, DomRootKind, DomUnsupported};
use vize_l3::decision::{DecisionTables, NativeAnalysis, policy::TargetPolicy};

use crate::expr::{AccessProvider, EmitError as ExpressionError, ResolvedExpressions};
use crate::runtime::{Runtime, Vocabulary, vocabulary};
use crate::write::{LinkSink, Writer};

mod children;
mod conditional;
mod element;
mod expression;
mod file;
#[cfg(test)]
mod tests;
mod vue;
mod write;

use expression::{BareExpressions, ExpressionWriter};
pub use file::emit_file;
pub use vue::emit_vue;
use write::{Helpers, helper, quoted};

/// A target refuses the actual node rather than emitting partial JavaScript.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DomError {
    pub node: Option<NodeId>,
    pub span: Span,
    pub kind: DomErrorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomErrorKind {
    WrongPolicy,
    MissingAnalysis,
    MissingNode,
    MissingBinding,
    InvalidGrouping,
    MissingRuntimeHelper,
    Unsupported(DomUnsupported),
    UnsupportedExpression,
    Expression(ExpressionError),
    FileOwnerMismatch,
    MissingFileExpression,
    MissingFileScope,
    RuntimeAccessUnavailable,
    UncertifiedExpressionSpelling,
    OutputTooLarge,
}

/// Emit a complete prepared render declaration, with no import preamble.
///
/// Expression tables are complete native L2 producer outputs, tied to their
/// retained ASTs. This target only consumes them. The module assembler imports
/// helpers afterwards in the fragment's transform-compatible use order.
pub fn emit<L: LinkSink>(
    analysis: &NativeAnalysis<'_, '_>,
    expressions: ResolvedExpressions<'_, '_>,
    access: &impl AccessProvider,
) -> Result<Writer<L>, DomError> {
    encode(
        analysis.artifact().source(),
        analysis.policy(),
        analysis.tables(),
        analysis.dom(),
        BareExpressions {
            analysis,
            expressions,
            access,
        },
    )
}

// Both entries derive these views only from their checked owner. No public
// entry accepts an independent artifact, row list, scope or completion flag.
fn encode<'owner, 'arena, L: LinkSink>(
    source: &'arena str,
    policy: TargetPolicy,
    tables: &DecisionTables,
    facts: Option<&DomFacts<'owner, 'arena>>,
    expressions: impl ExpressionWriter,
) -> Result<Writer<L>, DomError> {
    let whole_source = Span::new(0, source.len() as u32);
    let failure = |kind| DomError {
        node: None,
        span: whole_source,
        kind,
    };
    if policy != TargetPolicy::Dom {
        return Err(failure(DomErrorKind::WrongPolicy));
    }
    let facts = facts.ok_or_else(|| failure(DomErrorKind::MissingAnalysis))?;
    let vocabulary = vocabulary(Runtime::VueDom);
    let helpers =
        Helpers::checked(vocabulary).ok_or_else(|| failure(DomErrorKind::MissingRuntimeHelper))?;
    let mut emitter = Emitter {
        tables,
        facts,
        expressions,
        vocabulary,
        helpers,
        writer: Writer::with_capacity(source.len()),
    };
    if let Some(rejected) = facts.unsupported().first() {
        return Err(DomError {
            node: Some(rejected.node),
            span: rejected.span,
            kind: DomErrorKind::Unsupported(rejected.reason),
        });
    }
    for &dependency in facts.dependencies() {
        emitter
            .writer
            .use_helper(emitter.helpers.dependency(dependency));
    }
    emitter
        .writer
        .push("function render(_ctx, _cache, $props, $setup, $data, $options) {");
    emitter.writer.indent();
    emitter.writer.newline();
    emitter.writer.push("return ");
    match facts.root().kind {
        DomRootKind::Empty => emitter.writer.push("null"),
        DomRootKind::Direct => match facts.root().children.as_slice() {
            [child] => emitter.child(child, false)?,
            _ => return Err(failure(DomErrorKind::InvalidGrouping)),
        },
        DomRootKind::Fragment { single_non_comment } => {
            emitter.root_fragment(single_non_comment)?
        }
    }
    emitter.finish()
}

struct Emitter<'s, 'owner, 'arena, E: ExpressionWriter, L: LinkSink> {
    tables: &'s DecisionTables,
    facts: &'s DomFacts<'owner, 'arena>,
    expressions: E,
    vocabulary: &'static Vocabulary,
    helpers: Helpers,
    writer: Writer<L>,
}

impl<E: ExpressionWriter, L: LinkSink> Emitter<'_, '_, '_, E, L> {
    fn error(&self, node: NodeId, kind: DomErrorKind) -> DomError {
        let span = self
            .facts
            .node(node)
            .map(|fact| NodeRef::Op(fact.op()).span())
            .or_else(|| {
                self.facts
                    .binding(node)
                    .map(|fact| NodeRef::Binding(fact.binding()).span())
            })
            .unwrap_or(Span::new(0, 0));
        DomError {
            node: Some(node),
            span,
            kind,
        }
    }

    fn finish(mut self) -> Result<Writer<L>, DomError> {
        self.writer.deindent();
        self.writer.newline();
        self.writer.push("}");
        if u32::try_from(self.writer.len()).is_err() {
            return Err(DomError {
                node: None,
                span: Span::new(0, 0),
                kind: DomErrorKind::OutputTooLarge,
            });
        }
        Ok(self.writer)
    }

    fn node(&mut self, id: NodeId) -> Result<(), DomError> {
        ensure_sufficient_stack(|| {
            let fact = self
                .facts
                .node(id)
                .ok_or_else(|| self.error(id, DomErrorKind::MissingNode))?;
            match fact.op() {
                Op::Element(element) => self.element(id, element, fact, None),
                Op::If(owner) => self.conditional(id, owner),
                Op::Comment(comment) => {
                    helper(&mut self.writer, self.vocabulary, self.helpers.comment);
                    self.writer.push("(");
                    quoted(&mut self.writer, comment.content, comment.span);
                    self.writer.push(")");
                    Ok(())
                }
                _ => Err(self.error(id, DomErrorKind::InvalidGrouping)),
            }
        })
    }

    fn expression(&mut self, node: NodeId, expression: ExprRef<'_>) -> Result<(), DomError> {
        self.expressions.write(&mut self.writer, node, expression)
    }
}
