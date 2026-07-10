//! Frontend-neutral attributes, directives, and spread properties.

use crate::{RenduExpressionId, RenduName, RenduProvenance};

/// A static or dynamic attribute on an element, component, or slot outlet.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RenduAttribute {
    pub name: RenduName,
    /// `None` represents a presence/boolean attribute.
    pub value: Option<RenduAttributeValue>,
    pub provenance: RenduProvenance,
}

impl RenduAttribute {
    pub fn static_value(name: impl Into<Box<str>>, value: impl Into<Box<str>>) -> Self {
        Self {
            name: RenduName::Static(name.into()),
            value: Some(RenduAttributeValue::Static(value.into())),
            provenance: RenduProvenance::generated(),
        }
    }

    pub fn expression(name: impl Into<Box<str>>, value: RenduExpressionId) -> Self {
        Self {
            name: RenduName::Static(name.into()),
            value: Some(RenduAttributeValue::Expression(value)),
            provenance: RenduProvenance::generated(),
        }
    }

    pub fn presence(name: impl Into<Box<str>>) -> Self {
        Self {
            name: RenduName::Static(name.into()),
            value: None,
            provenance: RenduProvenance::generated(),
        }
    }

    pub fn with_provenance(mut self, provenance: RenduProvenance) -> Self {
        self.provenance = provenance;
        self
    }
}

/// Attribute material after frontend lowering.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RenduAttributeValue {
    Static(Box<str>),
    Expression(RenduExpressionId),
}

/// A directive preserved for a target or plugin to lower.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RenduDirective {
    pub name: Box<str>,
    pub argument: Option<RenduName>,
    pub expression: Option<RenduExpressionId>,
    pub modifiers: Vec<Box<str>>,
    pub provenance: RenduProvenance,
}

impl RenduDirective {
    pub fn new(name: impl Into<Box<str>>) -> Self {
        Self {
            name: name.into(),
            argument: None,
            expression: None,
            modifiers: Vec::new(),
            provenance: RenduProvenance::generated(),
        }
    }

    pub fn with_argument(mut self, argument: RenduName) -> Self {
        self.argument = Some(argument);
        self
    }

    pub const fn with_expression(mut self, expression: RenduExpressionId) -> Self {
        self.expression = Some(expression);
        self
    }

    pub fn with_modifier(mut self, modifier: impl Into<Box<str>>) -> Self {
        self.modifiers.push(modifier.into());
        self
    }

    pub fn with_provenance(mut self, provenance: RenduProvenance) -> Self {
        self.provenance = provenance;
        self
    }
}

/// Property vocabulary shared by all frontends.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RenduProperty {
    Attribute(RenduAttribute),
    Directive(RenduDirective),
    Spread {
        expression: RenduExpressionId,
        provenance: RenduProvenance,
    },
}

impl RenduProperty {
    pub const fn spread(expression: RenduExpressionId) -> Self {
        Self::Spread {
            expression,
            provenance: RenduProvenance::generated(),
        }
    }

    pub fn provenance(&self) -> &RenduProvenance {
        match self {
            Self::Attribute(attribute) => &attribute.provenance,
            Self::Directive(directive) => &directive.provenance,
            Self::Spread { provenance, .. } => provenance,
        }
    }
}
