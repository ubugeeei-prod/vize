use vize_carton::{BindingMetadata, String, camelize, capitalize};
use vize_relief::{SnapshotElement, SnapshotExpression, SnapshotProp};
use vize_rendu::{RenduComponentKind, RenduExpression, RenduExpressionKind, RenduName};

use super::{super::provenance::rendu_provenance, RenduLowerer};

impl<'a> RenduLowerer<'a> {
    pub(super) fn component_name(&mut self, element: &SnapshotElement) -> RenduName {
        let tag = element.tag.as_str();
        let Some((binding, suffix)) = self.component_binding(tag) else {
            return RenduName::static_name(tag);
        };
        let mut expression = String::new("$setup.");
        expression.push_str(binding);
        expression.push_str(suffix);
        let id = self.builder.add_expression(
            RenduExpression::new(expression.as_str(), RenduExpressionKind::Reference)
                .with_provenance(rendu_provenance(&element.location, self.source)),
        );
        RenduName::Dynamic(id)
    }

    fn component_binding<'b>(&'b self, tag: &'b str) -> Option<(&'b str, &'b str)> {
        let bindings = self.bindings?;
        if !bindings.is_script_setup {
            return None;
        }
        let split = tag.find('.').unwrap_or(tag.len());
        let root = &tag[..split];
        let suffix = &tag[split..];
        if let Some(binding) = setup_component_binding_key(bindings, root) {
            return Some((binding, suffix));
        }

        let camel = camelize(root);
        if let Some(binding) = setup_component_binding_key(bindings, camel.as_str()) {
            return Some((binding, suffix));
        }
        let pascal = capitalize(camel.as_str());
        setup_component_binding_key(bindings, pascal.as_str()).map(|binding| (binding, suffix))
    }
}

pub(super) fn component_kind(element: &SnapshotElement) -> RenduComponentKind {
    match element.tag.as_str() {
        "Suspense" | "suspense" => RenduComponentKind::Suspense,
        "Teleport" | "teleport" => RenduComponentKind::Teleport,
        "KeepAlive" | "keep-alive" => RenduComponentKind::KeepAlive,
        "BaseTransition" | "base-transition" | "Transition" | "transition" => {
            RenduComponentKind::Transition
        }
        "TransitionGroup" | "transition-group" => RenduComponentKind::TransitionGroup,
        "component" if has_explicit_dynamic_component_target(&element.props) => {
            RenduComponentKind::Dynamic
        }
        _ => RenduComponentKind::Ordinary,
    }
}

fn has_explicit_dynamic_component_target(properties: &[SnapshotProp]) -> bool {
    properties.iter().any(|property| match property {
        SnapshotProp::Attribute(attribute) => attribute.name == "is",
        SnapshotProp::Directive(directive) => {
            directive.name == "bind"
                && matches!(
                    directive.argument.as_ref(),
                    Some(SnapshotExpression::Simple(argument))
                        if argument
                            .js_raw
                            .as_deref()
                            .unwrap_or(argument.location.source.as_str())
                            .trim()
                            == "is"
                )
        }
    })
}

fn setup_component_binding_key<'a>(bindings: &'a BindingMetadata, name: &str) -> Option<&'a str> {
    let (binding, binding_type) = bindings.bindings.get_key_value(name)?;
    (binding_type.non_inline_template_prefix() == "$setup.").then_some(binding.as_str())
}
