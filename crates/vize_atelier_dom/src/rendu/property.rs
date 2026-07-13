use vize_rendu::{
    RenduAttributeValue, RenduComponentKind, RenduDirective, RenduName, RenduProperty,
};

use super::DomEmitter;
use super::syntax::{comma, quote};

impl DomEmitter<'_> {
    pub(super) fn emit_properties(&mut self, properties: &[RenduProperty], component: bool) {
        self.emit_properties_filtered(properties, component, false);
    }

    pub(super) fn emit_component_properties(
        &mut self,
        kind: RenduComponentKind,
        properties: &[RenduProperty],
    ) {
        self.emit_properties_filtered(properties, true, kind == RenduComponentKind::Dynamic);
    }

    fn emit_properties_filtered(
        &mut self,
        properties: &[RenduProperty],
        component: bool,
        consume_dynamic_is: bool,
    ) {
        self.output.code.push('{');
        let mut first = true;
        for property in properties {
            if consume_dynamic_is && is_named_property(property, "is") {
                continue;
            }
            let start = self.output.code.len();
            match property {
                RenduProperty::Attribute(attribute) => {
                    comma(&mut self.output.code, &mut first);
                    self.emit_object_key(&attribute.name);
                    self.output.code.push_str(": ");
                    self.emit_attribute_value(attribute.value.as_ref());
                }
                RenduProperty::Spread { expression, .. } => {
                    comma(&mut self.output.code, &mut first);
                    self.output.code.push_str("...");
                    self.emit_expression(*expression);
                }
                RenduProperty::Directive(directive) => {
                    self.emit_directive_property(directive, component, &mut first);
                }
            }
            self.map(start, property.provenance());
        }
        self.output.code.push('}');
    }

    fn emit_directive_property(
        &mut self,
        directive: &RenduDirective,
        component: bool,
        first: &mut bool,
    ) {
        match directive.name.as_ref() {
            "bind" => self.emit_bind(directive, first),
            "on" => self.emit_on(directive, first),
            "model" => self.emit_model(directive, component, first),
            "html" => self.emit_content_prop("innerHTML", directive, first),
            "text" => self.emit_content_prop("textContent", directive, first),
            _ => {}
        }
    }

    fn emit_bind(&mut self, directive: &RenduDirective, first: &mut bool) {
        let Some(expression) = directive.expression else {
            return;
        };
        comma(&mut self.output.code, first);
        if let Some(argument) = directive.argument.as_ref() {
            self.emit_object_key(argument);
            self.output.code.push_str(": ");
        } else {
            self.output.code.push_str("...");
        }
        self.emit_expression(expression);
    }

    fn emit_on(&mut self, directive: &RenduDirective, first: &mut bool) {
        let (Some(argument), Some(expression)) =
            (directive.argument.as_ref(), directive.expression)
        else {
            return;
        };
        comma(&mut self.output.code, first);
        match argument {
            RenduName::Static(name) => {
                self.output.code.push('"');
                self.output.code.push_str("on");
                let mut characters = name.chars();
                if let Some(first) = characters.next() {
                    self.output.code.extend(first.to_uppercase());
                }
                self.output.code.extend(characters);
                self.output.code.push_str("\": ");
            }
            RenduName::Dynamic(name) => {
                self.output.code.push_str("[\"on\" + (");
                self.emit_expression(*name);
                self.output.code.push_str(")]: ");
            }
        }
        if directive.modifiers.is_empty() {
            self.emit_expression(expression);
        } else {
            self.output.code.push_str("_withModifiers(");
            self.emit_expression(expression);
            self.output.code.push_str(", [");
            for (index, modifier) in directive.modifiers.iter().enumerate() {
                if index > 0 {
                    self.output.code.push_str(", ");
                }
                quote(&mut self.output.code, modifier);
            }
            self.output.code.push_str("])");
        }
    }

    fn emit_model(&mut self, directive: &RenduDirective, component: bool, first: &mut bool) {
        let Some(expression) = directive.expression else {
            return;
        };
        comma(&mut self.output.code, first);
        quote(
            &mut self.output.code,
            if component { "modelValue" } else { "value" },
        );
        self.output.code.push_str(": ");
        self.emit_expression(expression);
        if component {
            comma(&mut self.output.code, first);
            quote(&mut self.output.code, "onUpdate:modelValue");
            self.output.code.push_str(": $event => ((");
            self.emit_expression(expression);
            self.output.code.push_str(") = $event)");
        }
    }

    fn emit_content_prop(&mut self, name: &str, directive: &RenduDirective, first: &mut bool) {
        let Some(expression) = directive.expression else {
            return;
        };
        comma(&mut self.output.code, first);
        quote(&mut self.output.code, name);
        self.output.code.push_str(": ");
        self.emit_expression(expression);
    }

    pub(super) fn wrap_directives(
        &mut self,
        vnode_start: usize,
        properties: &[RenduProperty],
        component: bool,
    ) {
        let directives = properties.iter().filter_map(|property| match property {
            RenduProperty::Directive(directive) if is_runtime_directive(directive, component) => {
                Some(directive)
            }
            _ => None,
        });
        let directives = directives.collect::<Vec<_>>();
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
            self.emit_runtime_directive(directive);
        }
        self.output.code.push_str("])");
    }

    fn emit_runtime_directive(&mut self, directive: &RenduDirective) {
        self.output.code.push('[');
        match directive.name.as_ref() {
            "show" => self.output.code.push_str("_vShow"),
            "model" => self.output.code.push_str("_vModelText"),
            _ => {
                self.output.code.push_str("_resolveDirective(");
                quote(&mut self.output.code, &directive.name);
                self.output.code.push(')');
            }
        }
        self.output.code.push_str(", ");
        if let Some(expression) = directive.expression {
            self.emit_expression(expression);
        } else {
            self.output.code.push_str("void 0");
        }
        self.output.code.push_str(", ");
        if let Some(argument) = directive.argument.as_ref() {
            self.emit_name_value(argument);
        } else {
            self.output.code.push_str("void 0");
        }
        self.output.code.push_str(", {");
        for (index, modifier) in directive.modifiers.iter().enumerate() {
            if index > 0 {
                self.output.code.push_str(", ");
            }
            quote(&mut self.output.code, modifier);
            self.output.code.push_str(": true");
        }
        self.output.code.push_str("}]");
    }

    pub(super) fn emit_component_name(
        &mut self,
        kind: RenduComponentKind,
        name: &RenduName,
        properties: &[RenduProperty],
    ) {
        match kind {
            RenduComponentKind::Ordinary => match name {
                RenduName::Static(name) => {
                    self.output.code.push_str("_resolveComponent(");
                    quote(&mut self.output.code, name);
                    self.output.code.push(')');
                }
                RenduName::Dynamic(expression) => self.emit_expression(*expression),
            },
            RenduComponentKind::Suspense => self.output.code.push_str("_Suspense"),
            RenduComponentKind::Teleport => self.output.code.push_str("_Teleport"),
            RenduComponentKind::KeepAlive => self.output.code.push_str("_KeepAlive"),
            RenduComponentKind::Transition => {
                if matches!(name, RenduName::Static(name) if name.as_ref().eq_ignore_ascii_case("BaseTransition") || name.as_ref() == "base-transition")
                {
                    self.output.code.push_str("_BaseTransition");
                } else {
                    self.output.code.push_str("_Transition");
                }
            }
            RenduComponentKind::TransitionGroup => {
                self.output.code.push_str("_TransitionGroup");
            }
            RenduComponentKind::Dynamic => {
                self.output.code.push_str("_resolveDynamicComponent(");
                self.emit_named_property_value(properties, "is", "null");
                self.output.code.push(')');
            }
        }
    }

    fn emit_named_property_value(
        &mut self,
        properties: &[RenduProperty],
        name: &str,
        fallback: &str,
    ) {
        for property in properties {
            match property {
                RenduProperty::Attribute(attribute) if matches!(&attribute.name, RenduName::Static(key) if key.as_ref() == name) =>
                {
                    self.emit_attribute_value(attribute.value.as_ref());
                    return;
                }
                RenduProperty::Directive(directive)
                    if directive.name.as_ref() == "bind"
                        && matches!(&directive.argument, Some(RenduName::Static(key)) if key.as_ref() == name) =>
                {
                    if let Some(expression) = directive.expression {
                        self.emit_expression(expression);
                        return;
                    }
                }
                _ => {}
            }
        }
        self.output.code.push_str(fallback);
    }

    pub(super) fn emit_name_value(&mut self, name: &RenduName) {
        match name {
            RenduName::Static(name) => quote(&mut self.output.code, name),
            RenduName::Dynamic(expression) => self.emit_expression(*expression),
        }
    }

    fn emit_object_key(&mut self, name: &RenduName) {
        match name {
            RenduName::Static(name) if vize_carton::is_simple_identifier(name) => {
                self.output.code.push_str(name);
            }
            RenduName::Static(name) => quote(&mut self.output.code, name),
            RenduName::Dynamic(expression) => {
                self.output.code.push('[');
                self.emit_expression(*expression);
                self.output.code.push(']');
            }
        }
    }

    pub(super) fn emit_expression(&mut self, id: vize_rendu::RenduExpressionId) {
        let expression = self
            .root
            .expression(id)
            .expect("validated Rendu expression");
        let start = self.output.code.len();
        self.output.code.push_str(&expression.code);
        self.map(start, &expression.provenance);
    }

    fn emit_attribute_value(&mut self, value: Option<&RenduAttributeValue>) {
        match value {
            None => self.output.code.push_str("true"),
            Some(RenduAttributeValue::Static(value)) => quote(&mut self.output.code, value),
            Some(RenduAttributeValue::Expression(expression)) => self.emit_expression(*expression),
        }
    }
}

fn is_named_property(property: &RenduProperty, name: &str) -> bool {
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

fn is_runtime_directive(directive: &RenduDirective, component: bool) -> bool {
    match directive.name.as_ref() {
        "bind" | "on" | "html" | "text" => false,
        "model" => !component,
        _ => true,
    }
}
