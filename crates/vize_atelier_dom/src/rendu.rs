//! DOM/VDOM emission from the frontend-neutral Rendu artifact.

mod property;
mod syntax;

use vize_carton::{String, append, source_anchor::SourceAnchor};
use vize_rendu::{
    RenduEscapeMode, RenduNode, RenduNodeId, RenduProperty, RenduProvenance, RenduRoot, RenduSpan,
};

use self::syntax::quote;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RenduDomMapping {
    pub generated_start: usize,
    pub generated_end: usize,
    pub source: RenduSpan,
    pub anchor: Option<SourceAnchor>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct RenduDomOutput {
    pub code: String,
    pub mappings: Vec<RenduDomMapping>,
}

/// Emit a Vue VDOM render module without reaching back into a frontend AST.
pub fn compile_rendu(root: &RenduRoot) -> RenduDomOutput {
    let mut emitter = DomEmitter {
        root,
        output: RenduDomOutput::default(),
    };
    emitter.output.code.push_str(
        "import { Fragment as _Fragment, createCommentVNode as _createCommentVNode, h as _h, renderList as _renderList, renderSlot as _renderSlot, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, toDisplayString as _toDisplayString, vModelText as _vModelText, vShow as _vShow, withDirectives as _withDirectives, withModifiers as _withModifiers } from \"vue\"\n\nexport function render(_ctx, _cache, $props, $setup, $data, $options) {\n  return ",
    );
    emitter.emit_nodes(root.entry());
    emitter.output.code.push_str("\n}\n");
    emitter.output
}

pub(super) struct DomEmitter<'a> {
    pub(super) root: &'a RenduRoot,
    pub(super) output: RenduDomOutput,
}

impl DomEmitter<'_> {
    pub(super) fn emit_nodes(&mut self, nodes: &[RenduNodeId]) {
        match nodes {
            [] => self.output.code.push_str("null"),
            [node] => self.emit_node(*node),
            _ => {
                self.output.code.push_str("_h(_Fragment, null, [");
                self.emit_node_list(nodes);
                self.output.code.push_str("])");
            }
        }
    }

    pub(super) fn emit_node_list(&mut self, nodes: &[RenduNodeId]) {
        for (index, node) in nodes.iter().enumerate() {
            if index > 0 {
                self.output.code.push_str(", ");
            }
            self.emit_node(*node);
        }
    }

    fn emit_node(&mut self, id: RenduNodeId) {
        let node = self.root.node(id).expect("validated Rendu node");
        let generated_start = self.output.code.len();
        match node {
            RenduNode::Fragment { children, .. } => {
                self.output.code.push_str("_h(_Fragment, null, [");
                self.emit_node_list(children);
                self.output.code.push_str("])");
            }
            RenduNode::Element {
                tag,
                properties,
                children,
                ..
            } => self.emit_vnode(tag, properties, children),
            RenduNode::Component {
                name,
                properties,
                children,
                ..
            } => {
                self.output.code.push_str("_h(");
                self.emit_component_name(name);
                self.output.code.push_str(", ");
                self.emit_properties(properties, true);
                self.output.code.push_str(", ");
                self.emit_component_slots(children);
                self.output.code.push(')');
                self.wrap_directives(generated_start, properties, true);
            }
            RenduNode::SlotOutlet {
                name,
                properties,
                fallback,
                ..
            } => {
                self.output.code.push_str("_renderSlot(_ctx.$slots, ");
                self.emit_name_value(name);
                self.output.code.push_str(", ");
                self.emit_properties(properties, false);
                self.output.code.push_str(", () => [");
                self.emit_node_list(fallback);
                self.output.code.push_str("])");
            }
            RenduNode::SlotContent { children, .. } => self.emit_nodes(children),
            RenduNode::Text { value, .. } => quote(&mut self.output.code, value),
            RenduNode::Expression {
                expression, escape, ..
            } => {
                if matches!(escape, RenduEscapeMode::Escaped) {
                    self.output.code.push_str("_toDisplayString(");
                }
                self.emit_expression(*expression);
                if matches!(escape, RenduEscapeMode::Escaped) {
                    self.output.code.push(')');
                }
            }
            RenduNode::Comment { value, .. } => {
                self.output.code.push_str("_createCommentVNode(");
                quote(&mut self.output.code, value);
                self.output.code.push(')');
            }
            RenduNode::If { branches, .. } => self.emit_if(branches),
            RenduNode::For {
                source,
                value,
                key,
                index,
                key_expression,
                body,
                ..
            } => {
                self.output.code.push_str("_renderList(");
                self.emit_expression(*source);
                self.output.code.push_str(", (");
                self.output.code.push_str(&value.pattern);
                if let Some(key) = key {
                    append!(self.output.code, ", {}", key.pattern);
                }
                if let Some(index) = index {
                    append!(self.output.code, ", {}", index.pattern);
                }
                self.output.code.push_str(") => ");
                if let Some(key) = key_expression {
                    self.output.code.push_str("_h(_Fragment, { key: ");
                    self.emit_expression(*key);
                    self.output.code.push_str(" }, [");
                    self.emit_node_list(body);
                    self.output.code.push_str("])");
                } else {
                    self.emit_nodes(body);
                }
                self.output.code.push(')');
            }
            RenduNode::HoistRef { index, .. } => {
                append!(self.output.code, "_ctx._hoisted?.[{index}] ?? null");
            }
            _ => unreachable!("RenduNode is non-exhaustive across backend crates"),
        }
        self.map(generated_start, node.provenance());
    }

    fn emit_vnode(&mut self, tag: &str, properties: &[RenduProperty], children: &[RenduNodeId]) {
        let start = self.output.code.len();
        self.output.code.push_str("_h(");
        quote(&mut self.output.code, tag);
        self.output.code.push_str(", ");
        self.emit_properties(properties, false);
        self.output.code.push_str(", [");
        self.emit_node_list(children);
        self.output.code.push_str("])");
        self.wrap_directives(start, properties, false);
    }

    fn emit_if(&mut self, branches: &[vize_rendu::RenduIfBranch]) {
        for (index, branch) in branches.iter().enumerate() {
            if let Some(condition) = branch.condition {
                self.output.code.push('(');
                self.emit_expression(condition);
                self.output.code.push_str(") ? ");
            }
            self.emit_nodes(&branch.body);
            if index + 1 < branches.len() {
                self.output.code.push_str(" : ");
            }
        }
        if branches
            .last()
            .is_some_and(|branch| branch.condition.is_some())
        {
            self.output.code.push_str(" : null");
        }
    }

    pub(super) fn map(&mut self, start: usize, provenance: &RenduProvenance) {
        let end = self.output.code.len();
        self.output
            .mappings
            .extend(provenance.spans().map(|source| {
                RenduDomMapping {
                    generated_start: start,
                    generated_end: end,
                    source,
                    anchor: self
                        .root
                        .source(source.source)
                        .and_then(|source| source.anchor()),
                }
            }));
    }
}

#[cfg(test)]
mod tests;
