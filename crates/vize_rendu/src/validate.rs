//! Structural validation for arena references and provenance.

use std::fmt;

use crate::{
    RenduExpressionId, RenduName, RenduNode, RenduNodeId, RenduProperty, RenduRoot, RenduSourceId,
    RenduSpan,
};

#[derive(Debug, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum RenduValidationError {
    MissingEntryNode(RenduNodeId),
    MissingChildNode {
        parent: RenduNodeId,
        child: RenduNodeId,
    },
    MissingExpression {
        owner: Option<RenduNodeId>,
        expression: RenduExpressionId,
    },
    MissingSource(RenduSourceId),
    InvalidSpan(RenduSpan),
    CyclicNode(RenduNodeId),
    EmptyElementTag(RenduNodeId),
    EmptyDirectiveName(RenduNodeId),
    EmptyBinding(RenduNodeId),
    ElseBranchNotLast(RenduNodeId),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RenduValidationErrors(pub Vec<RenduValidationError>);

impl RenduValidationErrors {
    pub fn iter(&self) -> impl Iterator<Item = &RenduValidationError> {
        self.0.iter()
    }
}

impl fmt::Display for RenduValidationErrors {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid Rendu HIR ({} error(s))", self.0.len())
    }
}

impl std::error::Error for RenduValidationErrors {}

impl RenduRoot {
    pub fn validate(&self) -> Result<(), RenduValidationErrors> {
        let mut errors = Vec::new();
        self.validate_sources(&mut errors);
        self.validate_expressions(&mut errors);
        self.validate_nodes(&mut errors);
        self.validate_graph(&mut errors);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(RenduValidationErrors(errors))
        }
    }

    fn validate_sources(&self, errors: &mut Vec<RenduValidationError>) {
        for expression in &self.expressions {
            validate_provenance(self, &expression.provenance, errors);
        }
    }

    fn validate_expressions(&self, errors: &mut Vec<RenduValidationError>) {
        for (index, node) in self.nodes.iter().enumerate() {
            let id = RenduNodeId::from_index(index);
            node.visit_expressions(|expression| {
                if self.expression(expression).is_none() {
                    errors.push(RenduValidationError::MissingExpression {
                        owner: Some(id),
                        expression,
                    });
                }
            });
        }
    }

    fn validate_nodes(&self, errors: &mut Vec<RenduValidationError>) {
        for (index, node) in self.nodes.iter().enumerate() {
            let id = RenduNodeId::from_index(index);
            validate_provenance(self, node.provenance(), errors);
            node.visit_nested_provenance(|provenance| {
                validate_provenance(self, provenance, errors)
            });
            node.visit_children(|child| {
                if self.node(child).is_none() {
                    errors.push(RenduValidationError::MissingChildNode { parent: id, child });
                }
            });
            validate_node_shape(id, node, errors);
        }
        for &entry in &self.entry {
            if self.node(entry).is_none() {
                errors.push(RenduValidationError::MissingEntryNode(entry));
            }
        }
    }

    fn validate_graph(&self, errors: &mut Vec<RenduValidationError>) {
        let mut state = vec![VisitState::New; self.nodes.len()];
        for index in 0..self.nodes.len() {
            self.visit_graph(RenduNodeId::from_index(index), &mut state, errors);
        }
    }

    fn visit_graph(
        &self,
        id: RenduNodeId,
        state: &mut [VisitState],
        errors: &mut Vec<RenduValidationError>,
    ) {
        match state[id.index()] {
            VisitState::Active => {
                errors.push(RenduValidationError::CyclicNode(id));
                return;
            }
            VisitState::Done => return,
            VisitState::New => state[id.index()] = VisitState::Active,
        }
        self.node(id)
            .expect("validated node id")
            .visit_children(|child| {
                if self.node(child).is_some() {
                    self.visit_graph(child, state, errors);
                }
            });
        state[id.index()] = VisitState::Done;
    }
}

#[derive(Clone, Copy)]
enum VisitState {
    New,
    Active,
    Done,
}

fn validate_provenance(
    root: &RenduRoot,
    provenance: &crate::RenduProvenance,
    errors: &mut Vec<RenduValidationError>,
) {
    for span in provenance.spans() {
        let Some(source) = root.source(span.source) else {
            errors.push(RenduValidationError::MissingSource(span.source));
            continue;
        };
        if span.start.offset > span.end.offset || span.end.offset as usize > source.contents.len() {
            errors.push(RenduValidationError::InvalidSpan(span));
        }
    }
}

fn validate_node_shape(id: RenduNodeId, node: &RenduNode, errors: &mut Vec<RenduValidationError>) {
    match node {
        RenduNode::Element {
            tag, properties, ..
        } => {
            if tag.is_empty() {
                errors.push(RenduValidationError::EmptyElementTag(id));
            }
            validate_properties(id, properties, errors);
        }
        RenduNode::Component { properties, .. } | RenduNode::SlotOutlet { properties, .. } => {
            validate_properties(id, properties, errors);
        }
        RenduNode::SlotContent { bindings, .. } => {
            if bindings.iter().any(|binding| binding.pattern.is_empty()) {
                errors.push(RenduValidationError::EmptyBinding(id));
            }
        }
        RenduNode::If { branches, .. } => {
            if branches
                .iter()
                .enumerate()
                .any(|(index, branch)| branch.condition.is_none() && index + 1 != branches.len())
            {
                errors.push(RenduValidationError::ElseBranchNotLast(id));
            }
        }
        RenduNode::For {
            value, key, index, ..
        } => {
            if value.pattern.is_empty()
                || key
                    .as_ref()
                    .is_some_and(|binding| binding.pattern.is_empty())
                || index
                    .as_ref()
                    .is_some_and(|binding| binding.pattern.is_empty())
            {
                errors.push(RenduValidationError::EmptyBinding(id));
            }
        }
        RenduNode::Fragment { .. }
        | RenduNode::Text { .. }
        | RenduNode::Expression { .. }
        | RenduNode::Comment { .. }
        | RenduNode::HoistRef { .. } => {}
    }
}

fn validate_properties(
    owner: RenduNodeId,
    properties: &[RenduProperty],
    errors: &mut Vec<RenduValidationError>,
) {
    if properties.iter().any(|property| {
        matches!(property, RenduProperty::Directive(directive) if directive.name.is_empty())
    }) {
        errors.push(RenduValidationError::EmptyDirectiveName(owner));
    }
}

impl RenduNode {
    pub(crate) fn visit_expressions(&self, mut visit: impl FnMut(RenduExpressionId)) {
        match self {
            Self::Element { properties, .. }
            | Self::Component { properties, .. }
            | Self::SlotOutlet { properties, .. } => {
                visit_properties(properties, &mut visit);
            }
            Self::SlotContent { name, .. } => visit_name(name, &mut visit),
            Self::Expression { expression, .. } => visit(*expression),
            Self::If { branches, .. } => branches
                .iter()
                .filter_map(|branch| branch.condition)
                .for_each(&mut visit),
            Self::For {
                source,
                key_expression,
                ..
            } => {
                visit(*source);
                if let Some(key) = key_expression {
                    visit(*key);
                }
            }
            Self::Fragment { .. }
            | Self::Text { .. }
            | Self::Comment { .. }
            | Self::HoistRef { .. } => {}
        }
        if let Self::Component { name, .. } | Self::SlotOutlet { name, .. } = self {
            visit_name(name, visit);
        }
    }

    pub(crate) fn visit_nested_provenance(&self, mut visit: impl FnMut(&crate::RenduProvenance)) {
        match self {
            Self::Element { properties, .. }
            | Self::Component { properties, .. }
            | Self::SlotOutlet { properties, .. } => {
                properties
                    .iter()
                    .for_each(|property| visit(property.provenance()));
            }
            Self::SlotContent { bindings, .. } => {
                bindings
                    .iter()
                    .for_each(|binding| visit(&binding.provenance));
            }
            Self::If { branches, .. } => {
                branches.iter().for_each(|branch| visit(&branch.provenance));
            }
            Self::For {
                value, key, index, ..
            } => {
                visit(&value.provenance);
                key.iter().for_each(|binding| visit(&binding.provenance));
                index.iter().for_each(|binding| visit(&binding.provenance));
            }
            Self::Fragment { .. }
            | Self::Text { .. }
            | Self::Expression { .. }
            | Self::Comment { .. }
            | Self::HoistRef { .. } => {}
        }
    }
}

fn visit_properties(properties: &[RenduProperty], visit: &mut impl FnMut(RenduExpressionId)) {
    use crate::property::RenduAttributeValue;
    for property in properties {
        match property {
            RenduProperty::Attribute(attribute) => {
                visit_name(&attribute.name, &mut *visit);
                if let Some(RenduAttributeValue::Expression(expression)) = attribute.value {
                    visit(expression);
                }
            }
            RenduProperty::Directive(directive) => {
                if let Some(argument) = &directive.argument {
                    visit_name(argument, &mut *visit);
                }
                if let Some(expression) = directive.expression {
                    visit(expression);
                }
            }
            RenduProperty::Spread { expression, .. } => visit(*expression),
        }
    }
}

fn visit_name(name: &RenduName, mut visit: impl FnMut(RenduExpressionId)) {
    if let RenduName::Dynamic(expression) = name {
        visit(*expression);
    }
}
