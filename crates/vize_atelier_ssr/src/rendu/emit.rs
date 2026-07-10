mod component;
mod element;
mod property;
mod syntax;

use vize_rendu::{RenduEscapeMode, RenduNode, RenduNodeId, RenduProvenance, RenduRoot};

use self::syntax::{escape_html_comment, escape_html_text, quote_js};
use super::{RenduSsrMapping, RenduSsrMappingKind, RenduSsrOutput};

pub(super) struct SsrEmitter<'a> {
    root: &'a RenduRoot,
    output: RenduSsrOutput,
    indent: usize,
}

impl<'a> SsrEmitter<'a> {
    pub(super) fn new(root: &'a RenduRoot) -> Self {
        Self {
            root,
            output: RenduSsrOutput::default(),
            indent: 0,
        }
    }

    pub(super) fn emit(mut self) -> RenduSsrOutput {
        self.output.code.push_str(
            "import { mergeProps as _mergeProps, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective } from \"vue\"\n",
        );
        self.output.code.push_str(
            "import { ssrGetDirectiveProps as _ssrGetDirectiveProps, ssrInterpolate as _ssrInterpolate, ssrRenderAttr as _ssrRenderAttr, ssrRenderAttrs as _ssrRenderAttrs, ssrRenderComponent as _ssrRenderComponent, ssrRenderDynamicAttr as _ssrRenderDynamicAttr, ssrRenderList as _ssrRenderList, ssrRenderSlot as _ssrRenderSlot } from \"vue/server-renderer\"\n\n",
        );
        self.line("export function ssrRender(_ctx, _push, _parent, _attrs) {");
        self.indent += 1;
        self.emit_nodes(self.root.entry());
        self.indent -= 1;
        self.line("}");
        self.output
    }

    pub(super) fn emit_nodes(&mut self, nodes: &[RenduNodeId]) {
        for &node in nodes {
            self.emit_node(node);
        }
    }

    fn emit_node(&mut self, id: RenduNodeId) {
        let node = self.root.node(id).expect("validated Rendu node");
        let start = self.output.code.len();
        match node {
            RenduNode::Fragment { children, .. } => self.emit_nodes(children),
            RenduNode::Element {
                tag,
                properties,
                children,
                ..
            } => self.emit_element(tag, properties, children),
            RenduNode::Component {
                name,
                properties,
                children,
                ..
            } => self.emit_component(name, properties, children),
            RenduNode::SlotOutlet {
                name,
                properties,
                fallback,
                ..
            } => self.emit_slot_outlet(name, properties, fallback),
            RenduNode::SlotContent { children, .. } => self.emit_nodes(children),
            RenduNode::Text { value, .. } => {
                self.push_line_value(&escape_html_text(value));
            }
            RenduNode::Expression {
                expression, escape, ..
            } => self.emit_interpolation(*expression, *escape),
            RenduNode::Comment { value, .. } => {
                self.push_line_value(&vize_carton::cstr!("<!--{}-->", escape_html_comment(value)));
            }
            RenduNode::If { branches, .. } => self.emit_if(branches),
            RenduNode::For {
                source,
                value,
                key,
                index,
                body,
                ..
            } => self.emit_for(*source, value, key.as_ref(), index.as_ref(), body),
            RenduNode::HoistRef { index, .. } => {
                self.indent();
                self.output.code.push_str("_push(_ssr_hoisted_");
                vize_carton::append!(self.output.code, "{index}");
                self.output.code.push_str(")\n");
            }
            _ => self.push_line_value("<!---->"),
        }
        self.map(start, node.provenance(), RenduSsrMappingKind::Node);
    }

    fn emit_interpolation(
        &mut self,
        expression: vize_rendu::RenduExpressionId,
        escape: RenduEscapeMode,
    ) {
        self.indent();
        self.output.code.push_str("_push(");
        if matches!(escape, RenduEscapeMode::Escaped) {
            self.output.code.push_str("_ssrInterpolate(");
        } else {
            self.output.code.push('(');
        }
        self.emit_expression(expression);
        if matches!(escape, RenduEscapeMode::Escaped) {
            self.output.code.push(')');
        } else {
            self.output.code.push_str(") ?? \"\"");
        }
        self.output.code.push_str(")\n");
    }

    fn emit_for(
        &mut self,
        source: vize_rendu::RenduExpressionId,
        value: &vize_rendu::RenduBinding,
        key: Option<&vize_rendu::RenduBinding>,
        index: Option<&vize_rendu::RenduBinding>,
        body: &[RenduNodeId],
    ) {
        self.indent();
        self.output.code.push_str("_ssrRenderList(");
        self.emit_expression(source);
        self.output.code.push_str(", (");
        self.emit_binding(value);
        for binding in [key, index].into_iter().flatten() {
            self.output.code.push_str(", ");
            self.emit_binding(binding);
        }
        self.output.code.push_str(") => {\n");
        self.indent += 1;
        self.emit_nodes(body);
        self.indent -= 1;
        self.line("})");
    }

    fn emit_if(&mut self, branches: &[vize_rendu::RenduIfBranch]) {
        for (index, branch) in branches.iter().enumerate() {
            let start = self.output.code.len();
            self.indent();
            match (index, branch.condition) {
                (0, Some(condition)) => {
                    self.output.code.push_str("if (");
                    self.emit_expression(condition);
                    self.output.code.push_str(") {\n");
                }
                (_, Some(condition)) => {
                    self.output.code.push_str("else if (");
                    self.emit_expression(condition);
                    self.output.code.push_str(") {\n");
                }
                (0, None) => self.output.code.push_str("{\n"),
                (_, None) => self.output.code.push_str("else {\n"),
            }
            self.indent += 1;
            self.emit_nodes(&branch.body);
            self.indent -= 1;
            self.line("}");
            self.map(start, &branch.provenance, RenduSsrMappingKind::Branch);
        }
        if branches
            .last()
            .is_some_and(|branch| branch.condition.is_some())
        {
            self.line("else {");
            self.indent += 1;
            self.push_line_value("<!---->");
            self.indent -= 1;
            self.line("}");
        }
    }

    pub(super) fn emit_expression(&mut self, id: vize_rendu::RenduExpressionId) {
        let expression = self
            .root
            .expression(id)
            .expect("validated Rendu expression");
        let start = self.output.code.len();
        self.output.code.push_str(&expression.code);
        self.map(
            start,
            &expression.provenance,
            RenduSsrMappingKind::Expression,
        );
    }

    pub(super) fn emit_binding(&mut self, binding: &vize_rendu::RenduBinding) {
        let start = self.output.code.len();
        self.output.code.push_str(&binding.pattern);
        self.map(start, &binding.provenance, RenduSsrMappingKind::Binding);
    }

    pub(super) fn push_line_value(&mut self, value: &str) {
        self.indent();
        self.output.code.push_str("_push(");
        quote_js(&mut self.output.code, value);
        self.output.code.push_str(")\n");
    }

    pub(super) fn indent(&mut self) {
        for _ in 0..self.indent {
            self.output.code.push_str("  ");
        }
    }

    pub(super) fn line(&mut self, line: &str) {
        self.indent();
        self.output.code.push_str(line);
        self.output.code.push('\n');
    }

    pub(super) fn map(
        &mut self,
        generated_start: usize,
        provenance: &RenduProvenance,
        kind: RenduSsrMappingKind,
    ) {
        let generated_end = self.output.code.len();
        self.output
            .mappings
            .extend(provenance.spans().map(|source| {
                RenduSsrMapping {
                    generated_start,
                    generated_end,
                    source,
                    anchor: self
                        .root
                        .source(source.source)
                        .and_then(|source| source.anchor()),
                    kind,
                }
            }));
    }
}
