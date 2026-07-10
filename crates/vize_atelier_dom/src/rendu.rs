//! DOM/VDOM emission from the frontend-neutral Rendu artifact.

use std::fmt::Write as _;

use vize_carton::{String, source_anchor::SourceAnchor};
use vize_rendu::{
    RenduAttributeValue, RenduEscapeMode, RenduName, RenduNode, RenduNodeId, RenduProperty,
    RenduRoot, RenduSpan,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RenduDomMapping {
    pub generated_start: usize,
    pub generated_end: usize,
    pub source: RenduSpan,
    /// Stable compilation source identity behind the Rendu-local span.
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
        "import { Fragment as _Fragment, createCommentVNode as _createCommentVNode, h as _h, renderList as _renderList, renderSlot as _renderSlot, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, toDisplayString as _toDisplayString, withDirectives as _withDirectives } from \"vue\"\n\nexport function render(_ctx, _cache) {\n  return ",
    );
    emitter.emit_nodes(root.entry());
    emitter.output.code.push_str("\n}\n");
    emitter.output
}

struct DomEmitter<'a> {
    root: &'a RenduRoot,
    output: RenduDomOutput,
}

impl DomEmitter<'_> {
    fn emit_nodes(&mut self, nodes: &[RenduNodeId]) {
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

    fn emit_node_list(&mut self, nodes: &[RenduNodeId]) {
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
            } => self.emit_vnode(tag, false, properties, children),
            RenduNode::Component {
                name,
                properties,
                children,
                ..
            } => {
                self.output.code.push_str("_h(");
                self.emit_component_name(name);
                self.output.code.push_str(", ");
                self.emit_properties(properties);
                self.output.code.push_str(", [");
                self.emit_node_list(children);
                self.output.code.push_str("])");
                self.wrap_directives(generated_start, properties);
            }
            RenduNode::SlotOutlet {
                name,
                properties,
                fallback,
                ..
            } => {
                self.output.code.push_str("_renderSlot(_ctx.$slots, ");
                self.emit_name(name);
                self.output.code.push_str(", ");
                self.emit_properties(properties);
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
            RenduNode::If { branches, .. } => {
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
            RenduNode::For {
                source,
                value,
                key,
                index,
                body,
                ..
            } => {
                self.output.code.push_str("_renderList(");
                self.emit_expression(*source);
                self.output.code.push_str(", (");
                self.output.code.push_str(&value.pattern);
                if let Some(key) = key {
                    let _ = write!(self.output.code, ", {}", key.pattern);
                }
                if let Some(index) = index {
                    let _ = write!(self.output.code, ", {}", index.pattern);
                }
                self.output.code.push_str(") => ");
                self.emit_nodes(body);
                self.output.code.push(')');
            }
            RenduNode::HoistRef { index, .. } => {
                let _ = write!(self.output.code, "_hoisted_{index}");
            }
            _ => self
                .output
                .code
                .push_str("_createCommentVNode(\"unsupported Rendu node\")"),
        }
        if let Some(span) = node.provenance().primary {
            self.output.mappings.push(RenduDomMapping {
                generated_start,
                generated_end: self.output.code.len(),
                source: span,
                anchor: self
                    .root
                    .source(span.source)
                    .and_then(|source| source.anchor()),
            });
        }
    }

    fn emit_vnode(
        &mut self,
        tag: &str,
        _component: bool,
        properties: &[RenduProperty],
        children: &[RenduNodeId],
    ) {
        let start = self.output.code.len();
        self.output.code.push_str("_h(");
        quote(&mut self.output.code, tag);
        self.output.code.push_str(", ");
        self.emit_properties(properties);
        self.output.code.push_str(", [");
        self.emit_node_list(children);
        self.output.code.push_str("])");
        self.wrap_directives(start, properties);
    }

    fn wrap_directives(&mut self, vnode_start: usize, properties: &[RenduProperty]) {
        let directives: Vec<_> = properties
            .iter()
            .filter_map(|property| match property {
                RenduProperty::Directive(directive) => Some(directive),
                _ => None,
            })
            .collect();
        if directives.is_empty() {
            return;
        }
        const PREFIX: &str = "_withDirectives(";
        self.output.code.insert_str(vnode_start, PREFIX);
        for mapping in &mut self.output.mappings {
            if mapping.generated_start >= vnode_start {
                mapping.generated_start += PREFIX.len();
                mapping.generated_end += PREFIX.len();
            }
        }
        self.output.code.push_str(", [");
        for (index, directive) in directives.iter().enumerate() {
            if index > 0 {
                self.output.code.push_str(", ");
            }
            self.output.code.push_str("[_resolveDirective(");
            quote(&mut self.output.code, &directive.name);
            self.output.code.push(')');
            if let Some(expression) = directive.expression {
                self.output.code.push_str(", ");
                self.emit_expression(expression);
            }
            self.output.code.push(']');
        }
        self.output.code.push_str("])");
    }

    fn emit_properties(&mut self, properties: &[RenduProperty]) {
        self.output.code.push_str("{");
        let mut first = true;
        for property in properties {
            match property {
                RenduProperty::Attribute(attribute) => {
                    comma(&mut self.output.code, &mut first);
                    self.emit_name(&attribute.name);
                    self.output.code.push_str(": ");
                    match &attribute.value {
                        None => self.output.code.push_str("true"),
                        Some(RenduAttributeValue::Static(value)) => {
                            quote(&mut self.output.code, value)
                        }
                        Some(RenduAttributeValue::Expression(expression)) => {
                            self.emit_expression(*expression)
                        }
                    }
                }
                RenduProperty::Spread { expression, .. } => {
                    comma(&mut self.output.code, &mut first);
                    self.output.code.push_str("...");
                    self.emit_expression(*expression);
                }
                RenduProperty::Directive(_) => {}
            }
        }
        self.output.code.push('}');
    }

    fn emit_component_name(&mut self, name: &RenduName) {
        match name {
            RenduName::Static(name) => {
                self.output.code.push_str("_resolveComponent(");
                quote(&mut self.output.code, name);
                self.output.code.push(')');
            }
            RenduName::Dynamic(expression) => self.emit_expression(*expression),
        }
    }

    fn emit_name(&mut self, name: &RenduName) {
        match name {
            RenduName::Static(name) => quote(&mut self.output.code, name),
            RenduName::Dynamic(expression) => {
                self.output.code.push('[');
                self.emit_expression(*expression);
                self.output.code.push(']');
            }
        }
    }

    fn emit_expression(&mut self, id: vize_rendu::RenduExpressionId) {
        self.output.code.push_str(
            &self
                .root
                .expression(id)
                .expect("validated Rendu expression")
                .code,
        );
    }
}

fn comma(output: &mut String, first: &mut bool) {
    if *first {
        *first = false;
    } else {
        output.push_str(", ");
    }
}

fn quote(output: &mut String, value: &str) {
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character => output.push(character),
        }
    }
    output.push('"');
}

#[cfg(test)]
mod tests;
