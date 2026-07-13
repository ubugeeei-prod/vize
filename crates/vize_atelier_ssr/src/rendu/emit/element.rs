use vize_rendu::{RenduAttributeValue, RenduDirective, RenduName, RenduNodeId, RenduProperty};

use super::{SsrEmitter, syntax::escape_html_attribute, syntax::quote_js};
use crate::rendu::RenduSsrMappingKind;

impl SsrEmitter<'_> {
    pub(super) fn emit_element(
        &mut self,
        tag: &str,
        properties: &[RenduProperty],
        children: &[RenduNodeId],
    ) {
        self.push_line_value(&vize_carton::cstr!("<{tag}"));
        for property in properties {
            self.emit_element_property(tag, property);
        }
        self.push_line_value(">");
        if !vize_carton::is_void_tag(tag) {
            if let Some(directive) = content_directive(properties, "html") {
                self.emit_raw_directive_content(directive);
            } else if let Some(directive) = content_directive(properties, "text") {
                self.emit_text_directive_content(directive);
            } else {
                self.emit_nodes(children);
            }
            self.push_line_value(&vize_carton::cstr!("</{tag}>"));
        }
    }

    fn emit_element_property(&mut self, tag: &str, property: &RenduProperty) {
        let start = self.output.code.len();
        match property {
            RenduProperty::Attribute(attribute) => match (&attribute.name, &attribute.value) {
                (RenduName::Static(name), None) => {
                    self.push_line_value(&vize_carton::cstr!(" {name}"));
                }
                (RenduName::Static(name), Some(RenduAttributeValue::Static(value))) => {
                    self.push_line_value(&vize_carton::cstr!(
                        " {name}=\"{}\"",
                        escape_html_attribute(value)
                    ));
                }
                (RenduName::Static(name), Some(RenduAttributeValue::Expression(expression))) => {
                    self.indent();
                    self.output.code.push_str("_push(_ssrRenderAttr(");
                    quote_js(&mut self.output.code, name);
                    self.output.code.push_str(", ");
                    self.emit_expression(*expression);
                    self.output.code.push_str("))\n");
                }
                (RenduName::Dynamic(name), value) => {
                    self.indent();
                    self.output.code.push_str("_push(_ssrRenderDynamicAttr(");
                    self.emit_expression(*name);
                    self.output.code.push_str(", ");
                    self.emit_attribute_value(value.as_ref());
                    self.output.code.push_str(", ");
                    quote_js(&mut self.output.code, tag);
                    self.output.code.push_str("))\n");
                }
            },
            RenduProperty::Spread { expression, .. } => {
                self.indent();
                self.output.code.push_str("_push(_ssrRenderAttrs(");
                self.emit_expression(*expression);
                self.output.code.push_str("))\n");
            }
            RenduProperty::Directive(directive) => self.emit_element_directive(tag, directive),
        }
        self.map(start, property.provenance(), RenduSsrMappingKind::Property);
    }

    fn emit_element_directive(&mut self, tag: &str, directive: &RenduDirective) {
        match directive.name.as_ref() {
            "html" | "text" | "on" => {}
            "bind" => self.emit_bind_directive(tag, directive),
            "model" => {
                let Some(expression) = directive.expression else {
                    return;
                };
                self.indent();
                self.output
                    .code
                    .push_str("_push(_ssrRenderAttr(\"value\", ");
                self.emit_expression(expression);
                self.output.code.push_str("))\n");
            }
            "show" => {
                let Some(expression) = directive.expression else {
                    return;
                };
                self.indent();
                self.output.code.push_str("_push((");
                self.emit_expression(expression);
                self.output
                    .code
                    .push_str(") ? \"\" : \" style=\\\"display: none;\\\"\")\n");
            }
            _ => {
                self.indent();
                self.output
                    .code
                    .push_str("_push(_ssrRenderAttrs(_ssrGetDirectiveProps(_ctx, ");
                self.emit_directive(directive);
                self.output.code.push_str(")))\n");
            }
        }
    }

    fn emit_bind_directive(&mut self, tag: &str, directive: &RenduDirective) {
        let Some(expression) = directive.expression else {
            return;
        };
        self.indent();
        match directive.argument.as_ref() {
            None => {
                self.output.code.push_str("_push(_ssrRenderAttrs(");
                self.emit_expression(expression);
                self.output.code.push_str("))\n");
            }
            Some(RenduName::Static(name)) => {
                self.output.code.push_str("_push(_ssrRenderAttr(");
                quote_js(&mut self.output.code, name);
                self.output.code.push_str(", ");
                self.emit_expression(expression);
                self.output.code.push_str("))\n");
            }
            Some(RenduName::Dynamic(name)) => {
                self.output.code.push_str("_push(_ssrRenderDynamicAttr(");
                self.emit_expression(*name);
                self.output.code.push_str(", ");
                self.emit_expression(expression);
                self.output.code.push_str(", ");
                quote_js(&mut self.output.code, tag);
                self.output.code.push_str("))\n");
            }
        }
    }

    fn emit_raw_directive_content(&mut self, directive: &RenduDirective) {
        let Some(expression) = directive.expression else {
            return;
        };
        self.indent();
        self.output.code.push_str("_push((");
        self.emit_expression(expression);
        self.output.code.push_str(") ?? \"\")\n");
    }

    fn emit_text_directive_content(&mut self, directive: &RenduDirective) {
        let Some(expression) = directive.expression else {
            return;
        };
        self.indent();
        self.output.code.push_str("_push(_ssrInterpolate(");
        self.emit_expression(expression);
        self.output.code.push_str("))\n");
    }
}

fn content_directive<'a>(
    properties: &'a [RenduProperty],
    name: &str,
) -> Option<&'a RenduDirective> {
    properties.iter().find_map(|property| match property {
        RenduProperty::Directive(directive) if directive.name.as_ref() == name => Some(directive),
        _ => None,
    })
}
