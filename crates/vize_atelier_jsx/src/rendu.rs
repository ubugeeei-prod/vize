//! Direct owned JSX syntax -> Rendu lowering. No Relief value participates.

#[path = "rendu/property.rs"]
mod property;

use vize_carton::line_index::LineIndex;
use vize_rendu::{
    RenduBinding, RenduBuilder, RenduEscapeMode, RenduExpression, RenduExpressionId,
    RenduExpressionKind, RenduIfBranch, RenduName, RenduNamespace, RenduNode, RenduNodeId,
    RenduPosition, RenduProvenance, RenduRoot, RenduSource, RenduSourceId, RenduSpan,
    RenduValidationErrors,
};

use crate::syntax::{
    JsxSyntaxBinding, JsxSyntaxExpression, JsxSyntaxNode, JsxSyntaxSnapshot, JsxSyntaxSpan,
};
use crate::{JsxLang, snapshot_jsx};

/// Owned result of the direct JSX graph path.
#[derive(Debug, Clone)]
pub struct JsxRenduOutput {
    pub snapshot: JsxSyntaxSnapshot,
    pub root: RenduRoot,
}

/// Parse JSX/TSX to an owned snapshot and lower it directly into Rendu.
pub fn lower_source_to_rendu(
    source: &str,
    lang: JsxLang,
) -> Result<JsxRenduOutput, RenduValidationErrors> {
    let snapshot = snapshot_jsx(source, lang);
    let root = snapshot.lower_rendu()?;
    Ok(JsxRenduOutput { snapshot, root })
}

impl JsxSyntaxSnapshot {
    /// Lower this parser-independent snapshot directly into an owned Rendu HIR.
    pub fn lower_rendu(&self) -> Result<RenduRoot, RenduValidationErrors> {
        RenduLowerer::new(self).lower()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Namespace {
    Html,
    Svg,
    MathMl,
}

struct RenduLowerer<'s> {
    snapshot: &'s JsxSyntaxSnapshot,
    line_index: LineIndex<'s>,
    builder: RenduBuilder,
    source: RenduSourceId,
}

impl<'s> RenduLowerer<'s> {
    fn new(snapshot: &'s JsxSyntaxSnapshot) -> Self {
        let mut builder = RenduBuilder::new();
        let mut source = match &snapshot.filename {
            Some(filename) => RenduSource::named(filename.clone(), snapshot.source.clone()),
            None => RenduSource::anonymous(snapshot.source.clone()),
        };
        source = source.with_language(if snapshot.lang.is_typescript() {
            "tsx"
        } else {
            "jsx"
        });
        if let Some(anchor) = snapshot.source_anchor {
            source = source.with_anchor(anchor);
        }
        let source = builder.add_source(source);
        Self {
            snapshot,
            line_index: LineIndex::new(&snapshot.source),
            builder,
            source,
        }
    }

    fn lower(mut self) -> Result<RenduRoot, RenduValidationErrors> {
        let snapshot = self.snapshot;
        let entries = snapshot
            .roots
            .iter()
            .map(|node| self.node(node, Namespace::Html))
            .collect::<Vec<_>>();
        self.builder.set_entry(entries);
        self.builder.finish()
    }

    fn node(&mut self, node: &JsxSyntaxNode, namespace: Namespace) -> RenduNodeId {
        match node {
            JsxSyntaxNode::Element(element) => {
                let element_namespace = namespace_for_element(namespace, &element.name);
                let child_namespace = namespace_for_children(element_namespace, &element.name);
                let properties = self.properties(&element.attributes);
                let children = element
                    .children
                    .iter()
                    .map(|child| self.node(child, child_namespace))
                    .collect();
                let provenance = self.provenance(element.span);
                if element.component {
                    self.builder.add_node(RenduNode::Component {
                        name: RenduName::static_name(element.name.clone()),
                        properties,
                        children,
                        provenance,
                    })
                } else {
                    self.builder.add_node(RenduNode::Element {
                        tag: element.name.clone(),
                        namespace: rendu_namespace(element_namespace),
                        properties,
                        children,
                        provenance,
                    })
                }
            }
            JsxSyntaxNode::Fragment { children, span } => {
                let children = children
                    .iter()
                    .map(|child| self.node(child, namespace))
                    .collect();
                let provenance = self.provenance(*span);
                self.builder.add_node(RenduNode::Fragment {
                    children,
                    provenance,
                })
            }
            JsxSyntaxNode::Text { value, span } => {
                let provenance = self.provenance(*span);
                self.builder.add_node(RenduNode::Text {
                    value: value.clone(),
                    provenance,
                })
            }
            JsxSyntaxNode::Expression { expression, span } => {
                let expression = self.expression(expression);
                let provenance = self.provenance(*span);
                self.builder.add_node(RenduNode::Expression {
                    expression,
                    escape: RenduEscapeMode::Escaped,
                    provenance,
                })
            }
            JsxSyntaxNode::Comment { value, span } => {
                let provenance = self.provenance(*span);
                self.builder.add_node(RenduNode::Comment {
                    value: value.clone(),
                    provenance,
                })
            }
            JsxSyntaxNode::If { branches, span } => {
                let branches = branches
                    .iter()
                    .map(|branch| {
                        let condition = branch
                            .condition
                            .as_ref()
                            .map(|condition| self.expression(condition));
                        let body = branch
                            .body
                            .iter()
                            .map(|node| self.node(node, namespace))
                            .collect();
                        RenduIfBranch::new(condition, body)
                            .with_provenance(self.provenance(branch.span))
                    })
                    .collect();
                let provenance = self.provenance(*span);
                self.builder.add_node(RenduNode::If {
                    branches,
                    provenance,
                })
            }
            JsxSyntaxNode::For {
                source,
                value,
                index,
                body,
                span,
            } => {
                let source = self.expression(source);
                let value = value
                    .as_ref()
                    .map(|binding| self.binding(binding))
                    .unwrap_or_else(|| RenduBinding::new("_value"));
                let index = index.as_ref().map(|binding| self.binding(binding));
                let body = body.iter().map(|node| self.node(node, namespace)).collect();
                let provenance = self.provenance(*span);
                self.builder.add_node(RenduNode::For {
                    source,
                    value,
                    key: None,
                    index,
                    key_expression: None,
                    body,
                    provenance,
                })
            }
        }
    }

    fn expression(&mut self, expression: &JsxSyntaxExpression) -> RenduExpressionId {
        let kind = classify_expression(&expression.code, expression.synthetic);
        let provenance = self.provenance(expression.span);
        self.builder.add_expression(
            RenduExpression::new(expression.code.clone(), kind).with_provenance(provenance),
        )
    }

    fn binding(&self, binding: &JsxSyntaxBinding) -> RenduBinding {
        RenduBinding::new(binding.pattern.clone()).with_provenance(self.provenance(binding.span))
    }

    pub(super) fn provenance(&self, span: JsxSyntaxSpan) -> RenduProvenance {
        RenduProvenance::from_span(RenduSpan::new(
            self.source,
            self.position(span.start),
            self.position(span.end),
        ))
    }

    fn position(&self, offset: u32) -> RenduPosition {
        let (line, column) = self.line_index.line_col(offset as usize);
        RenduPosition::new(offset, line + 1, column + 1)
    }
}

fn classify_expression(code: &str, synthetic: bool) -> RenduExpressionKind {
    let code = code.trim();
    if synthetic {
        RenduExpressionKind::Compound
    } else if matches!(code, "true" | "false" | "null" | "undefined")
        || matches!(code.chars().next(), Some('\'' | '"' | '`'))
        || code.parse::<f64>().is_ok()
    {
        RenduExpressionKind::Literal
    } else if code.split('.').all(|part| {
        !part.is_empty()
            && part
                .chars()
                .all(|ch| ch == '_' || ch == '$' || ch.is_alphanumeric())
    }) {
        RenduExpressionKind::Reference
    } else {
        RenduExpressionKind::Compound
    }
}

fn namespace_for_element(parent: Namespace, name: &str) -> Namespace {
    match name {
        "svg" => Namespace::Svg,
        "math" => Namespace::MathMl,
        _ => parent,
    }
}

fn namespace_for_children(element: Namespace, name: &str) -> Namespace {
    if element == Namespace::Svg && name == "foreignObject" {
        Namespace::Html
    } else {
        element
    }
}

fn rendu_namespace(namespace: Namespace) -> RenduNamespace {
    match namespace {
        Namespace::Html => RenduNamespace::Html,
        Namespace::Svg => RenduNamespace::Svg,
        Namespace::MathMl => RenduNamespace::MathMl,
    }
}
