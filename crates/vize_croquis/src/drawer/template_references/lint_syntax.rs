//! Demand-only syntax roles derived from the resident L1/L2 lint artifact.

use super::{SyntaxFacts, SyntaxKind, SyntaxRecord};
use crate::drawer::lint_template::CompatibilityRoot;
use oxc_span::Span as JsSpan;
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::embed::syntax::{
    NativeForHead, NativeForInput, RetainedHandlerBody, RetainedSlotParams, parse_once,
};
use vize_l1::embed::{
    Embed, EmbedSource, Grammar, Lang, Shape, prepare_attribute_value, prepare_vue_interpolation_in,
};
use vize_l2::expr::JsExpr;
use vize_relief::{ExpressionNode, PropNode, RootNode, TemplateChildNode};

const RECORD_LIMIT: usize = 4096;

pub(in crate::drawer) struct LintSyntaxFacts<'a> {
    source: &'a str,
    records: Vec<LintSyntaxRecord<'a>>,
    complete: bool,
    legacy_slot_bindings: bool,
}

enum LintSyntax<'a> {
    Expression(Option<&'a JsExpr<'a>>),
    Slot(Option<Box<RetainedSlotParams<'a>>>),
    For(Box<NativeForHead<'a>>),
    HandlerExpression(&'a JsExpr<'a>),
    Handler(Box<RetainedHandlerBody<'a>>),
    LegacySlot(Box<RetainedSlotParams<'a>>),
    Refused,
}

pub(super) struct LintSyntaxRecord<'a> {
    span: JsSpan,
    cooked: &'a str,
    syntax: LintSyntax<'a>,
}

#[derive(Clone, Copy)]
enum SourceKind {
    Authored,
    Attribute,
    Interpolation,
}

#[derive(Clone, Copy)]
enum Role {
    Expression(SourceKind),
    Slot,
    For,
    Handler,
    LegacySlot,
}

impl<'a> LintSyntaxFacts<'a> {
    pub(in crate::drawer) fn collect(
        root: &RootNode<'a>,
        projection: &CompatibilityRoot<'_, 'a>,
    ) -> Self {
        let mut facts = Self {
            source: root.source,
            records: Vec::new(),
            complete: true,
            legacy_slot_bindings: projection.legacy_slot_bindings(),
        };
        facts.walk(&root.children, projection);
        facts
            .records
            .sort_unstable_by_key(|record| (record.span.start, record.span.end));
        facts
    }

    fn walk(&mut self, children: &[TemplateChildNode<'a>], projection: &CompatibilityRoot<'_, 'a>) {
        if !self.complete {
            return;
        }
        for child in children {
            match child {
                TemplateChildNode::Element(element) => {
                    for prop in &element.props {
                        match prop {
                            PropNode::Directive(directive) => {
                                if let Some(argument) = &directive.arg
                                    && !matches!(argument, ExpressionNode::Simple(node) if node.is_static)
                                {
                                    self.record(
                                        argument,
                                        Role::Expression(SourceKind::Authored),
                                        projection,
                                    );
                                }
                                let Some(value) = &directive.exp else {
                                    continue;
                                };
                                let role = match directive.name {
                                    "for" => Role::For,
                                    "slot" => Role::Slot,
                                    "on" if directive.arg.is_some() => Role::Handler,
                                    _ if directive.shorthand => {
                                        Role::Expression(SourceKind::Authored)
                                    }
                                    _ => Role::Expression(SourceKind::Attribute),
                                };
                                self.record(value, role, projection);
                            }
                            PropNode::Attribute(attribute)
                                if self.legacy_slot_bindings
                                    && (attribute.name == "slot-scope"
                                        || (attribute.name == "scope"
                                            && element.tag == "template")) =>
                            {
                                if let Some(value) = &attribute.value {
                                    self.record(
                                        &ExpressionNode::Simple(vize_l0::Box::new_in(
                                            vize_relief::SimpleExpressionNode::new(
                                                value.content,
                                                false,
                                                value.loc.clone(),
                                            ),
                                            &projection.allocator(),
                                        )),
                                        Role::LegacySlot,
                                        projection,
                                    );
                                }
                            }
                            _ => {}
                        }
                    }
                    self.walk(&element.children, projection);
                }
                TemplateChildNode::Interpolation(interpolation) => {
                    self.record(
                        &interpolation.content,
                        Role::Expression(SourceKind::Interpolation),
                        projection,
                    );
                }
                TemplateChildNode::If(node) => {
                    for branch in &node.branches {
                        if let Some(condition) = &branch.condition {
                            self.record(
                                condition,
                                Role::Expression(SourceKind::Authored),
                                projection,
                            );
                        }
                        self.walk(&branch.children, projection);
                    }
                }
                TemplateChildNode::IfBranch(branch) => {
                    if let Some(condition) = &branch.condition {
                        self.record(
                            condition,
                            Role::Expression(SourceKind::Authored),
                            projection,
                        );
                    }
                    self.walk(&branch.children, projection);
                }
                TemplateChildNode::For(node) => {
                    self.record(&node.source, Role::For, projection);
                    self.walk(&node.children, projection);
                }
                TemplateChildNode::Text(_)
                | TemplateChildNode::Comment(_)
                | TemplateChildNode::TextCall(_)
                | TemplateChildNode::CompoundExpression(_)
                | TemplateChildNode::Hoisted(_) => {}
            }
            if !self.complete {
                break;
            }
        }
    }

    fn record(
        &mut self,
        expression: &ExpressionNode<'a>,
        role: Role,
        projection: &CompatibilityRoot<'_, 'a>,
    ) {
        let ExpressionNode::Simple(node) = expression else {
            return;
        };
        if self.records.len() == RECORD_LIMIT {
            self.complete = false;
            self.records.clear();
            return;
        }
        let span = node.loc.span;
        let authored = Span::new(span.start, span.end);
        let source = match role {
            Role::Expression(SourceKind::Authored) => {
                EmbedSource::authored(self.source, authored).ok()
            }
            Role::Expression(SourceKind::Interpolation) => {
                prepare_vue_interpolation_in(projection.allocator(), self.source, authored).ok()
            }
            Role::Expression(SourceKind::Attribute)
            | Role::Slot
            | Role::Handler
            | Role::LegacySlot => {
                prepare_attribute_value(projection.allocator(), self.source, authored).ok()
            }
            Role::For => None,
        }
        .filter(|source| source.text() == node.content);
        let syntax = match role {
            Role::Expression(_) => match source {
                Some(_) => LintSyntax::Expression(projection.js_expression(authored, node.content)),
                None => LintSyntax::Refused,
            },
            Role::Handler => match source {
                None => LintSyntax::Refused,
                Some(source) => match projection.js_expression(authored, node.content) {
                    Some(expression) => LintSyntax::HandlerExpression(expression),
                    None => self
                        .parse_special(projection.allocator(), source, Shape::HandlerBody)
                        .into_handler_body()
                        .ok()
                        .map(|owner| LintSyntax::Handler(Box::new(owner)))
                        .unwrap_or(LintSyntax::Refused),
                },
            },
            Role::Slot => match source {
                Some(source) => self
                    .parse_special(projection.allocator(), source, Shape::SlotParams)
                    .into_slot_params()
                    .ok()
                    .map(|owner| LintSyntax::Slot(Some(Box::new(owner))))
                    .unwrap_or(LintSyntax::Slot(None)),
                None => LintSyntax::Refused,
            },
            Role::LegacySlot => match source {
                Some(source) => self
                    .parse_special(projection.allocator(), source, Shape::SlotParams)
                    .into_slot_params()
                    .ok()
                    .map(|owner| LintSyntax::LegacySlot(Box::new(owner)))
                    .unwrap_or(LintSyntax::Refused),
                None => LintSyntax::Refused,
            },
            Role::For => {
                let owner = SourceRoot::new(self.source)
                    .ok()
                    .and_then(|root| {
                        let raw = self
                            .source
                            .get(authored.start as usize..authored.end as usize)?;
                        NativeForInput::attribute_in(
                            projection.allocator(),
                            root.whole_block(),
                            raw,
                            Lang::Ts,
                        )
                        .ok()
                    })
                    .map(NativeForInput::observe);
                match owner {
                    Some(owner) if owner.source().text() == node.content => {
                        LintSyntax::For(Box::new(owner))
                    }
                    _ => LintSyntax::Refused,
                }
            }
        };
        self.records.push(LintSyntaxRecord {
            span: JsSpan::new(span.start, span.end),
            cooked: node.content,
            syntax,
        });
    }

    fn parse_special(
        &self,
        allocator: &'a Allocator,
        source: EmbedSource<'a>,
        shape: Shape,
    ) -> vize_l1::embed::syntax::NativeSyntax<'a> {
        parse_once(
            allocator,
            Embed {
                grammar: Grammar {
                    lang: Lang::Ts,
                    shape,
                },
                source,
            },
        )
    }
}

impl<'a> SyntaxRecord<'a> for LintSyntaxRecord<'a> {
    fn syntax(&self) -> SyntaxKind<'_, 'a> {
        match &self.syntax {
            LintSyntax::Expression(expression) => {
                SyntaxKind::Expression(expression.map(|expression| vize_relief::JsExpression {
                    ast: expression.ast,
                    raw: expression.source,
                }))
            }
            LintSyntax::Slot(owner) => SyntaxKind::SlotNative(owner.as_deref()),
            LintSyntax::For(owner) => SyntaxKind::For(owner),
            LintSyntax::HandlerExpression(expression) => {
                SyntaxKind::HandlerExpression(vize_relief::JsExpression {
                    ast: expression.ast,
                    raw: expression.source,
                })
            }
            LintSyntax::Handler(owner) => SyntaxKind::Handler(owner),
            LintSyntax::LegacySlot(owner) => SyntaxKind::LegacySlot(owner),
            LintSyntax::Refused => SyntaxKind::Refused,
        }
    }

    fn span(&self) -> JsSpan {
        self.span
    }

    fn authored_span(&self, span: JsSpan) -> Option<Span> {
        match &self.syntax {
            LintSyntax::Expression(Some(expression))
            | LintSyntax::HandlerExpression(expression) => expression
                .ast_span_to_source(span)
                .and_then(|span| expression.authored_span(span)),
            LintSyntax::Slot(Some(owner)) | LintSyntax::LegacySlot(owner) => {
                owner.authored_span(span).ok()
            }
            LintSyntax::Handler(owner) => owner.authored_span(span).ok(),
            LintSyntax::Expression(None)
            | LintSyntax::Slot(None)
            | LintSyntax::For(_)
            | LintSyntax::Refused => None,
        }
    }
}

impl<'a> SyntaxFacts<'a> for LintSyntaxFacts<'a> {
    type Record = LintSyntaxRecord<'a>;

    fn legacy_slot_bindings_enabled(&self) -> bool {
        self.legacy_slot_bindings
    }

    fn records_for(&self, source: &str) -> Option<&[Self::Record]> {
        (self.complete && core::ptr::eq(self.source, source)).then_some(&self.records)
    }

    fn syntax_for(&self, source: &str, span: JsSpan, cooked: &str) -> Option<&Self::Record> {
        let records = self.records_for(source)?;
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
