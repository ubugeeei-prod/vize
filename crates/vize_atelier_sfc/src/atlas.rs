//! Independently registered Atlas providers for Vue SFC sources.

#[path = "atlas/semantics.rs"]
mod semantics;

use vize_atelier_core::{ParserOptions, TransformOptions, parse_with_options, transform};
use vize_atlas::{
    Compilation, InputId, PlanningContext, Product, ProductId, Provider, ProviderContext,
    ProviderError, RegisterProviderError, SourceId, SourceRange, SourceRevision,
};
use vize_carton::{
    Bump, String, cstr, source_anchor::SourceAnchor, source_range::SourceRange as StableSourceRange,
};
use vize_croquis::{CroquisSemanticProduct, CroquisSemanticSnapshot};
use vize_flow::FlowProduct;
use vize_relief::{ReliefProduct, ReliefSnapshot, VueDialectInput};
use vize_rendu::RenduProduct;

use crate::croquis::{SfcCroquisOptions, analyze_sfc_descriptor};
use crate::graph_frontend::{
    lower_relief_snapshot_to_rendu_with_anchor, project_relief_snapshot_to_flow_with_anchor,
};
use crate::{SfcDescriptor, SfcGraphAdapterError, parse_sfc};
use semantics::project_template_semantics;

/// Parsed, owned SFC container descriptor.
pub struct SfcDescriptorProduct;

impl Product for SfcDescriptorProduct {
    type Value = SfcDescriptor<'static>;

    const NAME: &'static str = "sfc.descriptor";
}

/// Owned template block plus exact parent-source provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SfcTemplateSource {
    pub parent: SourceId,
    pub parent_revision: SourceRevision,
    pub range: SourceRange,
    pub name: String,
    pub text: String,
}

/// SFC template block selected from a container descriptor.
pub struct SfcTemplateProduct;

impl Product for SfcTemplateProduct {
    type Value = SfcTemplateSource;

    const NAME: &'static str = "sfc.template-source";
}

/// Parse an applicable `.vue` source without constructing downstream products.
pub struct SfcDescriptorProvider;

impl Provider for SfcDescriptorProvider {
    type Product = SfcDescriptorProduct;

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_sfc_source(context.source().name())
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<SfcDescriptor<'static>, ProviderError> {
        let source = context.source();
        parse_sfc(
            source.text(),
            crate::SfcParseOptions {
                filename: source.name().into(),
                ..Default::default()
            },
        )
        .map(SfcDescriptor::into_owned)
        .map_err(|error| ProviderError::message(error.message))
    }
}

/// Decompose the template block while retaining its parent identity/range.
pub struct SfcTemplateProvider;

impl Provider for SfcTemplateProvider {
    type Product = SfcTemplateProduct;

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_sfc_source(context.source().name())
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<SfcDescriptorProduct>()]
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<SfcTemplateSource, ProviderError> {
        let descriptor = context.get::<SfcDescriptorProduct>()?;
        let template = descriptor
            .template
            .as_ref()
            .ok_or_else(|| ProviderError::message("SFC has no template block"))?;
        let source = context.source();
        Ok(SfcTemplateSource {
            parent: source.id(),
            parent_revision: source.revision(),
            range: SourceRange::new(template.loc.start, template.loc.end),
            name: cstr!("{}#template", source.name()),
            text: template.content.as_ref().into(),
        })
    }
}

/// Parse and transform one template block into an owned Relief snapshot.
pub struct SfcReliefProvider;

impl Provider for SfcReliefProvider {
    type Product = ReliefProduct;

    fn input_dependencies(&self) -> Vec<InputId> {
        vec![InputId::of::<VueDialectInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_sfc_source(context.source().name())
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<SfcTemplateProduct>()]
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<ReliefSnapshot, ProviderError> {
        let template = context.get::<SfcTemplateProduct>()?;
        let dialect = context
            .input::<VueDialectInput>()
            .copied()
            .unwrap_or_default();
        let allocator = Bump::new();
        let (mut root, parse_errors) = parse_with_options(
            &allocator,
            &template.text,
            ParserOptions {
                dialect,
                ..Default::default()
            },
        );
        if let Some(error) = parse_errors.iter().find(|error| !error.is_recoverable()) {
            return Err(ProviderError::message(cstr!("{error:?}")));
        }
        let transformed = transform(
            &allocator,
            &mut root,
            TransformOptions {
                dialect,
                ..Default::default()
            },
            None,
        );
        if let Some(error) = transformed.errors.first() {
            return Err(ProviderError::message(cstr!("{error:?}")));
        }
        Ok(ReliefSnapshot::from_root(&root))
    }
}

/// Relief syntax to frontend-neutral Rendu for SFC sources.
pub struct SfcRenduProvider;

impl Provider for SfcRenduProvider {
    type Product = RenduProduct;

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_sfc_source(context.source().name())
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![
            ProductId::of::<SfcTemplateProduct>(),
            ProductId::of::<ReliefProduct>(),
        ]
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<<RenduProduct as Product>::Value, ProviderError> {
        let template = context.get::<SfcTemplateProduct>()?;
        let relief = context.get::<ReliefProduct>()?;
        let anchor = template_source_anchor(&template)?;
        lower_relief_snapshot_to_rendu_with_anchor(relief.as_ref(), anchor).map_err(graph_error)
    }
}

/// Relief syntax to the separate single-file Flow representation.
pub struct SfcFlowProvider;

impl Provider for SfcFlowProvider {
    type Product = FlowProduct;

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_sfc_source(context.source().name())
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![
            ProductId::of::<SfcTemplateProduct>(),
            ProductId::of::<ReliefProduct>(),
        ]
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<<FlowProduct as Product>::Value, ProviderError> {
        let template = context.get::<SfcTemplateProduct>()?;
        let relief = context.get::<ReliefProduct>()?;
        let anchor = template_source_anchor(&template)?;
        project_relief_snapshot_to_flow_with_anchor(relief.as_ref(), anchor).map_err(graph_error)
    }
}

/// SFC script plus cached Relief syntax to owned Croquis semantic facts.
pub struct SfcSemanticProvider;

impl Provider for SfcSemanticProvider {
    type Product = CroquisSemanticProduct;

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_sfc_source(context.source().name())
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![
            ProductId::of::<SfcDescriptorProduct>(),
            ProductId::of::<SfcTemplateProduct>(),
            ProductId::of::<ReliefProduct>(),
        ]
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<CroquisSemanticSnapshot, ProviderError> {
        let source_anchor = SourceAnchor::new(
            context.source().id().get(),
            context.source().revision().get(),
        );
        let descriptor = context.get::<SfcDescriptorProduct>()?;
        let template = context.get::<SfcTemplateProduct>()?;
        let relief = context.get::<ReliefProduct>()?;
        let script_semantics = analyze_sfc_descriptor(&descriptor, None, SfcCroquisOptions::full())
            .semantic_snapshot();
        let mut semantics = project_template_semantics(
            script_semantics,
            relief.as_ref(),
            template.range.start as u32,
        );
        semantics.source_anchor = Some(source_anchor);
        Ok(semantics)
    }
}

/// Register the SFC frontend's independently applicable providers.
pub fn register_atlas_providers(
    compilation: &mut Compilation,
) -> Result<(), RegisterProviderError> {
    compilation.register_provider(SfcDescriptorProvider)?;
    compilation.register_provider(SfcTemplateProvider)?;
    compilation.register_provider(SfcReliefProvider)?;
    compilation.register_provider(SfcRenduProvider)?;
    compilation.register_provider(SfcFlowProvider)?;
    compilation.register_provider(SfcSemanticProvider)
}

fn is_sfc_source(name: &str) -> bool {
    name.ends_with(".vue")
}

fn graph_error(error: SfcGraphAdapterError) -> ProviderError {
    ProviderError::message(cstr!("{error}"))
}

fn template_source_anchor(template: &SfcTemplateSource) -> Result<SourceAnchor, ProviderError> {
    let start = u32::try_from(template.range.start)
        .map_err(|_| ProviderError::message("SFC template start exceeds u32 source space"))?;
    let end = u32::try_from(template.range.end)
        .map_err(|_| ProviderError::message("SFC template end exceeds u32 source space"))?;
    Ok(
        SourceAnchor::new(template.parent.get(), template.parent_revision.get())
            .with_parent_range(StableSourceRange::new(start, end)),
    )
}
