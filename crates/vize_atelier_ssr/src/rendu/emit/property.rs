use vize_rendu::{
    RenduAttributeValue, RenduComponentKind, RenduDirective, RenduName, RenduProperty,
};

use super::{SsrEmitter, syntax::quote_js};

impl SsrEmitter<'_> {
    pub(super) fn emit_properties(&mut self, properties: &[RenduProperty]) {
        self.emit_properties_filtered(properties, false, false);
    }

    pub(super) fn emit_component_properties(
        &mut self,
        kind: RenduComponentKind,
        properties: &[RenduProperty],
    ) {
        self.emit_component_properties_with_fallthrough(kind, properties, false);
    }

    pub(super) fn emit_component_properties_with_fallthrough(
        &mut self,
        kind: RenduComponentKind,
        properties: &[RenduProperty],
        fallthrough: bool,
    ) {
        self.emit_properties_filtered(properties, kind == RenduComponentKind::Dynamic, fallthrough);
    }

    fn emit_properties_filtered(
        &mut self,
        properties: &[RenduProperty],
        consume_dynamic_is: bool,
        fallthrough: bool,
    ) {
        self.output.code.push_str("_mergeProps({");
        let mut first = true;
        for property in properties {
            if consume_dynamic_is && is_named_property(property, "is") {
                continue;
            }
            let start = self.output.code.len();
            let emitted = match property {
                RenduProperty::Attribute(attribute) => {
                    self.object_comma(&mut first);
                    self.emit_object_key(&attribute.name);
                    self.output.code.push_str(": ");
                    self.emit_attribute_value(attribute.value.as_ref());
                    true
                }
                RenduProperty::Directive(directive) => {
                    self.emit_component_directive_property(directive, &mut first)
                }
                RenduProperty::Spread { .. } => false,
            };
            if emitted {
                self.map(
                    start,
                    property.provenance(),
                    crate::rendu::RenduSsrMappingKind::Property,
                );
            }
        }
        self.output.code.push('}');
        for property in properties {
            let start = self.output.code.len();
            let emitted = match property {
                RenduProperty::Spread { expression, .. } => {
                    self.output.code.push_str(", ");
                    self.emit_expression(*expression);
                    true
                }
                RenduProperty::Directive(directive) if is_component_operand(directive) => {
                    self.output.code.push_str(", ");
                    if directive.name.as_ref() == "bind" {
                        self.emit_expression(
                            directive.expression.expect("v-bind operand has expression"),
                        );
                    } else {
                        self.output.code.push_str("_ssrGetDirectiveProps(_ctx, ");
                        self.emit_directive(directive);
                        self.output.code.push(')');
                    }
                    true
                }
                _ => false,
            };
            if emitted {
                self.map(
                    start,
                    property.provenance(),
                    crate::rendu::RenduSsrMappingKind::Property,
                );
            }
        }
        if fallthrough {
            self.output.code.push_str(", _attrs");
        }
        self.output.code.push(')');
    }

    fn emit_component_directive_property(
        &mut self,
        directive: &RenduDirective,
        first: &mut bool,
    ) -> bool {
        match directive.name.as_ref() {
            "bind" => {
                let (Some(argument), Some(expression)) =
                    (directive.argument.as_ref(), directive.expression)
                else {
                    return false;
                };
                self.object_comma(first);
                self.emit_object_key(argument);
                self.output.code.push_str(": ");
                self.emit_expression(expression);
            }
            "on" => {
                let (Some(argument), Some(expression)) =
                    (directive.argument.as_ref(), directive.expression)
                else {
                    return false;
                };
                self.object_comma(first);
                self.emit_event_key(argument);
                self.output.code.push_str(": ");
                if directive.modifiers.is_empty() {
                    self.emit_expression(expression);
                } else {
                    self.output.code.push_str("_withModifiers(");
                    self.emit_expression(expression);
                    self.output.code.push_str(", [");
                    self.emit_modifiers(&directive.modifiers);
                    self.output.code.push_str("])");
                }
            }
            "model" => {
                let Some(expression) = directive.expression else {
                    return false;
                };
                self.object_comma(first);
                quote_js(&mut self.output.code, "modelValue");
                self.output.code.push_str(": ");
                self.emit_expression(expression);
                self.object_comma(first);
                quote_js(&mut self.output.code, "onUpdate:modelValue");
                self.output.code.push_str(": $event => ((");
                self.emit_expression(expression);
                self.output.code.push_str(") = $event)");
            }
            "show" => {
                let Some(expression) = directive.expression else {
                    return false;
                };
                self.object_comma(first);
                quote_js(&mut self.output.code, "style");
                self.output.code.push_str(": (");
                self.emit_expression(expression);
                self.output
                    .code
                    .push_str(") ? null : { display: \"none\" }");
            }
            "html" | "text" => {
                let Some(expression) = directive.expression else {
                    return false;
                };
                self.object_comma(first);
                quote_js(
                    &mut self.output.code,
                    if directive.name.as_ref() == "html" {
                        "innerHTML"
                    } else {
                        "textContent"
                    },
                );
                self.output.code.push_str(": ");
                self.emit_expression(expression);
            }
            _ => return false,
        }
        true
    }

    pub(super) fn emit_directive(&mut self, directive: &RenduDirective) {
        self.output.code.push_str("_resolveDirective(");
        quote_js(&mut self.output.code, &directive.name);
        self.output.code.push_str("), ");
        if let Some(expression) = directive.expression {
            self.emit_expression(expression);
        } else {
            self.output.code.push_str("void 0");
        }
        self.output.code.push_str(", ");
        if let Some(argument) = &directive.argument {
            self.emit_name_value(argument);
        } else {
            self.output.code.push_str("void 0");
        }
        self.output.code.push_str(", {");
        for (index, modifier) in directive.modifiers.iter().enumerate() {
            if index > 0 {
                self.output.code.push_str(", ");
            }
            quote_js(&mut self.output.code, modifier);
            self.output.code.push_str(": true");
        }
        self.output.code.push('}');
    }

    pub(super) fn emit_name_value(&mut self, name: &RenduName) {
        match name {
            RenduName::Static(name) => quote_js(&mut self.output.code, name),
            RenduName::Dynamic(expression) => self.emit_expression(*expression),
        }
    }

    pub(super) fn emit_object_key(&mut self, name: &RenduName) {
        match name {
            RenduName::Static(name) => quote_js(&mut self.output.code, name),
            RenduName::Dynamic(expression) => {
                self.output.code.push('[');
                self.emit_expression(*expression);
                self.output.code.push(']');
            }
        }
    }

    pub(super) fn emit_attribute_value(&mut self, value: Option<&RenduAttributeValue>) {
        match value {
            None => self.output.code.push_str("true"),
            Some(RenduAttributeValue::Static(value)) => quote_js(&mut self.output.code, value),
            Some(RenduAttributeValue::Expression(expression)) => self.emit_expression(*expression),
        }
    }

    fn emit_event_key(&mut self, name: &RenduName) {
        match name {
            RenduName::Static(name) => {
                self.output.code.push('"');
                self.output.code.push_str("on");
                let mut chars = name.chars();
                if let Some(first) = chars.next() {
                    self.output.code.extend(first.to_uppercase());
                }
                self.output.code.extend(chars);
                self.output.code.push('"');
            }
            RenduName::Dynamic(expression) => {
                self.output.code.push_str("[\"on\" + (");
                self.emit_expression(*expression);
                self.output.code.push_str(")]");
            }
        }
    }

    fn emit_modifiers(&mut self, modifiers: &[Box<str>]) {
        for (index, modifier) in modifiers.iter().enumerate() {
            if index > 0 {
                self.output.code.push_str(", ");
            }
            quote_js(&mut self.output.code, modifier);
        }
    }

    fn object_comma(&mut self, first: &mut bool) {
        if *first {
            *first = false;
        } else {
            self.output.code.push_str(", ");
        }
    }
}

pub(super) fn is_named_property(property: &RenduProperty, name: &str) -> bool {
    match property {
        RenduProperty::Attribute(attribute) => {
            matches!(&attribute.name, RenduName::Static(key) if key.as_ref() == name)
        }
        RenduProperty::Directive(directive) => {
            directive.name.as_ref() == "bind"
                && matches!(&directive.argument, Some(RenduName::Static(key)) if key.as_ref() == name)
        }
        RenduProperty::Spread { .. } => false,
    }
}

fn is_component_operand(directive: &RenduDirective) -> bool {
    (directive.name.as_ref() == "bind"
        && directive.argument.is_none()
        && directive.expression.is_some())
        || !matches!(
            directive.name.as_ref(),
            "bind" | "on" | "model" | "show" | "html" | "text"
        )
}
