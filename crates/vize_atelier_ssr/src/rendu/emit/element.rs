use vize_rendu::{RenduAttributeValue, RenduDirective, RenduName, RenduNodeId, RenduProperty};

use super::{SsrEmitter, syntax::escape_html_attribute, syntax::quote_js};
use crate::rendu::RenduSsrMappingKind;

impl SsrEmitter<'_> {
    pub(super) fn emit_element(
        &mut self,
        tag: &str,
        properties: &[RenduProperty],
        children: &[RenduNodeId],
        fallthrough: bool,
    ) {
        self.push_line_value(&vize_carton::cstr!("<{tag}"));
        if fallthrough {
            self.emit_element_fallthrough_properties(properties);
        } else {
            for property in properties {
                self.emit_element_property(tag, property);
            }
        }
        if let Some(scope_id) = self.root.component_scope_id() {
            self.push_line_value(&vize_carton::cstr!(" {scope_id}"));
        }
        if self.slot_scope_depth > 0 {
            self.indent();
            self.output.code.push_str("_push(_scopeId)\n");
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

    fn emit_element_fallthrough_properties(&mut self, properties: &[RenduProperty]) {
        self.indent();
        self.output
            .code
            .push_str("_push(_ssrRenderAttrs(_mergeProps(");
        let mut wrote = false;
        for property in properties
            .iter()
            .filter(|property| is_fallthrough_operand(property))
        {
            if wrote {
                self.output.code.push_str(", ");
            }
            let start = self.output.code.len();
            self.emit_element_property_operand(property);
            self.map(start, property.provenance(), RenduSsrMappingKind::Property);
            wrote = true;
        }
        if wrote {
            self.output.code.push_str(", ");
        }
        self.output.code.push_str("_attrs)))\n");
    }

    fn emit_element_property_operand(&mut self, property: &RenduProperty) {
        match property {
            RenduProperty::Attribute(attribute) => {
                self.output.code.push('{');
                self.emit_object_key(&attribute.name);
                self.output.code.push_str(": ");
                self.emit_attribute_value(attribute.value.as_ref());
                self.output.code.push('}');
            }
            RenduProperty::Spread { expression, .. } => self.emit_expression(*expression),
            RenduProperty::Directive(directive) => match directive.name.as_ref() {
                "bind" => {
                    let expression = directive
                        .expression
                        .expect("filtered v-bind operand has an expression");
                    if let Some(argument) = directive.argument.as_ref() {
                        self.output.code.push('{');
                        self.emit_object_key(argument);
                        self.output.code.push_str(": ");
                        self.emit_expression(expression);
                        self.output.code.push('}');
                    } else {
                        self.emit_expression(expression);
                    }
                }
                "model" => {
                    self.output.code.push_str("{ value: ");
                    self.emit_expression(
                        directive
                            .expression
                            .expect("filtered v-model operand has an expression"),
                    );
                    self.output.code.push_str(" }");
                }
                "show" => {
                    self.output.code.push_str("{ style: (");
                    self.emit_expression(
                        directive
                            .expression
                            .expect("filtered v-show operand has an expression"),
                    );
                    self.output
                        .code
                        .push_str(") ? null : { display: \"none\" } }");
                }
                _ => {
                    self.output.code.push_str("_ssrGetDirectiveProps(_ctx, ");
                    self.emit_directive(directive);
                    self.output.code.push(')');
                }
            },
        }
    }

    pub(super) fn emit_element_property(&mut self, tag: &str, property: &RenduProperty) {
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

fn is_fallthrough_operand(property: &RenduProperty) -> bool {
    match property {
        RenduProperty::Attribute(_) | RenduProperty::Spread { .. } => true,
        RenduProperty::Directive(directive) => match directive.name.as_ref() {
            "on" | "html" | "text" => false,
            "bind" | "model" | "show" => directive.expression.is_some(),
            _ => true,
        },
    }
}
