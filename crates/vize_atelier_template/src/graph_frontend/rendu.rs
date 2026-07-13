use vize_carton::source_anchor::SourceAnchor;
use vize_relief::{
    ElementType, ReliefSnapshot, ReliefSnapshotNode, ReliefSnapshotNodeId, SnapshotElement,
    SnapshotFor, SnapshotIf, SnapshotIfBranch, SnapshotProp, SnapshotTextCallContent,
};
use vize_rendu::{
    RenduAttribute, RenduAttributeValue, RenduBuilder, RenduDirective, RenduEscapeMode,
    RenduIfBranch, RenduName, RenduNode, RenduNodeId, RenduProperty, RenduRoot, RenduSourceId,
};

use super::{
    TemplateGraphAdapterError,
    expression::{add_rendu_compound, add_rendu_expression, add_rendu_name},
    provenance::{add_rendu_source, rendu_provenance},
    rendu_helpers::{
        binding, is_name_argument, namespace, optional_binding, slot_bindings, slot_directive,
    },
};

/// Lower one cached Relief syntax product directly into independent Rendu HIR.
pub fn lower_relief_snapshot_to_rendu(
    snapshot: &ReliefSnapshot,
) -> Result<RenduRoot, TemplateGraphAdapterError> {
    RenduLowerer::new(snapshot, None).lower()
}

#[doc(hidden)]
pub fn lower_relief_snapshot_to_rendu_with_anchor(
    snapshot: &ReliefSnapshot,
    anchor: SourceAnchor,
) -> Result<RenduRoot, TemplateGraphAdapterError> {
    RenduLowerer::new(snapshot, Some(anchor)).lower()
}

struct RenduLowerer<'a> {
    snapshot: &'a ReliefSnapshot,
    builder: RenduBuilder,
    source: RenduSourceId,
}

impl<'a> RenduLowerer<'a> {
    fn new(snapshot: &'a ReliefSnapshot, anchor: Option<SourceAnchor>) -> Self {
        let mut builder = RenduBuilder::new();
        let source = add_rendu_source(&mut builder, snapshot, anchor);
        Self {
            snapshot,
            builder,
            source,
        }
    }

    fn lower(mut self) -> Result<RenduRoot, TemplateGraphAdapterError> {
        let entry = self.lower_nodes(self.snapshot.children())?;
        self.builder.set_entry(entry);
        Ok(self.builder.finish()?)
    }

    fn lower_nodes(
        &mut self,
        nodes: &[ReliefSnapshotNodeId],
    ) -> Result<Vec<RenduNodeId>, TemplateGraphAdapterError> {
        nodes.iter().map(|id| self.lower_node(*id)).collect()
    }

    fn lower_node(
        &mut self,
        id: ReliefSnapshotNodeId,
    ) -> Result<RenduNodeId, TemplateGraphAdapterError> {
        let snapshot = self.snapshot;
        let node = snapshot
            .node(id)
            .ok_or(TemplateGraphAdapterError::MissingSnapshotNode(id))?;
        match node {
            ReliefSnapshotNode::Element(element) => self.lower_element(element),
            ReliefSnapshotNode::Text(text) => Ok(self.builder.add_node(RenduNode::Text {
                value: text.content.as_str().into(),
                provenance: rendu_provenance(&text.location, self.source),
            })),
            ReliefSnapshotNode::Comment(comment) => Ok(self.builder.add_node(RenduNode::Comment {
                value: comment.content.as_str().into(),
                provenance: rendu_provenance(&comment.location, self.source),
            })),
            ReliefSnapshotNode::Interpolation(interpolation) => {
                let expression =
                    add_rendu_expression(&mut self.builder, &interpolation.content, self.source);
                Ok(self.builder.add_node(RenduNode::Expression {
                    expression,
                    escape: RenduEscapeMode::Escaped,
                    provenance: rendu_provenance(&interpolation.location, self.source),
                }))
            }
            ReliefSnapshotNode::If(if_node) => self.lower_if(if_node),
            ReliefSnapshotNode::IfBranch(branch) => self.lower_standalone_branch(branch),
            ReliefSnapshotNode::For(for_node) => self.lower_for(for_node),
            ReliefSnapshotNode::TextCall(call) => match &call.content {
                SnapshotTextCallContent::Text(text) => Ok(self.builder.add_node(RenduNode::Text {
                    value: text.content.as_str().into(),
                    provenance: rendu_provenance(&call.location, self.source),
                })),
                SnapshotTextCallContent::Interpolation(interpolation) => {
                    let expression = add_rendu_expression(
                        &mut self.builder,
                        &interpolation.content,
                        self.source,
                    );
                    Ok(self.builder.add_node(RenduNode::Expression {
                        expression,
                        escape: RenduEscapeMode::Escaped,
                        provenance: rendu_provenance(&call.location, self.source),
                    }))
                }
                SnapshotTextCallContent::Compound(compound) => {
                    let expression = add_rendu_compound(&mut self.builder, compound, self.source);
                    Ok(self.builder.add_node(RenduNode::Expression {
                        expression,
                        escape: RenduEscapeMode::Escaped,
                        provenance: rendu_provenance(&call.location, self.source),
                    }))
                }
            },
            ReliefSnapshotNode::CompoundExpression(compound) => {
                let expression = add_rendu_compound(&mut self.builder, compound, self.source);
                Ok(self.builder.add_node(RenduNode::Expression {
                    expression,
                    escape: RenduEscapeMode::Escaped,
                    provenance: rendu_provenance(&compound.location, self.source),
                }))
            }
            ReliefSnapshotNode::Hoisted(hoist) => {
                let index = u32::try_from(hoist.index)
                    .map_err(|_| TemplateGraphAdapterError::HoistIndexOverflow(hoist.index))?;
                Ok(self.builder.add_node(RenduNode::HoistRef {
                    index,
                    provenance: rendu_provenance(&hoist.location, self.source),
                }))
            }
        }
    }

    fn lower_element(
        &mut self,
        element: &SnapshotElement,
    ) -> Result<RenduNodeId, TemplateGraphAdapterError> {
        let children = self.lower_nodes(element.children())?;
        let provenance = rendu_provenance(&element.location, self.source);
        let node = match element.tag_type {
            ElementType::Element => RenduNode::Element {
                tag: element.tag.as_str().into(),
                namespace: namespace(element.namespace),
                properties: self.lower_properties(&element.props, None),
                children,
                provenance,
            },
            ElementType::Component => RenduNode::Component {
                name: RenduName::static_name(element.tag.as_str()),
                properties: self.lower_properties(&element.props, None),
                children,
                provenance,
            },
            ElementType::Slot => {
                let (name, consumed) = self.slot_outlet_name(&element.props);
                RenduNode::SlotOutlet {
                    name,
                    properties: self.lower_properties(&element.props, consumed),
                    fallback: children,
                    provenance,
                }
            }
            ElementType::Template => {
                if let Some((_index, directive)) = slot_directive(&element.props) {
                    RenduNode::SlotContent {
                        name: directive
                            .argument
                            .as_ref()
                            .map(|argument| {
                                add_rendu_name(&mut self.builder, argument, self.source)
                            })
                            .unwrap_or_else(|| RenduName::static_name("default")),
                        bindings: slot_bindings(directive, self.source),
                        children,
                        provenance,
                    }
                } else {
                    RenduNode::Fragment {
                        children,
                        provenance,
                    }
                }
            }
        };
        Ok(self.builder.add_node(node))
    }

    fn lower_if(&mut self, node: &SnapshotIf) -> Result<RenduNodeId, TemplateGraphAdapterError> {
        let mut branches = Vec::with_capacity(node.branches().len());
        for id in node.branches() {
            let Some(ReliefSnapshotNode::IfBranch(branch)) = self.snapshot.node(*id) else {
                return Err(TemplateGraphAdapterError::ExpectedIfBranch(*id));
            };
            branches.push(self.lower_branch(branch)?);
        }
        Ok(self.builder.add_node(RenduNode::If {
            branches,
            provenance: rendu_provenance(&node.location, self.source),
        }))
    }

    fn lower_standalone_branch(
        &mut self,
        branch: &SnapshotIfBranch,
    ) -> Result<RenduNodeId, TemplateGraphAdapterError> {
        let provenance = rendu_provenance(&branch.location, self.source);
        let branch = self.lower_branch(branch)?;
        Ok(self.builder.add_node(RenduNode::If {
            branches: vec![branch],
            provenance,
        }))
    }

    fn lower_branch(
        &mut self,
        branch: &SnapshotIfBranch,
    ) -> Result<RenduIfBranch, TemplateGraphAdapterError> {
        let condition = branch
            .condition
            .as_ref()
            .map(|condition| add_rendu_expression(&mut self.builder, condition, self.source));
        Ok(
            RenduIfBranch::new(condition, self.lower_nodes(branch.children())?)
                .with_provenance(rendu_provenance(&branch.location, self.source)),
        )
    }

    fn lower_for(&mut self, node: &SnapshotFor) -> Result<RenduNodeId, TemplateGraphAdapterError> {
        let source = add_rendu_expression(&mut self.builder, &node.source, self.source);
        let provenance = rendu_provenance(&node.location, self.source);
        let value = binding(
            node.value_alias
                .as_ref()
                .or(node.parse_result.value.as_ref()),
            "_value",
            &provenance,
        );
        let key = optional_binding(
            node.key_alias.as_ref().or(node.parse_result.key.as_ref()),
            &provenance,
        );
        let index = optional_binding(
            node.object_index_alias
                .as_ref()
                .or(node.parse_result.index.as_ref()),
            &provenance,
        );
        let body = self.lower_nodes(node.children())?;
        Ok(self.builder.add_node(RenduNode::For {
            source,
            value,
            key,
            index,
            key_expression: None,
            body,
            provenance,
        }))
    }

    fn lower_properties(
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
                    lowered = lowered.with_argument(add_rendu_name(
                        &mut self.builder,
                        argument,
                        self.source,
                    ));
                }
                if let Some(expression) = &directive.expression {
                    lowered = lowered.with_expression(add_rendu_expression(
                        &mut self.builder,
                        expression,
                        self.source,
                    ));
                }
                for modifier in &directive.modifiers {
                    lowered = lowered.with_modifier(modifier.content.as_str());
                }
                RenduProperty::Directive(lowered)
            }
        }
    }

    fn slot_outlet_name(&mut self, properties: &[SnapshotProp]) -> (RenduName, Option<usize>) {
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
                            RenduName::Dynamic(add_rendu_expression(
                                &mut self.builder,
                                expression,
                                self.source,
                            )),
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
