//! Independently registered Atlas providers for JSX and TSX sources.

#[path = "atlas/compile.rs"]
mod compile;
#[path = "atlas/render.rs"]
mod render;

pub use compile::{
    JsxCompileArtifact, JsxCompileProduct, JsxCompileProvider, JsxCompileRequest,
    JsxCompileSettings, JsxCompileSettingsInput, compile_jsx_with_atlas,
};
pub use render::{JsxRenderModule, JsxRenderModuleProduct, JsxRenderModuleProvider, JsxRenderRoot};

use vize_atlas::{
    Compilation, ObservationKind, PlanningContext, Product, ProductId, Provider, ProviderContext,
    ProviderError, RegisterProviderError, SourceInputId, SourceRange,
};
use vize_carton::{cstr, source_anchor::SourceAnchor};
use vize_croquis::{
    CroquisDocument, CroquisDocumentProduct, CroquisSemanticProduct, CroquisSemanticSnapshot,
    CroquisSemanticSnapshotBuilder, CroquisSourceSegment, SemanticSourceRange,
};
use vize_flow::FlowProduct;
use vize_module::{ModuleDocument, ModuleSyntaxProduct, append_module_flow};
use vize_rendu::{RenduModule, RenduProduct};

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

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![SourceInputId::of::<JsxCompileSettingsInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_jsx_source(context.source().name())
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<JsxSyntaxSnapshot, ProviderError> {
        let source = context.source();
        let lang = context
            .source_input::<JsxCompileSettingsInput>()
            .and_then(|request| request.lang)
            .unwrap_or_else(|| JsxLang::from_path(source.name()));
        let mut snapshot = snapshot_jsx_named(source.name(), source.text(), lang);
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
        let syntax = context.get::<JsxSyntaxProduct>()?;
        let roots = (0..syntax.roots.len())
            .map(|index| {
                syntax
                    .lower_rendu_root(index)
                    .ok_or_else(|| ProviderError::message("JSX root metadata is misaligned"))?
                    .map_err(|error| ProviderError::message(cstr!("{error}")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if roots.is_empty() {
            return vize_rendu::RenduBuilder::new()
                .finish()
                .map(RenduModule::from_root)
                .map_err(|error| ProviderError::message(cstr!("{error}")));
        }
        Ok(RenduModule::new(roots))
    }
}

/// JSX syntax to the complete Croquis document without a second parse.
pub struct JsxCroquisProvider;

impl Provider for JsxCroquisProvider {
    type Product = CroquisDocumentProduct;

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_jsx_source(context.source().name())
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<JsxSyntaxProduct>()]
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<CroquisDocument, ProviderError> {
        let syntax = context.get::<JsxSyntaxProduct>()?;
        let anchor = syntax.source_anchor.ok_or_else(|| {
            ProviderError::message("Atlas JSX syntax is missing its source anchor")
        })?;
        let role = match syntax.lang {
            JsxLang::Jsx => "jsx",
            JsxLang::Tsx => "tsx",
        };
        Ok(CroquisDocument::from_shared(syntax.shared_analysis())
            .with_source_anchor(anchor)
            .with_semantic_snapshot(project_semantics(syntax.as_ref()))
            .with_source(CroquisSourceSegment::new(
                role,
                syntax.source.as_ref(),
                anchor,
            )))
    }
}

/// Reuse the JSX frontend's one OXC parse for neutral module facts and CFG.
pub struct JsxModuleSyntaxProvider;

impl Provider for JsxModuleSyntaxProvider {
    type Product = ModuleSyntaxProduct;

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_jsx_source(context.source().name())
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<JsxSyntaxProduct>()]
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<ModuleDocument, ProviderError> {
        let syntax = context.get::<JsxSyntaxProduct>()?;
        let mut module = syntax.module().clone();
        let anchor = SourceAnchor::new(
            context.source().id().get(),
            context.source().revision().get(),
        );
        for syntax in &mut module.modules {
            syntax.source_anchor = Some(anchor);
        }
        Ok(module)
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
        vec![
            ProductId::of::<JsxSyntaxProduct>(),
            ProductId::of::<ModuleSyntaxProduct>(),
        ]
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<<FlowProduct as Product>::Value, ProviderError> {
        let syntax = context.get::<JsxSyntaxProduct>()?;
        let modules = context.get::<ModuleSyntaxProduct>()?;
        let mut graph = crate::project_jsx_syntax_to_flow(syntax.as_ref())
            .map_err(|error| ProviderError::message(cstr!("{error}")))?;
        append_module_flow(&modules, &mut graph)
            .map_err(|error| ProviderError::message(cstr!("{error}")))?;
        Ok(graph)
    }
}

/// Register the JSX frontend's independent product providers.
pub fn register_atlas_providers(
    compilation: &mut Compilation,
) -> Result<(), RegisterProviderError> {
    compilation.register_provider(JsxSyntaxProvider)?;
    compilation.register_provider(JsxModuleSyntaxProvider)?;
    compilation.register_provider(JsxRenduProvider)?;
    compilation.register_provider(JsxCroquisProvider)?;
    compilation.register_provider(JsxFlowProvider)?;
    compilation.register_provider(JsxRenderModuleProvider)?;
    vize_atelier_dom::register_atlas_provider(compilation)?;
    vize_atelier_ssr::register_atlas_provider(compilation)?;
    vize_atelier_vapor::register_atlas_provider(compilation)?;
    compilation.register_provider(JsxCompileProvider)?;
    if !compilation.has_provider::<CroquisSemanticProduct>() {
        vize_croquis::register_semantic_projection(compilation)?;
    }
    Ok(())
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
