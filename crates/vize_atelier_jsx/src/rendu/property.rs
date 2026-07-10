use vize_rendu::{
    RenduAttribute, RenduDirective, RenduExpression, RenduExpressionId, RenduExpressionKind,
    RenduName, RenduProperty,
};

use super::RenduLowerer;
use crate::syntax::{
    JsxSyntaxAttribute, JsxSyntaxAttributeValue, JsxSyntaxExpression, JsxSyntaxSpan,
};

impl RenduLowerer<'_> {
    pub(super) fn properties(&mut self, attributes: &[JsxSyntaxAttribute]) -> Vec<RenduProperty> {
        attributes
            .iter()
            .map(|attribute| self.property(attribute))
            .collect()
    }

    fn property(&mut self, attribute: &JsxSyntaxAttribute) -> RenduProperty {
        match attribute {
            JsxSyntaxAttribute::Spread { expression, span } => RenduProperty::Spread {
                expression: self.expression(expression),
                provenance: self.provenance(*span),
            },
            JsxSyntaxAttribute::Attribute {
                name,
                name_span,
                value,
                span,
            } => match directive_name(name) {
                Some(parts) => RenduProperty::Directive(self.directive(parts, value, *span)),
                None => RenduProperty::Attribute(self.attribute(name, *name_span, value, *span)),
            },
        }
    }

    fn attribute(
        &mut self,
        name: &str,
        name_span: JsxSyntaxSpan,
        value: &JsxSyntaxAttributeValue,
        span: JsxSyntaxSpan,
    ) -> RenduAttribute {
        let mut attribute = match value {
            JsxSyntaxAttributeValue::Presence => RenduAttribute::presence(name),
            JsxSyntaxAttributeValue::Static { value, .. } => {
                RenduAttribute::static_value(name, value.clone())
            }
            JsxSyntaxAttributeValue::Expression(expression) => {
                let expression = self.expression(expression);
                RenduAttribute::expression(name, expression)
            }
        };
        let mut provenance = self.provenance(span);
        if let Some(name) = self.provenance(name_span).primary {
            provenance.related.push(name);
        }
        attribute.provenance = provenance;
        attribute
    }

    fn directive(
        &mut self,
        parts: DirectiveName<'_>,
        value: &JsxSyntaxAttributeValue,
        span: JsxSyntaxSpan,
    ) -> RenduDirective {
        let mut directive = RenduDirective::new(parts.name);
        if let Some(argument) = parts.argument {
            directive = directive.with_argument(RenduName::static_name(argument));
        }
        if let Some(expression) = self.property_expression(value) {
            directive = directive.with_expression(expression);
        }
        for modifier in parts.modifiers {
            directive = directive.with_modifier(modifier);
        }
        directive.with_provenance(self.provenance(span))
    }

    fn property_expression(
        &mut self,
        value: &JsxSyntaxAttributeValue,
    ) -> Option<RenduExpressionId> {
        match value {
            JsxSyntaxAttributeValue::Presence => None,
            JsxSyntaxAttributeValue::Expression(expression) => Some(self.expression(expression)),
            JsxSyntaxAttributeValue::Static { value, span } => {
                let expression = JsxSyntaxExpression {
                    code: value.clone(),
                    span: *span,
                    synthetic: false,
                };
                let provenance = self.provenance(*span);
                Some(
                    self.builder.add_expression(
                        RenduExpression::new(expression.code, RenduExpressionKind::Literal)
                            .with_provenance(provenance),
                    ),
                )
            }
        }
    }
}

struct DirectiveName<'a> {
    name: &'a str,
    argument: Option<&'a str>,
    modifiers: Vec<&'a str>,
}

fn directive_name(raw: &str) -> Option<DirectiveName<'_>> {
    let raw = raw.strip_prefix("v-")?;
    let (head, argument) = raw
        .split_once(':')
        .map_or((raw, None), |(head, argument)| (head, Some(argument)));
    let mut segments = head.split('_');
    let name = segments.next()?;
    if name.is_empty() {
        return None;
    }
    Some(DirectiveName {
        name,
        argument,
        modifiers: segments.filter(|modifier| !modifier.is_empty()).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_directive_argument_and_modifiers() {
        let directive = directive_name("v-model_trim_lazy:value").expect("directive");
        assert_eq!(directive.name, "model");
        assert_eq!(directive.argument, Some("value"));
        assert_eq!(directive.modifiers, ["trim", "lazy"]);
        assert!(directive_name("onClick").is_none());
    }
}
