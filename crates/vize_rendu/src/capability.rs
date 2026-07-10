//! Capabilities inferred from a Rendu product and checked by consumers.

use crate::{RenduNode, RenduProperty, RenduRoot};

/// A render feature that a consumer must understand.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum RenduCapability {
    Elements,
    Components,
    Slots,
    Text,
    Expressions,
    Properties,
    Directives,
    Conditionals,
    Iteration,
    Hoists,
    SourceProvenance,
}

impl RenduCapability {
    pub const ALL: [Self; 11] = [
        Self::Elements,
        Self::Components,
        Self::Slots,
        Self::Text,
        Self::Expressions,
        Self::Properties,
        Self::Directives,
        Self::Conditionals,
        Self::Iteration,
        Self::Hoists,
        Self::SourceProvenance,
    ];

    const fn bit(self) -> u16 {
        1 << self as u8
    }
}

/// Compact feature set. The root stores the set inferred from its actual HIR,
/// while a backend can construct its supported set and compare the two.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub struct RenduCapabilities(u16);

impl RenduCapabilities {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn all() -> Self {
        Self((1 << RenduCapability::ALL.len()) - 1)
    }

    pub const fn with(mut self, capability: RenduCapability) -> Self {
        self.0 |= capability.bit();
        self
    }

    pub const fn contains(self, capability: RenduCapability) -> bool {
        self.0 & capability.bit() != 0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Required capabilities not present in `supported`.
    pub const fn unsupported_by(self, supported: Self) -> Self {
        Self(self.0 & !supported.0)
    }

    pub fn iter(self) -> impl Iterator<Item = RenduCapability> {
        RenduCapability::ALL
            .into_iter()
            .filter(move |capability| self.contains(*capability))
    }

    pub(crate) fn infer(root: &RenduRoot) -> Self {
        let mut capabilities = Self::empty();
        let mut visited = vec![false; root.nodes().len()];
        let mut pending = root.entry().to_vec();
        while let Some(id) = pending.pop() {
            if visited[id.index()] {
                continue;
            }
            visited[id.index()] = true;
            let node = root.node(id).expect("validated capability node");
            node.visit_children(|child| pending.push(child));
            if node.provenance().primary.is_some() || !node.provenance().related.is_empty() {
                capabilities = capabilities.with(RenduCapability::SourceProvenance);
            }
            node.visit_nested_provenance(|provenance| {
                if provenance.primary.is_some() || !provenance.related.is_empty() {
                    capabilities = capabilities.with(RenduCapability::SourceProvenance);
                }
            });
            node.visit_expressions(|expression| {
                capabilities = capabilities.with(RenduCapability::Expressions);
                let expression = root
                    .expression(expression)
                    .expect("validated capability expression");
                if expression.provenance.primary.is_some()
                    || !expression.provenance.related.is_empty()
                {
                    capabilities = capabilities.with(RenduCapability::SourceProvenance);
                }
            });
            match node {
                RenduNode::Fragment { .. } | RenduNode::Comment { .. } => {}
                RenduNode::Element { properties, .. } => {
                    capabilities = capabilities.with(RenduCapability::Elements);
                    capabilities.observe_properties(properties);
                }
                RenduNode::Component { properties, .. } => {
                    capabilities = capabilities.with(RenduCapability::Components);
                    capabilities.observe_properties(properties);
                }
                RenduNode::SlotOutlet { properties, .. } => {
                    capabilities = capabilities.with(RenduCapability::Slots);
                    capabilities.observe_properties(properties);
                }
                RenduNode::SlotContent { .. } => {
                    capabilities = capabilities.with(RenduCapability::Slots);
                }
                RenduNode::Text { .. } => {
                    capabilities = capabilities.with(RenduCapability::Text);
                }
                RenduNode::Expression { .. } => {
                    capabilities = capabilities.with(RenduCapability::Expressions);
                }
                RenduNode::If { .. } => {
                    capabilities = capabilities.with(RenduCapability::Conditionals);
                }
                RenduNode::For { .. } => {
                    capabilities = capabilities
                        .with(RenduCapability::Iteration)
                        .with(RenduCapability::Expressions);
                }
                RenduNode::HoistRef { .. } => {
                    capabilities = capabilities.with(RenduCapability::Hoists);
                }
            }
        }
        capabilities
    }

    fn observe_properties(&mut self, properties: &[RenduProperty]) {
        if !properties.is_empty() {
            *self = self.with(RenduCapability::Properties);
        }
        if properties
            .iter()
            .any(|property| matches!(property, RenduProperty::Directive(_)))
        {
            *self = self.with(RenduCapability::Directives);
        }
        if properties.iter().any(property_uses_expression) {
            *self = self.with(RenduCapability::Expressions);
        }
        if properties.iter().any(|property| {
            property.provenance().primary.is_some() || !property.provenance().related.is_empty()
        }) {
            *self = self.with(RenduCapability::SourceProvenance);
        }
    }
}

fn property_uses_expression(property: &RenduProperty) -> bool {
    use crate::RenduName;
    use crate::property::RenduAttributeValue;

    match property {
        RenduProperty::Attribute(attribute) => {
            matches!(attribute.name, RenduName::Dynamic(_))
                || matches!(attribute.value, Some(RenduAttributeValue::Expression(_)))
        }
        RenduProperty::Directive(directive) => {
            directive.expression.is_some()
                || matches!(directive.argument, Some(RenduName::Dynamic(_)))
        }
        RenduProperty::Spread { .. } => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_capabilities_are_a_set_difference() {
        let required = RenduCapabilities::empty()
            .with(RenduCapability::Elements)
            .with(RenduCapability::Directives);
        let supported = RenduCapabilities::empty().with(RenduCapability::Elements);

        let missing = required.unsupported_by(supported);
        assert!(!missing.contains(RenduCapability::Elements));
        assert!(missing.contains(RenduCapability::Directives));
    }

    #[test]
    fn iteration_is_stable_and_excludes_absent_capabilities() {
        let capabilities = RenduCapabilities::empty()
            .with(RenduCapability::Components)
            .with(RenduCapability::Iteration);
        assert_eq!(
            capabilities.iter().collect::<Vec<_>>(),
            [RenduCapability::Components, RenduCapability::Iteration]
        );
    }
}
