use vize_rendu::{RenduComponentKind, RenduName, RenduProperty};

use super::{SsrEmitter, syntax::quote_js};

impl SsrEmitter<'_> {
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
                    quote_js(&mut self.output.code, name);
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
}
