//! DOM/VDOM emission from the frontend-neutral Rendu artifact.

mod property;
mod slot;
mod syntax;

use vize_carton::{String, append, source_anchor::SourceAnchor};
use vize_rendu::{
    RenderEmitSettings, RenderOutputMode, RenduEscapeMode, RenduNode, RenduNodeId, RenduProperty,
    RenduProvenance, RenduRoot, RenduSpan,
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
    /// Complete executable module/function source.
    pub code: String,
    /// Typed helper-import or runtime-global section.
    pub preamble: String,
    /// Typed render-function body section.
    pub body: String,
    pub mappings: Vec<RenduDomMapping>,
}

/// Emit a Vue VDOM render module without reaching back into a frontend AST.
pub fn compile_rendu(root: &RenduRoot) -> RenduDomOutput {
    compile_rendu_with_settings(root, &RenderEmitSettings::default())
}

/// Emit a Vue VDOM render output with explicit packaging settings.
pub fn compile_rendu_with_settings(
    root: &RenduRoot,
    settings: &RenderEmitSettings,
) -> RenduDomOutput {
    let mut emitter = DomEmitter {
        root,
        output: RenduDomOutput::default(),
    };
    emitter.output.code.push_str(match settings.mode {
        RenderOutputMode::Module => {
            "export function render(_ctx, _cache, $props, $setup, $data, $options) {\n  return "
        }
        RenderOutputMode::Function => {
            "return function render(_ctx, _cache, $props, $setup, $data, $options) {\n  return "
        }
    });
    emitter.emit_nodes(root.entry());
    emitter.output.code.push_str("\n}\n");
    finish_output(emitter.output, settings)
}

const DOM_HELPERS: &str = "BaseTransition as _BaseTransition, Fragment as _Fragment, KeepAlive as _KeepAlive, Suspense as _Suspense, Teleport as _Teleport, Transition as _Transition, TransitionGroup as _TransitionGroup, createCommentVNode as _createCommentVNode, createSlots as _createSlots, h as _h, renderList as _renderList, renderSlot as _renderSlot, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, resolveDynamicComponent as _resolveDynamicComponent, toDisplayString as _toDisplayString, vModelText as _vModelText, vShow as _vShow, withCtx as _withCtx, withDirectives as _withDirectives, withModifiers as _withModifiers";

fn finish_output(mut output: RenduDomOutput, settings: &RenderEmitSettings) -> RenduDomOutput {
    let body = output.code;
    let preamble = match settings.mode {
        RenderOutputMode::Module => vize_carton::cstr!(
            "import {{ {DOM_HELPERS} }} from \"{}\"\n\n",
            settings.runtime_module_name
        ),
        RenderOutputMode::Function => vize_carton::cstr!(
            "const {{ {} }} = {}\n\n",
            DOM_HELPERS.replace(" as ", ": "),
            settings.runtime_global_name
        ),
    };
    let offset = preamble.len();
    for mapping in &mut output.mappings {
        mapping.generated_start = mapping.generated_start.saturating_add(offset);
        mapping.generated_end = mapping.generated_end.saturating_add(offset);
    }
    let mut code = String::with_capacity(offset + body.len());
    code.push_str(&preamble);
    code.push_str(&body);
    RenduDomOutput {
        code,
        preamble,
        body,
        mappings: output.mappings,
    }
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
                kind,
                name,
                properties,
                children,
                ..
            } => {
                self.output.code.push_str("_h(");
                self.emit_component_name(*kind, name, properties);
                self.output.code.push_str(", ");
                self.emit_component_properties(*kind, properties);
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
