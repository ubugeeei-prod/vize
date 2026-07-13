use vize_carton::BindingType;
use vize_relief::{SnapshotExpression, SnapshotProp};
use vize_rendu::{
    RenduAttribute, RenduAttributeValue, RenduDirective, RenduExpressionId, RenduName,
    RenduProperty,
};

use super::{
    super::{
        expression::add_rendu_expression_with_code, provenance::rendu_provenance,
        rendu_helpers::is_name_argument,
    },
    RenduLowerer,
};

impl RenduLowerer<'_> {
    pub(super) fn lower_properties(
        &mut self,
        properties: &[SnapshotProp],
        consumed: Option<usize>,
    ) -> Vec<RenduProperty> {
        properties
            .iter()
            .enumerate()
            .filter(|(index, _)| Some(*index) != consumed)
            .map(|(_, property)| self.lower_property(property))
            .collect()
    }

    fn lower_property(&mut self, property: &SnapshotProp) -> RenduProperty {
        match property {
            SnapshotProp::Attribute(attribute) => RenduProperty::Attribute(RenduAttribute {
                name: RenduName::static_name(attribute.name.as_str()),
                value: attribute
                    .value
                    .as_ref()
                    .map(|value| RenduAttributeValue::Static(value.content.as_str().into())),
                provenance: rendu_provenance(&attribute.location, self.source),
            }),
            SnapshotProp::Directive(directive) => {
                let mut lowered = RenduDirective::new(directive.name.as_str())
                    .with_provenance(rendu_provenance(&directive.location, self.source));
                if let Some(argument) = &directive.argument {
                    lowered = lowered.with_argument(self.add_name(argument));
                }
                if let Some(expression) = &directive.expression {
                    lowered = lowered.with_expression(
                        self.add_directive_expression(directive.name.as_str(), expression),
                    );
                }
                for modifier in &directive.modifiers {
                    lowered = lowered.with_modifier(modifier.content.as_str());
                }
                RenduProperty::Directive(lowered)
            }
        }
    }

    fn add_directive_expression(
        &mut self,
        directive: &str,
        expression: &SnapshotExpression,
    ) -> RenduExpressionId {
        if directive == "on"
            && let Some(name) = self.options_api_handler_name(expression)
        {
            let code = vize_carton::cstr!("(...args) => (_ctx.{name} && _ctx.{name}(...args))");
            return add_rendu_expression_with_code(
                &mut self.builder,
                expression,
                code.as_str(),
                self.source,
            );
        }
        self.add_expression(expression)
    }

    fn options_api_handler_name<'a>(&self, expression: &'a SnapshotExpression) -> Option<&'a str> {
        let SnapshotExpression::Simple(simple) = expression else {
            return None;
        };
        let name = simple
            .js_raw
            .as_deref()
            .unwrap_or(simple.location.source.as_str())
            .trim();
        if simple.is_static || !vize_carton::is_simple_identifier(name) {
            return None;
        }
        if self.scopes.iter().rev().any(|scope| scope.contains(name)) {
            return None;
        }
        self.bindings
            .and_then(|bindings| bindings.bindings.get(name))
            .filter(|binding| **binding == BindingType::Options)
            .map(|_| name)
    }

    pub(super) fn slot_outlet_name(
        &mut self,
        properties: &[SnapshotProp],
    ) -> (RenduName, Option<usize>) {
        for (index, property) in properties.iter().enumerate() {
            match property {
                SnapshotProp::Attribute(attribute) if attribute.name == "name" => {
                    let name = attribute
                        .value
                        .as_ref()
                        .map(|value| RenduName::static_name(value.content.as_str()))
                        .unwrap_or_else(|| RenduName::static_name("default"));
                    return (name, Some(index));
                }
                SnapshotProp::Directive(directive)
                    if directive.name == "bind"
                        && directive.argument.as_ref().is_some_and(is_name_argument) =>
                {
                    if let Some(expression) = &directive.expression {
                        return (
                            RenduName::Dynamic(self.add_expression(expression)),
                            Some(index),
                        );
                    }
                }
                _ => {}
            }
        }
        (RenduName::static_name("default"), None)
    }
}
