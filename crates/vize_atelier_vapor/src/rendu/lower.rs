use vize_carton::{String, cstr};
use vize_rendu::{
    RenduAttributeValue, RenduName, RenduNode, RenduNodeId, RenduProperty, RenduProvenance,
    RenduRoot,
};

#[path = "lower/slot.rs"]
mod slot;

use super::model::{
    VaporAttributeValue, VaporBinding, VaporBlock, VaporBlockId, VaporBranch, VaporDirective,
    VaporExpression, VaporExpressionId, VaporName, VaporOperation, VaporPlan, VaporProperty,
};
use super::syntax::{escape_html_attribute, escape_html_comment, escape_html_text};

/// Lower a Rendu root into an owned Vapor plan.
///
/// The result copies sources and opaque expressions and therefore has no
/// lifetime relationship with the frontend artifact.
pub fn plan_rendu(root: &RenduRoot) -> VaporPlan {
    Planner::new(root).plan()
}

struct Planner<'a> {
    root: &'a RenduRoot,
    blocks: Vec<VaporBlock>,
}

impl<'a> Planner<'a> {
    fn new(root: &'a RenduRoot) -> Self {
        Self {
            root,
            blocks: Vec::new(),
        }
    }

    fn plan(mut self) -> VaporPlan {
        let entry = self.lower_block(self.root.entry(), RenduProvenance::generated());
        let expressions = self
            .root
            .expressions()
            .iter()
            .map(|expression| VaporExpression {
                code: expression.code.clone(),
                kind: expression.kind,
                provenance: expression.provenance.clone(),
            })
            .collect();
        VaporPlan {
            sources: self.root.sources().to_vec(),
            expressions,
            blocks: self.blocks,
            entry,
        }
    }

    fn lower_block(&mut self, nodes: &[RenduNodeId], provenance: RenduProvenance) -> VaporBlockId {
        let id = VaporBlockId::from_index(self.blocks.len());
        self.blocks.push(VaporBlock {
            operations: Vec::new(),
            provenance,
        });
        let operations = nodes
            .iter()
            .copied()
            .map(|node| self.lower_node(node))
            .collect();
        self.blocks[id.index()].operations = operations;
        id
    }

    fn lower_node(&mut self, id: RenduNodeId) -> VaporOperation {
        let node = self.root.node(id).expect("validated Rendu node");
        if let Some(html) = self.static_html(id) {
            return VaporOperation::StaticHtml {
                html: html.as_str().into(),
                provenance: self.static_provenance(id),
            };
        }
        match node {
            RenduNode::Fragment {
                children,
                provenance,
            } => VaporOperation::Fragment {
                body: self.lower_block(children, provenance.clone()),
                provenance: provenance.clone(),
            },
            RenduNode::Element {
                tag,
                namespace,
                properties,
                children,
                provenance,
            } => VaporOperation::Element {
                tag: tag.clone(),
                namespace: namespace.clone(),
                properties: properties.iter().map(lower_property).collect(),
                body: self.lower_block(children, provenance.clone()),
                provenance: provenance.clone(),
            },
            RenduNode::Component {
                kind,
                name,
                properties,
                children,
                provenance,
            } => VaporOperation::Component {
                kind: *kind,
                name: lower_name(name),
                properties: properties.iter().map(lower_property).collect(),
                slots: self.lower_component_slots(children, provenance.clone()),
                provenance: provenance.clone(),
            },
            RenduNode::SlotOutlet {
                name,
                properties,
                fallback,
                provenance,
            } => VaporOperation::SlotOutlet {
                name: lower_name(name),
                properties: properties.iter().map(lower_property).collect(),
                fallback: self.lower_block(fallback, provenance.clone()),
                provenance: provenance.clone(),
            },
            RenduNode::SlotContent {
                name,
                bindings,
                children,
                provenance,
            } => VaporOperation::SlotContent {
                name: lower_name(name),
                bindings: bindings.iter().map(lower_binding).collect(),
                body: self.lower_block(children, provenance.clone()),
                provenance: provenance.clone(),
            },
            RenduNode::Expression {
                expression,
                escape,
                provenance,
            } => VaporOperation::DynamicText {
                expression: expression_id(*expression),
                escape: *escape,
                provenance: provenance.clone(),
            },
            RenduNode::If {
                branches,
                provenance,
            } => VaporOperation::Conditional {
                branches: branches
                    .iter()
                    .map(|branch| VaporBranch {
                        condition: branch.condition.map(expression_id),
                        body: self.lower_block(&branch.body, branch.provenance.clone()),
                        provenance: branch.provenance.clone(),
                    })
                    .collect(),
                provenance: provenance.clone(),
            },
            RenduNode::For {
                source,
                value,
                key,
                index,
                key_expression,
                body,
                provenance,
            } => VaporOperation::Iterate {
                source: expression_id(*source),
                value: lower_binding(value),
                key: key.as_ref().map(lower_binding),
                index: index.as_ref().map(lower_binding),
                key_expression: key_expression.map(expression_id),
                body: self.lower_block(body, provenance.clone()),
                provenance: provenance.clone(),
            },
            RenduNode::HoistRef { index, provenance } => VaporOperation::HoistRef {
                index: *index,
                provenance: provenance.clone(),
            },
            RenduNode::Text { .. } | RenduNode::Comment { .. } => {
                unreachable!("text and comments always form static HTML")
            }
            _ => {
                let description = cstr!("{node:?}");
                VaporOperation::Unsupported {
                    description: description.as_str().into(),
                    provenance: node.provenance().clone(),
                }
            }
        }
    }

    fn static_html(&self, id: RenduNodeId) -> Option<String> {
        let node = self.root.node(id).expect("validated Rendu node");
        match node {
            RenduNode::Fragment { children, .. } => self.static_children(children),
            RenduNode::Element {
                tag,
                properties,
                children,
                ..
            } => {
                let mut html = String::default();
                html.push('<');
                html.push_str(tag);
                for property in properties {
                    let RenduProperty::Attribute(attribute) = property else {
                        return None;
                    };
                    let RenduName::Static(name) = &attribute.name else {
                        return None;
                    };
                    html.push(' ');
                    html.push_str(name);
                    match &attribute.value {
                        None => {}
                        Some(RenduAttributeValue::Static(value)) => {
                            html.push_str("=\"");
                            html.push_str(&escape_html_attribute(value));
                            html.push('"');
                        }
                        Some(RenduAttributeValue::Expression(_)) => return None,
                    }
                }
                html.push('>');
                if !vize_carton::is_void_tag(tag) {
                    html.push_str(&self.static_children(children)?);
                    html.push_str("</");
                    html.push_str(tag);
                    html.push('>');
                }
                Some(html)
            }
            RenduNode::Text { value, .. } => Some(escape_html_text(value)),
            RenduNode::Comment { value, .. } => {
                Some(cstr!("<!--{}-->", escape_html_comment(value)))
            }
            _ => None,
        }
    }

    fn static_children(&self, children: &[RenduNodeId]) -> Option<String> {
        let mut html = String::default();
        for &child in children {
            html.push_str(&self.static_html(child)?);
        }
        Some(html)
    }

    fn static_provenance(&self, id: RenduNodeId) -> RenduProvenance {
        let mut spans = Vec::new();
        self.collect_static_spans(id, &mut spans);
        let primary = spans.first().copied();
        let related = spans.into_iter().skip(1).collect();
        RenduProvenance { primary, related }
    }

    fn collect_static_spans(&self, id: RenduNodeId, spans: &mut Vec<vize_rendu::RenduSpan>) {
        let node = self.root.node(id).expect("validated Rendu node");
        push_unique_spans(spans, node.provenance());
        match node {
            RenduNode::Fragment { children, .. } => {
                for &child in children {
                    self.collect_static_spans(child, spans);
                }
            }
            RenduNode::Element {
                properties,
                children,
                ..
            } => {
                for property in properties {
                    push_unique_spans(spans, property.provenance());
                }
                for &child in children {
                    self.collect_static_spans(child, spans);
                }
            }
            RenduNode::Text { .. } | RenduNode::Comment { .. } => {}
            _ => unreachable!("only static nodes are collected"),
        }
    }
}

fn lower_name(name: &RenduName) -> VaporName {
    match name {
        RenduName::Static(name) => VaporName::Static(name.clone()),
        RenduName::Dynamic(expression) => VaporName::Dynamic(expression_id(*expression)),
    }
}

fn lower_binding(binding: &vize_rendu::RenduBinding) -> VaporBinding {
    VaporBinding {
        pattern: binding.pattern.clone(),
        provenance: binding.provenance.clone(),
    }
}

fn lower_property(property: &RenduProperty) -> VaporProperty {
    match property {
        RenduProperty::Attribute(attribute) => VaporProperty::Attribute {
            name: lower_name(&attribute.name),
            value: attribute.value.as_ref().map(|value| match value {
                RenduAttributeValue::Static(value) => VaporAttributeValue::Static(value.clone()),
                RenduAttributeValue::Expression(expression) => {
                    VaporAttributeValue::Expression(expression_id(*expression))
                }
            }),
            provenance: attribute.provenance.clone(),
        },
        RenduProperty::Directive(directive) => VaporProperty::Directive(VaporDirective {
            name: directive.name.clone(),
            argument: directive.argument.as_ref().map(lower_name),
            expression: directive.expression.map(expression_id),
            modifiers: directive.modifiers.clone(),
            provenance: directive.provenance.clone(),
        }),
        RenduProperty::Spread {
            expression,
            provenance,
        } => VaporProperty::Spread {
            expression: expression_id(*expression),
            provenance: provenance.clone(),
        },
    }
}

fn expression_id(id: vize_rendu::RenduExpressionId) -> VaporExpressionId {
    VaporExpressionId::from_index(id.index())
}

fn push_unique_spans(spans: &mut Vec<vize_rendu::RenduSpan>, provenance: &RenduProvenance) {
    for span in provenance.spans() {
        if !spans.contains(&span) {
            spans.push(span);
        }
    }
}
