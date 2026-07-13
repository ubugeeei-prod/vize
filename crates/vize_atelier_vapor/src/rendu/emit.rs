//! JavaScript emission from the owned, frontend-neutral Vapor plan.

#[path = "emit/components.rs"]
mod components;
#[path = "emit/directive.rs"]
mod directive;
#[path = "emit/output.rs"]
mod output;
#[path = "emit/property.rs"]
mod property;

#[cfg(test)]
#[path = "emit/tests.rs"]
mod tests;

use vize_carton::{String, ToCompactString, appendln, cstr};
use vize_rendu::{RenderEmitSettings, RenderOutputMode, RenduEscapeMode, RenduRoot};

use super::{VaporBlockId, VaporBranch, VaporOperation, VaporPlan, VaporProperty, plan_rendu};
use property::{emit_element_properties, expression, quote_js, use_helper};

pub use output::VaporEmitResult;

/// Plan and emit a validated Rendu root without materializing a frontend AST.
pub fn emit_rendu(root: &RenduRoot) -> VaporEmitResult {
    emit_vapor_plan(&plan_rendu(root))
}

/// Emit executable Vue Vapor-shaped JavaScript from an owned Vapor plan.
pub fn emit_vapor_plan(plan: &VaporPlan) -> VaporEmitResult {
    emit_vapor_plan_with_settings(plan, &RenderEmitSettings::default())
}

pub fn emit_vapor_plan_with_settings(
    plan: &VaporPlan,
    settings: &RenderEmitSettings,
) -> VaporEmitResult {
    Emitter::new(plan).emit(settings)
}

struct Emitter<'a> {
    plan: &'a VaporPlan,
    templates: Vec<String>,
    helpers: Vec<&'static str>,
    next_node: usize,
}

impl<'a> Emitter<'a> {
    fn new(plan: &'a VaporPlan) -> Self {
        Self {
            plan,
            templates: Vec::new(),
            helpers: Vec::new(),
            next_node: 0,
        }
    }

    fn emit(mut self, settings: &RenderEmitSettings) -> VaporEmitResult {
        let (body, value) = self.emit_block(self.plan.entry(), 1);
        let preamble = output::preamble(&self.helpers, settings);
        let mut render = String::default();
        for (index, html) in self.templates.iter().enumerate() {
            appendln!(
                render,
                "const t",
                @index,
                " = _template(",
                quote_js(html).as_str(),
                ", true)"
            );
        }
        render.push_str(match settings.mode {
            RenderOutputMode::Module => {
                "export function render(_ctx = {}, _cache, $props, $setup, $data, $options) {\n"
            }
            RenderOutputMode::Function => {
                "return function render(_ctx = {}, _cache, $props, $setup, $data, $options) {\n"
            }
        });
        render.push_str("  const $slots = _ctx.$slots || {}\n");
        render.push_str("  const _hoisted = _ctx._hoisted || []\n");
        render.push_str(&body);
        appendln!(render, "  return ", value.as_str());
        appendln!(render, "}");
        let mut code = String::with_capacity(preamble.len() + render.len());
        code.push_str(&preamble);
        code.push_str(&render);
        VaporEmitResult {
            code,
            preamble,
            body: render,
            templates: self.templates,
        }
    }

    fn emit_block(&mut self, id: VaporBlockId, indent: usize) -> (String, String) {
        let operations = &self
            .plan
            .block(id)
            .expect("validated Vapor block")
            .operations;
        self.emit_operations(operations, indent)
    }

    fn emit_operations(
        &mut self,
        operations: &[VaporOperation],
        indent: usize,
    ) -> (String, String) {
        let mut body = String::default();
        let values = operations
            .iter()
            .map(|operation| self.emit_operation(operation, indent, &mut body))
            .collect::<Vec<_>>();
        let value = match values.as_slice() {
            [] => String::from("null"),
            [value] => value.clone(),
            _ => cstr!("[{}]", values.join(", ")),
        };
        (body, value)
    }

    fn emit_operation(
        &mut self,
        operation: &VaporOperation,
        indent: usize,
        out: &mut String,
    ) -> String {
        match operation {
            VaporOperation::StaticHtml { html, .. } => self.emit_template(html, indent, out),
            VaporOperation::Fragment { body, .. } => {
                let (statements, value) = self.emit_block(*body, indent);
                out.push_str(&statements);
                value
            }
            VaporOperation::Element {
                tag,
                properties,
                body,
                ..
            } => self.emit_element(tag, properties, *body, indent, out),
            VaporOperation::Component {
                name: component,
                properties,
                body,
                ..
            } => self.emit_component(component, properties, *body, indent, out),
            VaporOperation::SlotOutlet {
                name: slot,
                properties,
                fallback,
                ..
            } => self.emit_slot(slot, properties, *fallback, indent, out),
            VaporOperation::SlotContent { body, .. } => {
                let (statements, value) = self.emit_block(*body, indent);
                out.push_str(&statements);
                value
            }
            VaporOperation::DynamicText {
                expression: value,
                escape,
                ..
            } => self.emit_text(*value, *escape, indent, out),
            VaporOperation::Conditional { branches, .. } => {
                self.emit_conditional(branches, indent, out)
            }
            VaporOperation::Iterate {
                source,
                value,
                key,
                index,
                key_expression,
                body,
                ..
            } => {
                use_helper(&mut self.helpers, "createFor");
                let variable = self.node();
                let params = [Some(value), key.as_ref(), index.as_ref()]
                    .into_iter()
                    .flatten()
                    .map(|binding| binding.pattern.as_ref())
                    .collect::<Vec<_>>()
                    .join(", ");
                let callback = self.callback(*body, &params, indent);
                let key = key_expression
                    .map(|key| cstr!(", ({params}) => ({})", expression(self.plan, key)))
                    .unwrap_or_default();
                self.line(
                    out,
                    indent,
                    &cstr!(
                        "const {variable} = _createFor(() => ({}), {callback}{key})",
                        expression(self.plan, *source)
                    ),
                );
                variable
            }
            VaporOperation::HoistRef { index, .. } => {
                let variable = self.node();
                self.line(out, indent, &cstr!("const {variable} = _hoisted[{index}]"));
                variable
            }
            VaporOperation::Unsupported { description, .. } => {
                self.line(out, indent, &cstr!("/* unsupported: {description} */"));
                String::from("null")
            }
        }
    }

    fn emit_template(&mut self, html: &str, indent: usize, out: &mut String) -> String {
        use_helper(&mut self.helpers, "template");
        let template = self.templates.len();
        self.templates.push(html.to_compact_string());
        let variable = self.node();
        self.line(out, indent, &cstr!("const {variable} = t{template}()"));
        variable
    }

    fn emit_element(
        &mut self,
        tag: &str,
        properties: &[VaporProperty],
        body: VaporBlockId,
        indent: usize,
        out: &mut String,
    ) -> String {
        let variable = self.emit_template(&cstr!("<{tag}></{tag}>"), indent, out);
        emit_element_properties(
            self.plan,
            properties,
            &variable,
            tag,
            indent,
            out,
            &mut self.helpers,
        );
        let (children, value) = self.emit_block(body, indent);
        out.push_str(&children);
        if value != "null" {
            use_helper(&mut self.helpers, "insert");
            self.line(out, indent, &cstr!("_insert({variable}, {value})"));
        }
        variable
    }

    fn emit_text(
        &mut self,
        value: super::VaporExpressionId,
        escape: RenduEscapeMode,
        indent: usize,
        out: &mut String,
    ) -> String {
        let variable = self.emit_template(
            if matches!(escape, RenduEscapeMode::Escaped) {
                " "
            } else {
                "<span></span>"
            },
            indent,
            out,
        );
        use_helper(&mut self.helpers, "renderEffect");
        match escape {
            RenduEscapeMode::Escaped => {
                use_helper(&mut self.helpers, "setText");
                use_helper(&mut self.helpers, "toDisplayString");
                self.line(
                    out,
                    indent,
                    &cstr!(
                        "_renderEffect(() => _setText({variable}, _toDisplayString({})))",
                        expression(self.plan, value)
                    ),
                );
            }
            RenduEscapeMode::Raw => {
                use_helper(&mut self.helpers, "setProp");
                self.line(
                    out,
                    indent,
                    &cstr!(
                        "_renderEffect(() => _setProp({variable}, \"innerHTML\", {}))",
                        expression(self.plan, value)
                    ),
                );
            }
        }
        variable
    }

    fn emit_conditional(
        &mut self,
        branches: &[VaporBranch],
        indent: usize,
        out: &mut String,
    ) -> String {
        use_helper(&mut self.helpers, "createIf");
        let variable = self.node();
        let value = self.conditional_expression(branches, indent);
        self.line(out, indent, &cstr!("const {variable} = {value}"));
        variable
    }

    fn conditional_expression(&mut self, branches: &[VaporBranch], indent: usize) -> String {
        let Some((branch, rest)) = branches.split_first() else {
            return String::from("null");
        };
        let callback = self.callback(branch.body, "", indent);
        let Some(condition) = branch.condition else {
            return cstr!("({callback})()");
        };
        let fallback = if rest.is_empty() {
            String::default()
        } else {
            cstr!(", () => {}", self.conditional_expression(rest, indent + 1))
        };
        cstr!(
            "_createIf(() => ({}), {callback}{fallback})",
            expression(self.plan, condition)
        )
    }

    fn callback(&mut self, body: VaporBlockId, params: &str, indent: usize) -> String {
        let (statements, value) = self.emit_block(body, indent + 1);
        cstr!(
            "({params}) => {{\n{statements}{}return {value}\n{}}}",
            self.pad(indent + 1),
            self.pad(indent)
        )
    }

    fn node(&mut self) -> String {
        let node = cstr!("n{}", self.next_node);
        self.next_node += 1;
        node
    }

    fn line(&self, out: &mut String, indent: usize, line: &str) {
        appendln!(out, self.pad(indent).as_str(), line);
    }

    fn pad(&self, indent: usize) -> String {
        let mut pad = String::default();
        for _ in 0..indent {
            pad.push_str("  ");
        }
        pad
    }
}
