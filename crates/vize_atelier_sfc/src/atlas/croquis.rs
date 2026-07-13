//! Complete Croquis document provider for SFC sources.

#[path = "croquis/settings.rs"]
mod settings;

use vize_atlas::{
    PlanningContext, ProductId, Provider, ProviderContext, ProviderError, SourceId, SourceInputId,
    SourceRevision,
};
use vize_carton::{Bump, source_anchor::SourceAnchor, source_range::SourceRange};
use vize_croquis::{CroquisDocument, CroquisDocumentProduct, CroquisSourceSegment};
use vize_relief::ReliefProduct;

use crate::croquis::{
    SfcCroquisOptions, analyze_sfc_descriptor, analyze_sfc_descriptor_resolved,
    analyze_sfc_descriptor_resolved_for_canon_compatibility,
    analyze_sfc_descriptor_with_context_legacy_vue2,
    analyze_sfc_descriptor_with_context_options_api,
};

use super::{SfcDescriptorProduct, is_sfc_source};
pub use settings::{
    SfcCroquisMode, SfcCroquisRequest, SfcCroquisSettings, SfcCroquisSettingsInput,
    SfcResolvedPropsPolicy,
};

#[cfg(test)]
#[path = "croquis/tests.rs"]
mod tests;

/// SFC descriptor plus parse-only Relief syntax to complete semantic analysis.
pub struct SfcCroquisProvider;

impl Provider for SfcCroquisProvider {
    type Product = CroquisDocumentProduct;

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![SourceInputId::of::<SfcCroquisSettingsInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_sfc_source(context.source().name())
    }

    fn dependencies(&self, context: &PlanningContext<'_>) -> Vec<ProductId> {
        let mode = context
            .source_input::<SfcCroquisSettingsInput>()
            .map(|request| request.mode)
            .unwrap_or_default();
        let mut dependencies = vec![ProductId::of::<SfcDescriptorProduct>()];
        if mode != SfcCroquisMode::Declaration {
            dependencies.push(ProductId::of::<ReliefProduct>());
        }
        dependencies
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<CroquisDocument, ProviderError> {
        let artifact = context.get::<SfcDescriptorProduct>()?;
        let source_id = context.source().id();
        let source_revision = context.source().revision();
        let root_anchor = SourceAnchor::new(source_id.get(), source_revision.get());
        let Some(descriptor) = artifact.descriptor() else {
            return Ok(CroquisDocument::new(Default::default()).with_source_anchor(root_anchor));
        };
        let request = context.source_input::<SfcCroquisSettingsInput>().cloned();
        let mode = request
            .as_ref()
            .map(|request| request.mode)
            .unwrap_or_default();
        let resolved_filename = request
            .as_ref()
            .and_then(|request| request.resolved_filename.clone());
        let resolved_policy = request
            .as_ref()
            .map(|request| request.resolved_props_policy)
            .unwrap_or_default();
        let allocator = Bump::new();
        let root = if mode == SfcCroquisMode::Declaration {
            None
        } else {
            let syntax = context.get::<ReliefProduct>()?;
            match (descriptor.template.as_ref(), syntax.as_ref()) {
                (Some(_), Some(syntax)) if !syntax.has_fatal_diagnostics() => {
                    Some(syntax.snapshot().materialize(&allocator))
                }
                (Some(_), Some(_)) | (None, None) => None,
                _ => {
                    return Err(ProviderError::message(
                        "SFC descriptor and Relief syntax disagree about template presence",
                    ));
                }
            }
        };
        let analysis = analyze_document(
            descriptor,
            root.as_ref(),
            mode,
            resolved_filename.as_deref(),
            resolved_policy,
        );
        let mut semantics = analysis.semantic_snapshot();
        if let Some(template) = descriptor.template.as_ref() {
            absolutize_template_ranges(&mut semantics, template.loc.start as u32);
        }
        let mut document = CroquisDocument::new(analysis)
            .with_source_anchor(root_anchor)
            .with_semantic_snapshot(semantics);

        if let Some(block) = descriptor.script.as_ref() {
            document = add_segment(
                document,
                "script",
                block.content.as_ref(),
                block.lang.as_deref(),
                source_id,
                source_revision,
                block.loc.start,
                block.loc.end,
            )?;
        }
        if let Some(block) = descriptor.script_setup.as_ref() {
            document = add_segment(
                document,
                "script-setup",
                block.content.as_ref(),
                block.lang.as_deref(),
                source_id,
                source_revision,
                block.loc.start,
                block.loc.end,
            )?;
        }
        if let Some(block) = descriptor.template.as_ref() {
            document = add_segment(
                document,
                "template",
                block.content.as_ref(),
                None,
                source_id,
                source_revision,
                block.loc.start,
                block.loc.end,
            )?;
        }
        Ok(document)
    }
}

fn analyze_document(
    descriptor: &crate::SfcDescriptor<'_>,
    root: Option<&vize_relief::RootNode<'_>>,
    mode: SfcCroquisMode,
    resolved_filename: Option<&str>,
    resolved_policy: SfcResolvedPropsPolicy,
) -> vize_croquis::Croquis {
    if let Some(filename) = resolved_filename {
        let analyze = match resolved_policy {
            SfcResolvedPropsPolicy::BeforeTemplate => analyze_sfc_descriptor_resolved,
            SfcResolvedPropsPolicy::PreserveCanonAfterTemplate => {
                analyze_sfc_descriptor_resolved_for_canon_compatibility
            }
        };
        return analyze(
            descriptor,
            root,
            SfcCroquisOptions::full(),
            matches!(mode, SfcCroquisMode::OptionsApi),
            matches!(mode, SfcCroquisMode::LegacyVue2),
            filename,
        )
        .croquis;
    }
    match mode {
        SfcCroquisMode::Full => analyze_sfc_descriptor(descriptor, root, SfcCroquisOptions::full()),
        SfcCroquisMode::OptionsApi => {
            analyze_sfc_descriptor_with_context_options_api(
                descriptor,
                root,
                SfcCroquisOptions::full(),
            )
            .croquis
        }
        SfcCroquisMode::LegacyVue2 => {
            analyze_sfc_descriptor_with_context_legacy_vue2(
                descriptor,
                root,
                SfcCroquisOptions::full(),
            )
            .croquis
        }
        SfcCroquisMode::Declaration => {
            analyze_sfc_descriptor(descriptor, None, SfcCroquisOptions::for_declaration())
        }
    }
}

fn absolutize_template_ranges(semantics: &mut vize_croquis::CroquisSemanticSnapshot, offset: u32) {
    let mut template_scopes = vize_carton::FxHashSet::default();
    template_scopes.extend(
        semantics
            .template_expressions
            .iter()
            .map(|expression| expression.scope_id),
    );
    template_scopes.extend(
        semantics
            .component_usages
            .iter()
            .map(|usage| usage.scope_id),
    );
    template_scopes.extend(
        semantics
            .scopes
            .iter()
            .filter(|scope| matches!(scope.kind, "v-for" | "v-slot" | "event"))
            .map(|scope| scope.id),
    );
    loop {
        let before = template_scopes.len();
        for scope in &semantics.scopes {
            if template_scopes.contains(&scope.id) {
                template_scopes.extend(scope.parent_ids.iter().copied().filter(|id| *id != 0));
            }
        }
        if template_scopes.len() == before {
            break;
        }
    }
    for scope in &mut semantics.scopes {
        if template_scopes.contains(&scope.id) {
            shift_range(&mut scope.range, offset);
            for binding in &mut scope.bindings {
                binding.declaration_offset = binding.declaration_offset.saturating_add(offset);
            }
        }
    }
    for expression in &mut semantics.template_expressions {
        shift_range(&mut expression.range, offset);
    }
    for usage in &mut semantics.component_usages {
        shift_range(&mut usage.range, offset);
        for property in &mut usage.props {
            shift_range(&mut property.range, offset);
        }
        for event in &mut usage.events {
            shift_range(&mut event.range, offset);
        }
        for slot in &mut usage.slots {
            shift_range(&mut slot.range, offset);
        }
    }
}

fn shift_range(range: &mut vize_croquis::SemanticSourceRange, offset: u32) {
    range.start = range.start.saturating_add(offset);
    range.end = range.end.saturating_add(offset);
}

#[allow(clippy::too_many_arguments)]
fn add_segment(
    document: CroquisDocument,
    role: &'static str,
    text: &str,
    language: Option<&str>,
    source: SourceId,
    revision: SourceRevision,
    start: usize,
    end: usize,
) -> Result<CroquisDocument, ProviderError> {
    let start = u32::try_from(start)
        .map_err(|_| ProviderError::message("SFC block start exceeds u32 source space"))?;
    let end = u32::try_from(end)
        .map_err(|_| ProviderError::message("SFC block end exceeds u32 source space"))?;
    let anchor = SourceAnchor::new(source.get(), revision.get())
        .with_parent_range(SourceRange::new(start, end));
    let mut segment = CroquisSourceSegment::new(role, text, anchor);
    if let Some(language) = language {
        segment = segment.with_language(language);
    }
    Ok(document.with_source(segment))
}
