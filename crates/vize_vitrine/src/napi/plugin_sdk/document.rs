//! The plugin document: one SFC template lowered to S2 and flattened into
//! page-order node records (P4-16 spike).
//!
//! Every op line of the S2 page — ops and attached bindings — becomes one
//! [`PluginNode`] whose `id` is its S2 `NodeId` (dense page order,
//! `folio-format.md` "Node numbering"), so a JS report names a node the way
//! the S2 side tables key it. The records are owned: the document outlives
//! the lowering arena, which both crossing shapes the spike measures need
//! (a serialized batch and a proxy handle).

#![expect(
    clippy::disallowed_types,
    reason = "N-API values cross the boundary as std `String`s"
)]
#![expect(
    clippy::disallowed_methods,
    reason = "N-API values cross the boundary as std `String`s"
)]

use serde::Serialize;
use vize_croquis::sfc::{SfcParseOptions, parse_sfc};
use vize_davinci::id::NodeId;
use vize_s0::{Allocator, SourceRoot, Span};
use vize_s1_to_s2::lower_source_block;
use vize_s2::expr::ExprRef;
use vize_s2::op::{BindingOp, DynamicName, Op, Region};

use super::error::HostError;

/// One S2 op line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PluginNode {
    /// The S2 `NodeId` (page order).
    pub id: u32,
    /// The owning op (a binding's element, a region's op).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<u32>,
    /// The S2 mnemonic (`ui.element`, `ui.for`, `ui.bind`, ...).
    pub kind: &'static str,
    /// File-absolute byte range. Host-side only: reports anchor on it, so
    /// it never crosses (a plugin cannot report a range the file lacks).
    #[serde(skip)]
    pub start: u32,
    #[serde(skip)]
    pub end: u32,
    /// Tag, component or static binding name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Expression or text: a binding's value, a handler, an interpolation,
    /// a `ui.for` source, text content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Static attributes, as authored.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attrs: Vec<(String, Option<String>)>,
    /// A `ui.for`'s alias positions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<ForAlias>,
}

/// The authored text of a `ui.for`'s three alias positions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ForAlias {
    pub value: String,
    pub key: Option<String>,
    pub index: Option<String>,
}

/// One SFC's template as page-order node records plus the scope names the
/// lowering attached.
#[derive(Debug)]
pub struct PluginDocument {
    pub filename: String,
    pub source: String,
    pub nodes: Vec<PluginNode>,
    /// `lowered.scopes` names per binding-introducing op, in id order.
    pub scopes: Vec<(u32, Vec<String>)>,
}

impl PluginDocument {
    /// Split `source` with the shared SFC splitter, lower its template
    /// through S1→S2, and flatten the page. No template → no nodes.
    ///
    /// # Errors
    ///
    /// [`HostError::Split`] when the SFC container cannot be split.
    pub fn build(source: &str, filename: &str) -> Result<Self, HostError> {
        let descriptor = parse_sfc(source, SfcParseOptions::default())
            .map_err(|error| HostError::Split(error.message.to_string()))?;
        let mut document = Self {
            filename: filename.to_owned(),
            source: source.to_owned(),
            nodes: Vec::new(),
            scopes: Vec::new(),
        };
        let Some(template) = descriptor.template else {
            return Ok(document);
        };
        let (start, end) = (template.loc.start, template.loc.end);
        let Some(content) = source.get(start..end) else {
            return Ok(document);
        };
        let root = SourceRoot::new(source).map_err(|_| HostError::Split("too large".into()))?;
        let block = root
            .block(content, start as u32)
            .map_err(|_| HostError::Split("template is not a source slice".into()))?;
        let allocator = Allocator::new();
        let (tree, errors) = vize_s1::parse(&allocator, content);
        let lowered = lower_source_block(&allocator, &tree, &errors, block);
        let mut walk = Walk::default();
        walk.region(&lowered.root, None);
        document.nodes = walk.nodes;
        for id in 0..lowered.op_count {
            let facts = NodeId::from_index(id).and_then(|node| lowered.scopes.get(node));
            if let Some(facts) = facts {
                let names = facts.bindings.iter().map(|b| b.name.to_string()).collect();
                document.scopes.push((id, names));
            }
        }
        Ok(document)
    }

    /// 1-based line and UTF-16 column of a byte offset.
    #[must_use]
    pub fn position(&self, offset: u32) -> (u32, u32) {
        let offset = (offset as usize).min(self.source.len());
        let offset = self.source.floor_char_boundary(offset);
        let before = self.source.get(..offset).unwrap_or_default();
        let current_line = before.rsplit_once('\n').map_or(before, |(_, line)| line);
        let line = before.bytes().filter(|byte| *byte == b'\n').count() + 1;
        let column = current_line.encode_utf16().count() + 1;
        (line as u32, column as u32)
    }
}

#[derive(Default)]
struct Walk {
    nodes: Vec<PluginNode>,
}

impl Walk {
    /// Append a node filled in by `fill`; returns its index.
    fn push(
        &mut self,
        parent: Option<u32>,
        kind: &'static str,
        span: Span,
        fill: impl FnOnce(&mut PluginNode),
    ) -> usize {
        let at = self.nodes.len();
        let mut node = PluginNode {
            id: at as u32,
            parent,
            kind,
            start: span.start,
            end: span.end,
            name: None,
            value: None,
            attrs: Vec::new(),
            alias: None,
        };
        fill(&mut node);
        self.nodes.push(node);
        at
    }

    fn region(&mut self, region: &Region<'_>, parent: Option<u32>) {
        for op in region.ops.iter() {
            self.op(op, parent);
        }
    }

    fn op(&mut self, op: &Op<'_>, parent: Option<u32>) {
        match op {
            Op::Element(element) => {
                let at = self.push(parent, op.mnemonic(), element.span, |node| {
                    node.name = Some(element.tag.to_owned());
                    node.attrs = attrs(&element.attributes);
                });
                self.bindings(&element.bindings, at as u32);
                self.region(&element.children, Some(at as u32));
            }
            Op::Component(component) => {
                let at = self.push(parent, op.mnemonic(), component.span, |node| {
                    node.name = Some(component.name.to_owned());
                    node.attrs = attrs(&component.attributes);
                });
                self.bindings(&component.bindings, at as u32);
                self.region(&component.children, Some(at as u32));
            }
            Op::Text(text) => {
                self.push(parent, op.mnemonic(), text.span, |node| {
                    node.value = Some(text.content.to_owned());
                });
            }
            Op::Interpolation(interpolation) => {
                self.push(parent, op.mnemonic(), interpolation.span, |node| {
                    node.value = Some(interpolation.expression.source().to_owned());
                });
            }
            Op::Comment(comment) => {
                self.push(parent, op.mnemonic(), comment.span, |node| {
                    node.value = Some(comment.content.to_owned());
                });
            }
            Op::If(if_op) => {
                let at = self.push(parent, op.mnemonic(), if_op.span, |_| {}) as u32;
                for branch in if_op.branches.iter() {
                    self.region(&branch.region, Some(at));
                }
            }
            Op::For(for_op) => {
                let binding = &for_op.binding;
                let at = self.push(parent, op.mnemonic(), for_op.span, |node| {
                    node.value = Some(binding.source.source().to_owned());
                    node.alias = Some(ForAlias {
                        value: binding.value.source().to_owned(),
                        key: binding.key.as_ref().map(expr_text),
                        index: binding.index.as_ref().map(expr_text),
                    });
                });
                self.region(&for_op.region, Some(at as u32));
            }
            Op::Slot(slot) => {
                let at = self.push(parent, op.mnemonic(), slot.span, |node| {
                    node.name = static_name(Some(&slot.name));
                    node.attrs = attrs(&slot.attributes);
                });
                self.bindings(&slot.bindings, at as u32);
                self.region(&slot.fallback, Some(at as u32));
            }
        }
    }

    fn bindings(&mut self, bindings: &[BindingOp<'_>], owner: u32) {
        for binding in bindings {
            let (span, name, value) = binding_parts(binding);
            self.push(Some(owner), binding.mnemonic(), span, |node| {
                node.name = name;
                node.value = value;
            });
        }
    }
}

fn expr_text(expr: &ExprRef<'_>) -> String {
    expr.source().to_owned()
}

fn static_name(name: Option<&DynamicName<'_>>) -> Option<String> {
    match name {
        Some(DynamicName::Static(name)) => Some((*name).to_owned()),
        _ => None,
    }
}

fn attrs(attributes: &[vize_s2::op::Attribute<'_>]) -> Vec<(String, Option<String>)> {
    let own =
        |attr: &vize_s2::op::Attribute<'_>| (attr.name.to_owned(), attr.value.map(str::to_owned));
    attributes.iter().map(own).collect()
}

/// A binding line's span, static name and expression text.
fn binding_parts(binding: &BindingOp<'_>) -> (Span, Option<String>, Option<String>) {
    let value = |expr: Option<&ExprRef<'_>>| expr.map(expr_text);
    match binding {
        BindingOp::Bind(bind) => (
            bind.span,
            static_name(bind.name.as_ref()),
            value(bind.value.as_ref()),
        ),
        BindingOp::On(on) => (
            on.span,
            static_name(on.name.as_ref()),
            value(on.handler.as_ref()),
        ),
        BindingOp::Model(model) => (model.span, static_name(model.argument.as_ref()), None),
        BindingOp::SlotContent(slot) => (
            slot.span,
            static_name(slot.name.as_ref()),
            value(slot.params.as_ref()),
        ),
        BindingOp::VueDirective(directive) => (
            directive.span,
            Some(directive.name.to_owned()),
            value(directive.value.as_ref()),
        ),
        BindingOp::VueCssBind(op) => (op.span, None, None),
        BindingOp::VueSync(op) => (op.span, None, None),
        BindingOp::VueSlotScope(op) => (op.span, None, None),
        BindingOp::VueOnce(op) => (op.span, None, None),
        BindingOp::VueMemo(op) => (op.span, None, None),
        BindingOp::VueShow(op) => (op.span, None, None),
        BindingOp::VueHtml(op) => (op.span, None, None),
        BindingOp::VueText(op) => (op.span, None, None),
        BindingOp::VueCloak(op) => (op.span, None, None),
    }
}
