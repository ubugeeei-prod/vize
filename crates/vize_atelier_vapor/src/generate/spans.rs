//! Span-carrying Vapor emission (Davinci P3-9, S4).
//!
//! Vapor code is generated from IR, which keeps expression nodes (with their
//! authored spans) but not the elements that carried them. For a
//! map-requesting compile, [`VaporSourceSpans`] is collected once from the
//! transformed template: each structural unit (`v-if`/`v-else-if` branch,
//! `v-else`, `v-for`, `<template #slot>`) is keyed by the authored span of
//! the expression the IR keeps for it (condition, loop source, slot name), so
//! generation looks the unit up by exact span identity. Template strings bring
//! their own anchors from lowering. When no map is requested none of this is
//! built and every write is a plain append (TS-11).

use vize_atelier_core::{
    ElementNode, PropNode, RootNode, SimpleExpressionNode, TemplateChildNode,
    codegen::document::{EmitDocument, SpanLink},
};
use vize_carton::{FxHashMap, String};

use super::context::GenerateContext;

/// Authored anchors a map-requesting Vapor compile needs beyond the IR.
#[derive(Debug, Default)]
pub(crate) struct VaporSourceSpans {
    /// Template-string anchors by template index.
    pub(crate) templates: FxHashMap<usize, std::vec::Vec<SpanLink>>,
    /// Authored start of the template section.
    pub(crate) root: u32,
    /// Expression span start -> authored start of the unit it belongs to.
    units: FxHashMap<u32, u32>,
    /// `v-if`/`v-else-if` condition span start -> the following `v-else`.
    else_units: FxHashMap<u32, u32>,
    /// Tag name -> authored tag-name start of its first element.
    tags: FxHashMap<String, u32>,
}

/// Authored anchors of each registered template string, by template index.
pub(crate) type TemplateSpans = FxHashMap<usize, std::vec::Vec<SpanLink>>;

impl VaporSourceSpans {
    /// Spans for a native S3 compile: its template anchors, and each
    /// control-flow unit keyed by the authored start of the expression the IR
    /// keeps for it (condition, loop source) as the legacy walk keys them.
    pub(crate) fn native(
        templates: TemplateSpans,
        units: FxHashMap<u32, u32>,
        else_units: FxHashMap<u32, u32>,
    ) -> Self {
        Self {
            templates,
            units,
            else_units,
            ..Self::default()
        }
    }

    pub(crate) fn collect(
        root: &RootNode<'_>,
        templates: FxHashMap<usize, std::vec::Vec<SpanLink>>,
    ) -> Self {
        let mut spans = Self {
            templates,
            root: root.loc.span.start,
            ..Self::default()
        };
        spans.walk(&root.children);
        spans
    }

    fn walk(&mut self, children: &[TemplateChildNode<'_>]) {
        for child in children {
            match child {
                TemplateChildNode::Element(el) => self.element(el),
                TemplateChildNode::If(node) => {
                    for (index, branch) in node.branches.iter().enumerate() {
                        if let Some(condition) = &branch.condition {
                            let key = condition.loc().span.start;
                            self.units.insert(key, branch.loc.span.start);
                            if let Some(next) = node.branches.get(index + 1)
                                && next.condition.is_none()
                            {
                                self.else_units.insert(key, next.loc.span.start);
                            }
                        }
                        self.walk(&branch.children);
                    }
                }
                TemplateChildNode::For(node) => {
                    self.units
                        .insert(node.source.loc().span.start, node.loc.span.start);
                    self.walk(&node.children);
                }
                _ => {}
            }
        }
    }

    fn element(&mut self, el: &ElementNode<'_>) {
        self.tags
            .entry(String::new(el.tag))
            .or_insert(el.loc.span.start + 1);
        for prop in el.props.iter() {
            if let PropNode::Directive(dir) = prop
                && dir.name == "slot"
                && let Some(arg) = &dir.arg
            {
                self.units.insert(arg.loc().span.start, el.loc.span.start);
            }
        }
        self.walk(&el.children);
    }
}

/// The `escape_template` replacements, for writing a linked template string
/// into a double-quoted `_template("...")` literal.
pub(crate) const TEMPLATE_ESCAPES: [(&str, &str); 4] =
    [("\\", "\\\\"), ("\"", "\\\""), ("\n", "\\n"), ("\r", "\\r")];

impl GenerateContext<'_> {
    /// Authored start of the unit keyed by `expr`'s span.
    pub(crate) fn unit_of(&self, expr: &SimpleExpressionNode<'_>) -> Option<u32> {
        let spans = self.spans?;
        spans.units.get(&expr.loc.span.start).copied()
    }

    /// Authored start of the `v-else` following the branch keyed by `expr`.
    pub(crate) fn else_unit_of(&self, expr: &SimpleExpressionNode<'_>) -> Option<u32> {
        let spans = self.spans?;
        spans.else_units.get(&expr.loc.span.start).copied()
    }

    /// Authored tag-name start of the first element authoring `tag`.
    pub(crate) fn tag_start(&self, tag: &str) -> Option<u32> {
        self.spans?.tags.get(tag).copied()
    }

    /// `_createIf(() => (condition), () => {`, opening at the authored branch
    /// and carrying the condition's anchors.
    pub(crate) fn if_head(&self, if_node: &crate::ir::IfIRNode<'_>) -> EmitDocument {
        let mut head = self.spanned_at("_createIf(", self.unit_of(&if_node.condition));
        head.push_str("() => ");
        if if_node.condition.is_static {
            head.push_str(&["\"", if_node.condition.content, "\""].concat());
        } else {
            head.push_str("(");
            head.push_spanned(&self.spanned_expression_node(&if_node.condition));
            head.push_str(")");
        }
        head.push_str(", () => {");
        head
    }

    /// `}, () => {` opening the `v-else` branch of `if_node` at its authored
    /// element.
    pub(crate) fn else_head(&self, if_node: &crate::ir::IfIRNode<'_>) -> EmitDocument {
        let mut head = EmitDocument::plain("}, ");
        head.push_spanned(&self.spanned_at("() => {", self.else_unit_of(&if_node.condition)));
        head
    }

    /// `text` anchored at `source` when maps are on and there is a source.
    pub(crate) fn spanned_at(&self, text: &str, source: Option<u32>) -> EmitDocument {
        match source.filter(|_| self.spans.is_some()) {
            Some(source) => EmitDocument::mapped(text, source),
            None => EmitDocument::plain(text),
        }
    }

    /// A resolved expression node, anchored when maps are on.
    pub(crate) fn spanned_expression_node(&self, node: &SimpleExpressionNode<'_>) -> EmitDocument {
        let resolved = self.resolve_expression_node(node);
        let mut piece = EmitDocument::default();
        if self.spans.is_some() {
            piece.push_expression(&resolved, node.loc.span, self.source);
        } else {
            piece.push_str(&resolved);
        }
        piece
    }

    /// Write a spanned piece at the current position.
    pub(crate) fn push_spanned(&mut self, piece: &EmitDocument) {
        self.out.push_spanned(piece);
    }

    /// Write an indented line from a spanned piece.
    pub(crate) fn push_line_spanned(&mut self, piece: &EmitDocument) {
        self.push_indent();
        self.push_spanned(piece);
        self.push("\n");
    }
}
