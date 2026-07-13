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
use vize_carton::{FxHashMap, cstr, source_anchor::SourceAnchor};
use vize_croquis::{
    CroquisDocument, CroquisDocumentProduct, CroquisSemanticSnapshot,
    CroquisSemanticSnapshotBuilder, CroquisSourceSegment, SemanticScopeBindingSnapshot,
    SemanticSourceRange,
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

/// Register only the JSX frontend's independent product providers.
pub fn register_atlas_providers(
    compilation: &mut Compilation,
) -> Result<(), RegisterProviderError> {
    compilation.register_provider(JsxSyntaxProvider)?;
    compilation.register_provider(JsxModuleSyntaxProvider)?;
    compilation.register_provider(JsxRenduProvider)?;
    compilation.register_provider(JsxCroquisProvider)?;
    compilation.register_provider(JsxFlowProvider)?;
    compilation.register_provider(JsxRenderModuleProvider)?;
    compilation.register_provider(JsxCompileProvider)?;
    Ok(())
}

fn is_jsx_source(name: &str) -> bool {
    let name = name.split(['?', '#']).next().unwrap_or(name);
    name.ends_with(".jsx") || name.ends_with(".tsx")
}

fn project_semantics(snapshot: &JsxSyntaxSnapshot) -> CroquisSemanticSnapshot {
    let script_semantics = semantics_with_module_scopes(snapshot);
    let mut builder = CroquisSemanticSnapshotBuilder::from_snapshot(script_semantics.clone());
    for node in &snapshot.roots {
        collect_node(node, &script_semantics, &mut builder);
    }
    let mut semantics = builder.finish();
    semantics.source_anchor = snapshot.source_anchor;
    semantics
}

fn semantics_with_module_scopes(snapshot: &JsxSyntaxSnapshot) -> CroquisSemanticSnapshot {
    let semantics = snapshot.analysis().semantic_snapshot();
    let mut next_scope_id = semantics
        .scopes
        .iter()
        .map(|scope| scope.id)
        .max()
        .map_or(0, |id| id + 1);
    let mut builder = CroquisSemanticSnapshotBuilder::from_snapshot(semantics.clone());
    for module in &snapshot.module().modules {
        let mut scope_ids = FxHashMap::default();
        for function in &module.operations.functions {
            scope_ids.insert(function.id, next_scope_id);
            next_scope_id += 1;
        }
        for function in &module.operations.functions {
            let scope_id = scope_ids[&function.id];
            let parent_id = function
                .parent
                .and_then(|parent| scope_ids.get(&parent).copied())
                .unwrap_or_else(|| {
                    scope_id_for_span(
                        &semantics,
                        JsxSyntaxSpan::new(function.span.start, function.span.end),
                    )
                });
            let bindings = function
                .local_bindings
                .iter()
                .map(|name| {
                    SemanticScopeBindingSnapshot::new(
                        name,
                        "callbackLocal",
                        function.span.start,
                        function.references.contains(name),
                        false,
                    )
                })
                .collect();
            builder.add_scope(
                scope_id,
                vec![parent_id],
                "callback",
                SemanticSourceRange::new(function.span.start, function.span.end),
                bindings,
            );
        }
    }
    builder.finish()
}

fn collect_node(
    node: &JsxSyntaxNode,
    semantics: &CroquisSemanticSnapshot,
    builder: &mut CroquisSemanticSnapshotBuilder,
) {
    match node {
        JsxSyntaxNode::Element(element) => {
            let scope_id = scope_id_for_span(semantics, element.span);
            let usage = element.component.then(|| {
                builder.add_component_usage(
                    &element.name,
                    range(element.span),
                    scope_id,
                    element
                        .attributes
                        .iter()
                        .any(|attribute| matches!(attribute, JsxSyntaxAttribute::Spread { .. })),
                )
            });
            for attribute in &element.attributes {
                match attribute {
                    JsxSyntaxAttribute::Attribute {
                        name, value, span, ..
                    } => {
                        if let crate::JsxSyntaxAttributeValue::Expression(expression) = value {
                            builder.add_template_expression(
                                &expression.code,
                                "jsx-attribute",
                                range(expression.span),
                                scope_id_for_span(semantics, expression.span),
                            );
                        }
                        if let Some(usage) = usage {
                            add_component_attribute(builder, usage, name, value, *span);
                        }
                    }
                    JsxSyntaxAttribute::Spread { expression, .. } => builder
                        .add_template_expression(
                            &expression.code,
                            "jsx-attribute",
                            range(expression.span),
                            scope_id_for_span(semantics, expression.span),
                        ),
                }
            }
            if let Some(usage) = usage.filter(|_| !element.children.is_empty()) {
                builder.add_component_slot(usage, "default", range(element.span));
            }
            collect_nodes(&element.children, semantics, builder);
        }
        JsxSyntaxNode::Fragment { children, .. } => collect_nodes(children, semantics, builder),
        JsxSyntaxNode::Expression { expression, .. } => builder.add_template_expression(
            &expression.code,
            "jsx-expression",
            range(expression.span),
            scope_id_for_span(semantics, expression.span),
        ),
        JsxSyntaxNode::If { branches, .. } => {
            for branch in branches {
                if let Some(condition) = &branch.condition {
                    builder.add_template_expression(
                        &condition.code,
                        "jsx-condition",
                        range(condition.span),
                        scope_id_for_span(semantics, condition.span),
                    );
                }
                collect_nodes(&branch.body, semantics, builder);
            }
        }
        JsxSyntaxNode::For { source, body, .. } => {
            builder.add_template_expression(
                &source.code,
                "jsx-iteration",
                range(source.span),
                scope_id_for_span(semantics, source.span),
            );
            collect_nodes(body, semantics, builder);
        }
        JsxSyntaxNode::Text { .. } | JsxSyntaxNode::Comment { .. } => {}
    }
}

fn collect_nodes(
    nodes: &[JsxSyntaxNode],
    semantics: &CroquisSemanticSnapshot,
    builder: &mut CroquisSemanticSnapshotBuilder,
) {
    for node in nodes {
        collect_node(node, semantics, builder);
    }
}

fn add_component_attribute(
    builder: &mut CroquisSemanticSnapshotBuilder,
    usage: usize,
    name: &str,
    value: &crate::JsxSyntaxAttributeValue,
    span: JsxSyntaxSpan,
) {
    let value_text: Option<&str> = match value {
        crate::JsxSyntaxAttributeValue::Presence => None,
        crate::JsxSyntaxAttributeValue::Static { value, .. } => Some(value.as_ref()),
        crate::JsxSyntaxAttributeValue::Expression(expression) => Some(expression.code.as_ref()),
    };
    if let Some(event) = jsx_event_name(name) {
        builder.add_component_event(usage, event.as_str(), value_text, range(span));
    } else {
        builder.add_component_prop(
            usage,
            name,
            value_text,
            range(span),
            matches!(value, crate::JsxSyntaxAttributeValue::Expression(_)),
        );
    }
}

fn jsx_event_name(name: &str) -> Option<vize_carton::String> {
    let event = name.strip_prefix("on")?;
    let first = event.chars().next()?;
    if !first.is_ascii_uppercase() {
        return None;
    }
    let mut normalized = vize_carton::String::with_capacity(event.len());
    normalized.push(first.to_ascii_lowercase());
    normalized.push_str(&event[first.len_utf8()..]);
    Some(normalized)
}

fn scope_id_for_span(semantics: &CroquisSemanticSnapshot, span: JsxSyntaxSpan) -> u32 {
    semantics
        .scopes
        .iter()
        .filter(|scope| scope.range.start <= span.start && span.end <= scope.range.end)
        .min_by_key(|scope| scope.range.end.saturating_sub(scope.range.start))
        .or_else(|| semantics.scopes.first())
        .map_or(0, |scope| scope.id)
}

const fn range(span: JsxSyntaxSpan) -> SemanticSourceRange {
    SemanticSourceRange::new(span.start, span.end)
}
