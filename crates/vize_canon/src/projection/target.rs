//! The S2 projection as an S4 target (P4-5b, slice 1).
//!
//! The TypeScript projection of an SFC is emitted from the S2 template ops
//! and the retained script text into the one S4 [`EmitDocument`] every
//! backend writes, and its [`ProjectionMapping`] is read off the document's
//! links — no projection-private mapping model or emitter. Each authored
//! construct becomes one statement linked to the construct
//! ([`EmitDocument::link_since`]) whose expression text is linked to the
//! expression, so diagnostics land on the exact authored bytes.
//!
//! Expressions project through their [`ExprRef`] payload: a retained JS
//! expression is emitted verbatim; a foreign, filter or opaque payload has no
//! projectable AST, follows the pessimal law (nothing is emitted, so nothing
//! is claimed) and is counted per class.
//!
//! Slice 1 covers the construct walk and the linking discipline; scoping
//! (context bindings, `v-for`/slot parameter types, component props) is the
//! differential's job to close, measured by
//! `tests/davinci_projection_differential.rs`.

use std::ops::Range;

use vize_atelier_core::codegen::document::EmitDocument;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_s0::{Allocator, SourceRoot, Span};
use vize_s2::expr::{ExprRef, OpaqueReason};
use vize_s2::op::{BindingOp, DynamicName, Op, Region};

use crate::virtual_ts::ProjectionMapping;

/// How the projection treated each expression payload class.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExprClassCounts {
    /// Retained JS expressions emitted verbatim.
    pub projected: u32,
    /// Foreign-dialect expressions (no projection until phase 6).
    pub foreign: u32,
    /// Vue 2 filter chains.
    pub filter: u32,
    /// Opaque expressions, per [`OpaqueReason`] in declaration order:
    /// for-value, multi-statement, nesting-refused, parse-rejected, compound.
    pub opaque: [u32; 5],
}

/// One SFC projected through the S4 document.
#[derive(Debug)]
pub struct S2Projection {
    pub document: EmitDocument,
    pub mapping: ProjectionMapping,
    pub counts: ExprClassCounts,
}

/// Project `source` (a whole SFC), or `None` when it does not parse as one.
pub fn project_sfc(source: &str) -> Option<S2Projection> {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).ok()?;
    let mut target = Target {
        document: EmitDocument::new(true),
        counts: ExprClassCounts::default(),
    };
    for script in [descriptor.script.as_ref(), descriptor.script_setup.as_ref()]
        .into_iter()
        .flatten()
    {
        let start = script.loc.start as u32;
        target.document.push_linked(
            &script.content,
            Span::new(start, start + script.content.len() as u32),
        );
        target.document.push_str("\n");
    }
    if let Some(template) = descriptor.template.as_ref() {
        let start = template.loc.start;
        let block = content_block(source, start..start + template.content.len())?;
        let allocator = Allocator::default();
        let (tree, errors) = vize_s1::parse(&allocator, block.source());
        let lowered = vize_s1_to_s2::lower_source_block(&allocator, &tree, &errors, block);
        target.document.push_str("function __vize_template() {\n");
        target.region(&lowered.root);
        target.document.push_str("}\n");
    }
    let mapping = ProjectionMapping::from_emit_document(&target.document);
    Some(S2Projection {
        document: target.document,
        mapping,
        counts: target.counts,
    })
}

fn content_block(source: &str, range: Range<usize>) -> Option<vize_s0::SourceBlock<'_>> {
    SourceRoot::new(source)
        .ok()?
        .block(source.get(range.clone())?, range.start as u32)
        .ok()
}

struct Target {
    document: EmitDocument,
    counts: ExprClassCounts,
}

impl Target {
    fn region(&mut self, region: &Region<'_>) {
        for op in &region.ops {
            self.op(op);
        }
    }

    fn op(&mut self, op: &Op<'_>) {
        match op {
            Op::Element(element) => {
                self.bindings(&element.bindings);
                self.region(&element.children);
            }
            Op::Component(component) => {
                self.bindings(&component.bindings);
                self.region(&component.children);
            }
            Op::Interpolation(interpolation) => {
                self.statement(interpolation.expression, interpolation.span);
            }
            Op::Text(_) | Op::Comment(_) => {}
            Op::If(node) => {
                for (index, branch) in node.branches.iter().enumerate() {
                    let keyword = if index == 0 { "if (" } else { " else if (" };
                    match branch.condition {
                        Some(condition) => {
                            self.document.push_str(keyword);
                            if self.admit(condition) {
                                self.expression(condition);
                            } else {
                                self.document.push_str("true");
                            }
                            self.document.push_str(") {\n");
                        }
                        None => self.document.push_str(" else {\n"),
                    }
                    self.region(&branch.region);
                    self.document.push_str("}");
                }
                self.document.push_str("\n");
            }
            Op::For(node) => {
                self.document.push_str("{\n");
                self.statement(node.binding.source, node.span);
                for alias in [Some(node.binding.value), node.binding.key, node.binding.index]
                    .into_iter()
                    .flatten()
                {
                    self.declaration(alias);
                }
                self.region(&node.region);
                self.document.push_str("}\n");
            }
            Op::Slot(slot) => {
                if let DynamicName::Dynamic(name) = slot.name {
                    self.statement(name, slot.span);
                }
                self.bindings(&slot.bindings);
                self.region(&slot.fallback);
            }
        }
    }

    fn bindings(&mut self, bindings: &[BindingOp<'_>]) {
        for binding in bindings {
            match binding {
                BindingOp::Bind(bind) => {
                    self.dynamic_name(bind.name, bind.span);
                    self.optional(bind.value, bind.span);
                }
                BindingOp::On(on) => {
                    self.dynamic_name(on.name, on.span);
                    self.optional(on.handler, on.span);
                }
                BindingOp::Model(model) => {
                    self.dynamic_name(model.argument, model.span);
                    self.statement(model.contract.read, model.span);
                }
                BindingOp::SlotContent(slot) => {
                    self.dynamic_name(slot.name, slot.span);
                    if let Some(params) = slot.params {
                        self.declaration(params);
                    }
                }
                BindingOp::VueDirective(directive) => {
                    self.optional(directive.value, directive.span);
                }
                BindingOp::VueCssBind(bind) => self.statement(bind.value, bind.span),
                BindingOp::VueSync(sync) => self.statement(sync.value, sync.span),
                BindingOp::VueSlotScope(scope) => {
                    if let Some(params) = scope.params {
                        self.declaration(params);
                    }
                }
                BindingOp::VueMemo(memo) => self.statement(memo.value, memo.span),
                BindingOp::VueShow(show) => self.statement(show.value, show.span),
                BindingOp::VueHtml(html) => self.optional(html.value, html.span),
                BindingOp::VueText(text) => self.optional(text.value, text.span),
                BindingOp::VueOnce(_) | BindingOp::VueCloak(_) => {}
            }
        }
    }

    fn dynamic_name(&mut self, name: Option<DynamicName<'_>>, construct: Span) {
        if let Some(DynamicName::Dynamic(expression)) = name {
            self.statement(expression, construct);
        }
    }

    fn optional(&mut self, expression: Option<ExprRef<'_>>, construct: Span) {
        if let Some(expression) = expression {
            self.statement(expression, construct);
        }
    }

    /// `void (<expression>);` linked to the authored construct.
    fn statement(&mut self, expression: ExprRef<'_>, construct: Span) {
        if !self.admit(expression) {
            return;
        }
        let start = self.document.len() as u32;
        self.document.push_str("void (");
        self.expression(expression);
        self.document.push_str(");");
        self.document.link_since(start, construct);
        self.document.push_str("\n");
    }

    /// `const <pattern>: any = undefined as any;` for a binding pattern.
    fn declaration(&mut self, pattern: ExprRef<'_>) {
        if !self.admit(pattern) {
            return;
        }
        self.document.push_str("const ");
        self.expression(pattern);
        self.document.push_str(": any = undefined as any;\n");
    }

    fn expression(&mut self, expression: ExprRef<'_>) {
        if let ExprRef::Js(js) = expression {
            self.document.push_linked(js.source, js.span);
        }
    }

    /// Count the payload; only a retained JS expression projects.
    fn admit(&mut self, expression: ExprRef<'_>) -> bool {
        match expression {
            ExprRef::Js(_) => {
                self.counts.projected += 1;
                true
            }
            ExprRef::Foreign(_) => {
                self.counts.foreign += 1;
                false
            }
            ExprRef::Filter(_) => {
                self.counts.filter += 1;
                false
            }
            ExprRef::Opaque(opaque) => {
                self.counts.opaque[opaque_index(opaque.reason)] += 1;
                false
            }
        }
    }
}

const fn opaque_index(reason: OpaqueReason) -> usize {
    match reason {
        OpaqueReason::ForValue => 0,
        OpaqueReason::MultiStatement => 1,
        OpaqueReason::NestingRefused => 2,
        OpaqueReason::ParseRejected => 3,
        OpaqueReason::Compound => 4,
    }
}

#[cfg(test)]
mod tests;
