use vize_rendu::{
    RenduAttributeValue, RenduDirective, RenduName, RenduNode, RenduNodeId, RenduProperty,
};

use super::DomEmitter;
use super::syntax::{comma, quote};

impl DomEmitter<'_> {
    pub(super) fn emit_properties(&mut self, properties: &[RenduProperty], component: bool) {
        self.output.code.push('{');
        let mut first = true;
        for property in properties {
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

    pub(super) fn emit_component_name(&mut self, name: &RenduName) {
        match name {
            RenduName::Static(name) => {
                self.output.code.push_str("_resolveComponent(");
                quote(&mut self.output.code, name);
                self.output.code.push(')');
            }
            RenduName::Dynamic(expression) => self.emit_expression(*expression),
        }
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

    pub(super) fn emit_component_slots(&mut self, children: &[RenduNodeId]) {
        self.output.code.push('{');
        let ordinary = children
            .iter()
            .copied()
            .filter(|id| {
                !matches!(
                    self.root.node(*id).expect("validated slot child"),
                    RenduNode::SlotContent { .. }
                )
            })
            .collect::<Vec<_>>();
        let mut first = true;
        if !ordinary.is_empty() {
            comma(&mut self.output.code, &mut first);
            quote(&mut self.output.code, "default");
            self.output.code.push_str(": () => [");
            self.emit_node_list(&ordinary);
            self.output.code.push(']');
        }
        for &child in children {
            let Some(RenduNode::SlotContent {
                name,
                bindings,
                children,
                ..
            }) = self.root.node(child)
            else {
                continue;
            };
            comma(&mut self.output.code, &mut first);
            self.emit_object_key(name);
            self.output.code.push_str(": (");
            for (index, binding) in bindings.iter().enumerate() {
                if index > 0 {
                    self.output.code.push_str(", ");
                }
                self.output.code.push_str(&binding.pattern);
            }
            self.output.code.push_str(") => [");
            self.emit_node_list(children);
            self.output.code.push(']');
        }
        self.output.code.push('}');
    }
}

fn is_runtime_directive(directive: &RenduDirective, component: bool) -> bool {
    match directive.name.as_ref() {
        "bind" | "on" | "html" | "text" => false,
        "model" => !component,
        _ => true,
    }
}
