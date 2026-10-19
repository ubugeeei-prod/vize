//! Demand-only custody from the original markup producer, never a second tree walk.

use oxc_span::Span as JsSpan;
use vize_l0::{SourceRoot, Span};
use vize_l1::embed::{
    Embed, EmbedSource, Grammar, Lang, Shape, prepare_attribute_value,
    prepare_vue_interpolation_in,
    syntax::{NativeForHead, NativeForInput, RetainedHandlerBody, RetainedSlotParams, parse_once},
};
use vize_relief::{
    JsExpression, RetainedJsAst, RootNode, SimpleExpressionNode, errors::CompilerError,
};

use super::Parser;

const TEMPLATE_SYNTAX_RECORD_LIMIT: usize = 4096;

pub(super) struct TemplateSyntaxCapture<'a> {
    records: std::vec::Vec<TemplateSyntaxRecord<'a>>,
    complete: bool,
    legacy_slot_bindings: bool,
}

/// The actual grammar selected by the original attribute/interpolation callback.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateSyntaxRole {
    Expression,
    Slot,
    For,
    HandlerExpression,
    Handler,
    LegacySlot,
}

/// Existing expression/slot roots or actual native owners, including local refusals.
/// A missing root is a refused parse, never evidence of an empty scope.
#[doc(hidden)]
#[derive(Debug)]
pub enum TemplateSyntax<'a> {
    Expression(Option<JsExpression<'a>>),
    Slot(Option<RetainedJsAst<'a>>),
    For(std::boxed::Box<NativeForHead<'a>>),
    /// The original complete expression of a named event, at original offsets.
    HandlerExpression(JsExpression<'a>),
    Handler(std::boxed::Box<RetainedHandlerBody<'a>>),
    LegacySlot(std::boxed::Box<RetainedSlotParams<'a>>),
    Refused(TemplateSyntaxRole),
}

/// Source and syntax admitted together at one original producer callback.
/// Private fields prevent a consumer from attaching a foreign map to an AST.
#[doc(hidden)]
#[derive(Debug)]
pub struct TemplateSyntaxRecord<'a> {
    span: JsSpan,
    cooked: &'a str,
    source: Option<EmbedSource<'a>>,
    syntax: TemplateSyntax<'a>,
}

impl<'a> TemplateSyntaxRecord<'a> {
    pub const fn syntax(&self) -> &TemplateSyntax<'a> {
        &self.syntax
    }

    pub const fn source(&self) -> Option<EmbedSource<'a>> {
        self.source
    }

    pub const fn span(&self) -> JsSpan {
        self.span
    }

    pub const fn cooked(&self) -> &'a str {
        self.cooked
    }

    /// Project complete AST tokens, not semantic-name lengths. For aliases and
    /// collections use their separate actual owners' `authored_span` methods.
    pub fn authored_span(&self, ast: JsSpan) -> Option<Span> {
        match &self.syntax {
            TemplateSyntax::Expression(Some(_)) | TemplateSyntax::HandlerExpression(_) => self
                .source?
                .authored_span(Span::new(ast.start, ast.end))
                .ok(),
            TemplateSyntax::Slot(Some(retained)) => {
                retained.as_slot_parameters()?.ok()?;
                self.source?
                    .authored_span(Span::new(ast.start, ast.end))
                    .ok()
            }
            TemplateSyntax::Handler(owner) => {
                owner.body()?;
                owner.authored_span(ast).ok()
            }
            TemplateSyntax::LegacySlot(owner) => {
                owner.parameters()?;
                owner.authored_span(ast).ok()
            }
            _ => None,
        }
    }
}

/// Non-serialized, separately owned observations bound to one exact source buffer.
/// Ordinary `parse()` leaves capture disabled and allocates no record storage.
#[doc(hidden)]
#[derive(Debug)]
pub struct TemplateSyntaxFacts<'a> {
    source: &'a str,
    records: std::vec::Vec<TemplateSyntaxRecord<'a>>,
    complete: bool,
    legacy_slot_bindings: bool,
}

impl<'a> TemplateSyntaxFacts<'a> {
    pub(super) fn new(source: &'a str, capture: TemplateSyntaxCapture<'a>) -> Self {
        let TemplateSyntaxCapture {
            mut records,
            complete,
            legacy_slot_bindings,
        } = capture;
        records.sort_unstable_by_key(|record| (record.span.start, record.span.end));
        Self {
            source,
            records,
            complete,
            legacy_slot_bindings,
        }
    }

    /// Dialect selected by the original producer, never inferred from attributes.
    pub fn legacy_slot_bindings_enabled(&self) -> bool {
        self.legacy_slot_bindings
    }

    /// A truncated or undemanded packet cannot authorize any conclusion.
    pub fn is_complete_for(&self, root_source: &str) -> bool {
        self.complete && core::ptr::eq(self.source, root_source)
    }

    pub fn records_for(&self, root_source: &str) -> Option<&[TemplateSyntaxRecord<'a>]> {
        self.is_complete_for(root_source)
            .then_some(self.records.as_slice())
    }

    /// Query the original whole expression/value by source identity, extent and
    /// cooked text. Ambiguous observations cannot authorize a consumer.
    pub fn syntax_for(
        &self,
        root_source: &str,
        span: JsSpan,
        cooked: &str,
    ) -> Option<&TemplateSyntaxRecord<'a>> {
        let records = self.records_for(root_source)?;
        let start = records.partition_point(|record| record.span.start < span.start);
        let mut matches = records
            .get(start..)?
            .iter()
            .take_while(|record| record.span.start == span.start)
            .filter(|record| record.span == span && record.cooked == cooked);
        let record = matches.next()?;
        matches.next().is_none().then_some(record)
    }
}

impl<'a> Parser<'a> {
    /// Retain original syntax owners only when requested by a semantic consumer.
    #[doc(hidden)]
    pub fn parse_with_template_syntax_facts(
        mut self,
    ) -> (
        RootNode<'a>,
        std::vec::Vec<CompilerError>,
        TemplateSyntaxFacts<'a>,
    ) {
        self.template_syntax_facts = Some(TemplateSyntaxCapture {
            records: std::vec::Vec::new(),
            complete: true,
            legacy_slot_bindings: self.options.dialect.is_legacy(),
        });
        let (root, errors, _, syntax) = self.parse_inner();
        // Capture is armed above. If an internal early-return ever loses it,
        // refuse the packet instead of minting empty-scope authority or panicking.
        let syntax = syntax.unwrap_or_else(|| TemplateSyntaxFacts {
            source: root.source,
            records: std::vec::Vec::new(),
            complete: false,
            legacy_slot_bindings: false,
        });
        (root, errors, syntax)
    }

    pub(in crate::parser) fn discard_frozen_template_syntax(&mut self, start: u32, end: u32) {
        if let Some(capture) = self.template_syntax_facts.as_mut() {
            capture
                .records
                .retain(|record| record.span.start < start || record.span.end > end);
        }
    }

    fn admit_template_syntax_record(&mut self) -> bool {
        let Some(capture) = self.template_syntax_facts.as_mut() else {
            return false;
        };
        if !capture.complete {
            return false;
        }
        if capture.records.len() == TEMPLATE_SYNTAX_RECORD_LIMIT {
            capture.complete = false;
            capture.records.clear();
            return false;
        }
        true
    }

    #[inline]
    pub(in crate::parser) fn record_expression_syntax(
        &mut self,
        node: &SimpleExpressionNode<'a>,
        attribute: bool,
        interpolation: bool,
        slot: bool,
    ) {
        if self.template_syntax_facts.is_none() {
            return;
        }
        let role = if slot {
            TemplateSyntaxRole::Slot
        } else {
            TemplateSyntaxRole::Expression
        };
        self.record_retained_template_syntax(node, attribute, interpolation, role);
    }

    #[inline]
    pub(in crate::parser) fn record_handler_expression_syntax(
        &mut self,
        node: &SimpleExpressionNode<'a>,
    ) {
        if self.template_syntax_facts.is_none() {
            return;
        }
        self.record_retained_template_syntax(
            node,
            true,
            false,
            TemplateSyntaxRole::HandlerExpression,
        );
    }

    fn record_retained_template_syntax(
        &mut self,
        node: &SimpleExpressionNode<'a>,
        attribute: bool,
        interpolation: bool,
        role: TemplateSyntaxRole,
    ) {
        if self.in_v_pre || !self.admit_template_syntax_record() {
            return;
        }
        let span = node.loc.span;
        let source = if attribute {
            prepare_attribute_value(self.allocator, self.source, span)
        } else if interpolation {
            prepare_vue_interpolation_in(self.allocator, self.source, span)
        } else {
            EmbedSource::authored(self.source, span)
        }
        .ok()
        .filter(|source| source.text() == node.content);
        let syntax = if source.is_none() {
            TemplateSyntax::Refused(role)
        } else if role == TemplateSyntaxRole::Slot {
            TemplateSyntax::Slot(node.js_ast.filter(|retained| retained.raw == node.content))
        } else if role == TemplateSyntaxRole::HandlerExpression {
            match node
                .js_ast
                .filter(|retained| retained.raw == node.content)
                .and_then(RetainedJsAst::as_expression)
            {
                Some(expression) => TemplateSyntax::HandlerExpression(expression),
                None => TemplateSyntax::Refused(role),
            }
        } else {
            TemplateSyntax::Expression(
                node.js_ast
                    .filter(|retained| retained.raw == node.content)
                    .and_then(RetainedJsAst::as_expression),
            )
        };
        self.push_template_syntax(node.content, span, source, syntax);
    }

    /// Select an original For goal, failed expression's HandlerBody, or legacy
    /// parameters. This never retries the general expression parser.
    pub(in crate::parser) fn retain_special_template_syntax(
        &mut self,
        cooked: &'a str,
        span: Span,
        role: TemplateSyntaxRole,
    ) -> bool {
        if self.template_syntax_facts.is_none() || self.in_v_pre {
            return false;
        }
        if !self.admit_template_syntax_record() {
            return true;
        }
        if role == TemplateSyntaxRole::For {
            let owner = SourceRoot::new(self.source)
                .ok()
                .and_then(|root| {
                    let raw = self.source.get(span.start as usize..span.end as usize)?;
                    NativeForInput::attribute_in(self.allocator, root.whole_block(), raw, Lang::Ts)
                        .ok()
                })
                .map(NativeForInput::observe);
            let (source, syntax) = match owner {
                Some(owner) if owner.source().text() == cooked => (
                    Some(owner.source()),
                    TemplateSyntax::For(std::boxed::Box::new(owner)),
                ),
                _ => (None, TemplateSyntax::Refused(role)),
            };
            self.push_template_syntax(cooked, span, source, syntax);
            return true;
        }
        let source = prepare_attribute_value(self.allocator, self.source, span)
            .ok()
            .filter(|source| source.text() == cooked);
        let syntax = match source {
            Some(source) => {
                let shape = match role {
                    TemplateSyntaxRole::Handler => Shape::HandlerBody,
                    TemplateSyntaxRole::LegacySlot => Shape::SlotParams,
                    _ => return false,
                };
                let native = parse_once(
                    self.allocator,
                    Embed {
                        grammar: Grammar {
                            lang: Lang::Ts,
                            shape,
                        },
                        source,
                    },
                );
                match role {
                    TemplateSyntaxRole::Handler => match native.into_handler_body() {
                        Ok(owner) => TemplateSyntax::Handler(std::boxed::Box::new(owner)),
                        Err(_) => TemplateSyntax::Refused(role),
                    },
                    TemplateSyntaxRole::LegacySlot => match native.into_slot_params() {
                        Ok(owner) => TemplateSyntax::LegacySlot(std::boxed::Box::new(owner)),
                        Err(_) => TemplateSyntax::Refused(role),
                    },
                    _ => TemplateSyntax::Refused(role),
                }
            }
            None => TemplateSyntax::Refused(role),
        };
        self.push_template_syntax(cooked, span, source, syntax);
        true
    }

    fn push_template_syntax(
        &mut self,
        cooked: &'a str,
        span: Span,
        source: Option<EmbedSource<'a>>,
        syntax: TemplateSyntax<'a>,
    ) {
        if let Some(capture) = self.template_syntax_facts.as_mut() {
            capture.records.push(TemplateSyntaxRecord {
                span: JsSpan::new(span.start, span.end),
                cooked,
                source,
                syntax,
            });
        }
    }
}

#[cfg(test)]
#[path = "template_syntax_tests.rs"]
mod tests;
