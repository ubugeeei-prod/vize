use vize_carton::{BindingMetadata, FxHashSet, String, source_anchor::SourceAnchor};
use vize_relief::{
    ElementType, ReliefSnapshot, ReliefSnapshotNode, ReliefSnapshotNodeId, SnapshotElement,
    SnapshotExpression, SnapshotTextCallContent,
};
use vize_rendu::{
    RenduBuilder, RenduEscapeMode, RenduName, RenduNode, RenduNodeId, RenduRoot, RenduSourceId,
};

use super::{
    TemplateGraphAdapterError,
    expression::{
        add_rendu_compound_with_code, add_rendu_expression_with_code, compound_code,
        expression_code,
    },
    provenance::{add_rendu_source, rendu_provenance},
    rendu_helpers::{namespace, slot_bindings, slot_directive},
    scope::{pattern_bindings, strip_local_scope_prefixes},
};

mod component;
mod control;
mod property;

use component::component_kind;

/// Lower one cached Relief syntax product directly into independent Rendu HIR.
pub fn lower_relief_snapshot_to_rendu(
    snapshot: &ReliefSnapshot,
) -> Result<RenduRoot, TemplateGraphAdapterError> {
    RenduLowerer::new(snapshot, None, None).lower()
}

#[doc(hidden)]
pub fn lower_relief_snapshot_to_rendu_with_anchor(
    snapshot: &ReliefSnapshot,
    anchor: SourceAnchor,
) -> Result<RenduRoot, TemplateGraphAdapterError> {
    RenduLowerer::new(snapshot, Some(anchor), None).lower()
}

/// Lower one cached Relief snapshot while preserving component bindings
/// resolved by a peer semantic product.
///
/// Raw-template frontends intentionally call the binding-free entry point.
/// SFC hosts pass their Croquis projection through the neutral Carton binding
/// contract so Rendu remains independent of both frontend and semantic crates.
#[doc(hidden)]
pub fn lower_relief_snapshot_to_rendu_with_anchor_and_bindings(
    snapshot: &ReliefSnapshot,
    anchor: SourceAnchor,
    bindings: &BindingMetadata,
) -> Result<RenduRoot, TemplateGraphAdapterError> {
    RenduLowerer::new(snapshot, Some(anchor), Some(bindings)).lower()
}

struct RenduLowerer<'a> {
    snapshot: &'a ReliefSnapshot,
    builder: RenduBuilder,
    source: RenduSourceId,
    bindings: Option<&'a BindingMetadata>,
    scopes: Vec<FxHashSet<String>>,
}

impl<'a> RenduLowerer<'a> {
    fn new(
        snapshot: &'a ReliefSnapshot,
        anchor: Option<SourceAnchor>,
        bindings: Option<&'a BindingMetadata>,
    ) -> Self {
        let mut builder = RenduBuilder::new();
        let source = add_rendu_source(&mut builder, snapshot, anchor);
        Self {
            snapshot,
            builder,
            source,
            bindings,
            scopes: Vec::new(),
        }
    }

    fn lower(mut self) -> Result<RenduRoot, TemplateGraphAdapterError> {
        let entry = self.lower_nodes(self.snapshot.children())?;
        self.builder.set_entry(entry);
        Ok(self.builder.finish()?)
    }

    fn add_expression(&mut self, expression: &SnapshotExpression) -> vize_rendu::RenduExpressionId {
        let code = strip_local_scope_prefixes(&self.scopes, &expression_code(expression));
        add_rendu_expression_with_code(&mut self.builder, expression, code.as_str(), self.source)
    }

    fn add_compound(
        &mut self,
        expression: &vize_relief::SnapshotCompoundExpression,
    ) -> vize_rendu::RenduExpressionId {
        let code = strip_local_scope_prefixes(&self.scopes, &compound_code(expression));
        add_rendu_compound_with_code(&mut self.builder, expression, code.as_str(), self.source)
    }

    fn add_name(&mut self, expression: &SnapshotExpression) -> RenduName {
        match expression {
            SnapshotExpression::Simple(simple) if simple.is_static => {
                RenduName::static_name(simple.content.as_str())
            }
            _ => RenduName::Dynamic(self.add_expression(expression)),
        }
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
                let expression = self.add_expression(&interpolation.content);
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
                    let expression = self.add_expression(&interpolation.content);
                    Ok(self.builder.add_node(RenduNode::Expression {
                        expression,
                        escape: RenduEscapeMode::Escaped,
                        provenance: rendu_provenance(&call.location, self.source),
                    }))
                }
                SnapshotTextCallContent::Compound(compound) => {
                    let expression = self.add_compound(compound);
                    Ok(self.builder.add_node(RenduNode::Expression {
                        expression,
                        escape: RenduEscapeMode::Escaped,
                        provenance: rendu_provenance(&call.location, self.source),
                    }))
                }
            },
            ReliefSnapshotNode::CompoundExpression(compound) => {
                let expression = self.add_compound(compound);
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
        let provenance = rendu_provenance(&element.location, self.source);
        let node = match element.tag_type {
            ElementType::Element => RenduNode::Element {
                tag: element.tag.as_str().into(),
                namespace: namespace(element.namespace),
                properties: self.lower_properties(&element.props, None),
                children: self.lower_nodes(element.children())?,
                provenance,
            },
            ElementType::Component => {
                // Capture Vue's source-level component identity before
                // script-setup binding resolution can turn its name into an
                // opaque runtime expression such as `$setup.Suspense`.
                let kind = component_kind(element);
                let name = if kind.is_builtin() {
                    RenduName::static_name(element.tag.as_str())
                } else {
                    self.component_name(element)
                };
                if let Some((index, directive)) = slot_directive(&element.props) {
                    // `v-slot` on a component is shorthand for a default slot on
                    // the component itself. Lower it into the same first-class
                    // Rendu node used by `<template #default>` instead of letting
                    // a backend mistake it for a runtime directive.
                    let slot_name = directive
                        .argument
                        .as_ref()
                        .map(|argument| self.add_name(argument))
                        .unwrap_or_else(|| RenduName::static_name("default"));
                    let bindings = slot_bindings(directive, self.source);
                    let mut scope = FxHashSet::default();
                    for binding in &bindings {
                        scope.extend(pattern_bindings(&binding.pattern));
                    }
                    self.scopes.push(scope);
                    let children = self.lower_nodes(element.children())?;
                    self.scopes.pop();
                    let slot = self.builder.add_node(RenduNode::SlotContent {
                        name: slot_name,
                        bindings,
                        children,
                        provenance: provenance.clone(),
                    });
                    RenduNode::Component {
                        kind,
                        name,
                        properties: self.lower_properties(&element.props, Some(index)),
                        children: vec![slot],
                        provenance,
                    }
                } else {
                    RenduNode::Component {
                        kind,
                        name,
                        properties: self.lower_properties(&element.props, None),
                        children: self.lower_nodes(element.children())?,
                        provenance,
                    }
                }
            }
            ElementType::Slot => {
                let (name, consumed) = self.slot_outlet_name(&element.props);
                RenduNode::SlotOutlet {
                    name,
                    properties: self.lower_properties(&element.props, consumed),
                    fallback: self.lower_nodes(element.children())?,
                    provenance,
                }
            }
            ElementType::Template => {
                if let Some((_index, directive)) = slot_directive(&element.props) {
                    let name = directive
                        .argument
                        .as_ref()
                        .map(|argument| self.add_name(argument))
                        .unwrap_or_else(|| RenduName::static_name("default"));
                    let bindings = slot_bindings(directive, self.source);
                    let mut scope = FxHashSet::default();
                    for binding in &bindings {
                        scope.extend(pattern_bindings(&binding.pattern));
                    }
                    self.scopes.push(scope);
                    let children = self.lower_nodes(element.children())?;
                    self.scopes.pop();
                    RenduNode::SlotContent {
                        name,
                        bindings,
                        children,
                        provenance,
                    }
                } else {
                    RenduNode::Fragment {
                        children: self.lower_nodes(element.children())?,
                        provenance,
                    }
                }
            }
        };
        Ok(self.builder.add_node(node))
    }
}
