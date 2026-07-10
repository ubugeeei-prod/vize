use vize_rendu::{RenduAttributeValue, RenduDirective, RenduName, RenduProperty};

use super::{SsrEmitter, syntax::quote_js};

impl SsrEmitter<'_> {
    pub(super) fn emit_properties(&mut self, properties: &[RenduProperty]) {
        let attributes = properties
            .iter()
            .filter(|property| matches!(property, RenduProperty::Attribute(_)))
            .count();
        let operands = usize::from(attributes > 0 || properties.is_empty())
            + properties
                .iter()
                .filter(|property| !matches!(property, RenduProperty::Attribute(_)))
                .count();
        if operands > 1 {
            self.output.code.push_str("_mergeProps(");
        }
        let mut first_operand = true;
        if attributes > 0 || properties.is_empty() {
            self.output.code.push('{');
            let mut first = true;
            for property in properties {
                let RenduProperty::Attribute(attribute) = property else {
                    continue;
                };
                let start = self.output.code.len();
                if !first {
                    self.output.code.push_str(", ");
                }
                first = false;
                self.emit_name(&attribute.name);
                self.output.code.push_str(": ");
                self.emit_attribute_value(attribute.value.as_ref());
                self.map(
                    start,
                    property.provenance(),
                    crate::rendu::RenduSsrMappingKind::Property,
                );
            }
            self.output.code.push('}');
            first_operand = false;
        }
        for property in properties {
            if matches!(property, RenduProperty::Attribute(_)) {
                continue;
            }
            let start = self.output.code.len();
            if !first_operand {
                self.output.code.push_str(", ");
            }
            first_operand = false;
            match property {
                RenduProperty::Spread { expression, .. } => self.emit_expression(*expression),
                RenduProperty::Directive(directive) => {
                    self.output.code.push_str("_ssrGetDirectiveProps(_ctx, ");
                    self.emit_directive(directive);
                    self.output.code.push(')');
                }
                RenduProperty::Attribute(_) => unreachable!(),
            }
            self.map(
                start,
                property.provenance(),
                crate::rendu::RenduSsrMappingKind::Property,
            );
        }
        if operands > 1 {
            self.output.code.push(')');
        }
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
            self.emit_name(argument);
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

    pub(super) fn emit_component_name(&mut self, name: &RenduName) {
        match name {
            RenduName::Static(name) => {
                self.output.code.push_str("_resolveComponent(");
                quote_js(&mut self.output.code, name);
                self.output.code.push(')');
            }
            RenduName::Dynamic(expression) => self.emit_expression(*expression),
        }
    }

    pub(super) fn emit_name(&mut self, name: &RenduName) {
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
}
