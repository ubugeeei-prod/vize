//! Independently registered Atlas providers for JSX and TSX sources.

use vize_atlas::{
    Compilation, ObservationKind, PlanningContext, Product, ProductId, Provider, ProviderContext,
    ProviderError, RegisterProviderError, SourceRange,
};
use vize_carton::{cstr, source_anchor::SourceAnchor};
use vize_croquis::{
    CroquisSemanticProduct, CroquisSemanticSnapshot, CroquisSemanticSnapshotBuilder,
    SemanticSourceRange,
};
use vize_flow::FlowProduct;
use vize_rendu::RenduProduct;

use crate::{
    JsxLang, JsxSyntaxAttribute, JsxSyntaxNode, JsxSyntaxSnapshot, JsxSyntaxSpan, Severity,
    snapshot_jsx_named,
};

/// Owned OXC-derived JSX syntax product. No Relief tree is constructed.
pub struct JsxSyntaxProduct;

impl Product for JsxSyntaxProduct {
    type Value = JsxSyntaxSnapshot;

    const NAME: &'static str = "jsx.syntax";
}

/// Parse an applicable `.jsx` or `.tsx` source into owned syntax.
pub struct JsxSyntaxProvider;

impl Provider for JsxSyntaxProvider {
    type Product = JsxSyntaxProduct;

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_jsx_source(context.source().name())
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<JsxSyntaxSnapshot, ProviderError> {
        let source = context.source();
        let mut snapshot = snapshot_jsx_named(
            source.name(),
            source.text(),
            JsxLang::from_path(source.name()),
        );
        snapshot.source_anchor = Some(SourceAnchor::new(
            source.id().get(),
            source.revision().get(),
        ));
        for diagnostic in &snapshot.diagnostics {
            context.observe(
                ObservationKind::Diagnostic,
                match diagnostic.severity {
                    Severity::Error => "jsx.parse.error",
                    Severity::Warning => "jsx.parse.warning",
                },
                diagnostic.message.as_str(),
                Some(SourceRange::new(
                    diagnostic.start as usize,
                    diagnostic.end as usize,
                )),
            );
        }
        if snapshot.panicked {
            context.observe(
                ObservationKind::Fallback,
                "jsx.parse.recovery",
                "the JSX parser recovered from an internal panic",
                None,
            );
        }
        Ok(snapshot)
    }
}

/// Direct JSX syntax to frontend-neutral Rendu provider.
pub struct JsxRenduProvider;

impl Provider for JsxRenduProvider {
    type Product = RenduProduct;

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_jsx_source(context.source().name())
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<JsxSyntaxProduct>()]
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<<RenduProduct as Product>::Value, ProviderError> {
        context
            .get::<JsxSyntaxProduct>()?
            .lower_rendu()
            .map_err(|error| ProviderError::message(cstr!("{error}")))
    }
}

/// JSX syntax to owned Croquis semantic facts without a second parse.
pub struct JsxSemanticProvider;

impl Provider for JsxSemanticProvider {
    type Product = CroquisSemanticProduct;

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_jsx_source(context.source().name())
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<JsxSyntaxProduct>()]
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<CroquisSemanticSnapshot, ProviderError> {
        let syntax = context.get::<JsxSyntaxProduct>()?;
        Ok(project_semantics(syntax.as_ref()))
    }
}

/// JSX syntax to the peer frontend-neutral control/data/effect graph.
pub struct JsxFlowProvider;

impl Provider for JsxFlowProvider {
    type Product = FlowProduct;

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_jsx_source(context.source().name())
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<JsxSyntaxProduct>()]
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<<FlowProduct as Product>::Value, ProviderError> {
        let syntax = context.get::<JsxSyntaxProduct>()?;
        crate::project_jsx_syntax_to_flow(syntax.as_ref())
            .map_err(|error| ProviderError::message(cstr!("{error}")))
    }
}

/// Register the JSX frontend's independent product providers.
pub fn register_atlas_providers(
    compilation: &mut Compilation,
) -> Result<(), RegisterProviderError> {
    compilation.register_provider(JsxSyntaxProvider)?;
    compilation.register_provider(JsxRenduProvider)?;
    compilation.register_provider(JsxSemanticProvider)?;
    compilation.register_provider(JsxFlowProvider)
}

fn is_jsx_source(name: &str) -> bool {
    name.ends_with(".jsx") || name.ends_with(".tsx")
}

fn project_semantics(snapshot: &JsxSyntaxSnapshot) -> CroquisSemanticSnapshot {
    let mut builder = CroquisSemanticSnapshotBuilder::new();
    for node in &snapshot.roots {
        collect_node(node, &mut builder);
    }
    let mut semantics = builder.finish();
    semantics.source_anchor = snapshot.source_anchor;
    semantics
}

fn collect_node(node: &JsxSyntaxNode, builder: &mut CroquisSemanticSnapshotBuilder) {
    match node {
        JsxSyntaxNode::Element(element) => {
            if element.component {
                builder.add_component_usage(
                    &element.name,
                    range(element.span),
                    0,
                    element
                        .attributes
                        .iter()
                        .any(|attribute| matches!(attribute, JsxSyntaxAttribute::Spread { .. })),
                );
            }
            for attribute in &element.attributes {
                match attribute {
                    JsxSyntaxAttribute::Attribute {
                        value: crate::JsxSyntaxAttributeValue::Expression(expression),
                        ..
                    }
                    | JsxSyntaxAttribute::Spread { expression, .. } => builder
                        .add_template_expression(
                            &expression.code,
                            "jsx-attribute",
                            range(expression.span),
                            0,
                        ),
                    _ => {}
                }
            }
            collect_nodes(&element.children, builder);
        }
        JsxSyntaxNode::Fragment { children, .. } => collect_nodes(children, builder),
        JsxSyntaxNode::Expression { expression, .. } => builder.add_template_expression(
            &expression.code,
            "jsx-expression",
            range(expression.span),
            0,
        ),
        JsxSyntaxNode::If { branches, .. } => {
            for branch in branches {
                if let Some(condition) = &branch.condition {
                    builder.add_template_expression(
                        &condition.code,
                        "jsx-condition",
                        range(condition.span),
                        0,
                    );
                }
                collect_nodes(&branch.body, builder);
            }
        }
        JsxSyntaxNode::For {
            source,
            value,
            index,
            body,
            ..
        } => {
            builder.add_template_expression(&source.code, "jsx-iteration", range(source.span), 0);
            for binding in [value.as_ref(), index.as_ref()].into_iter().flatten() {
                builder.add_binding(
                    &binding.pattern,
                    "iteration",
                    "template-local",
                    Some(range(binding.span)),
                );
            }
            collect_nodes(body, builder);
        }
        JsxSyntaxNode::Text { .. } | JsxSyntaxNode::Comment { .. } => {}
    }
}

fn collect_nodes(nodes: &[JsxSyntaxNode], builder: &mut CroquisSemanticSnapshotBuilder) {
    for node in nodes {
        collect_node(node, builder);
    }
}

const fn range(span: JsxSyntaxSpan) -> SemanticSourceRange {
    SemanticSourceRange::new(span.start, span.end)
}
