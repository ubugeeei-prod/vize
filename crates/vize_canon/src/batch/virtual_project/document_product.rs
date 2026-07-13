//! Persistent Atlas root for one editor-facing Vue virtual document.

use std::path::Path;

use oxc_span::SourceType;
use vize_atelier_sfc::{SfcDescriptorProduct, SfcScriptSyntaxProduct, sfc_source_structure};
use vize_atlas::{
    Compilation, CompilationInputError, PlanningContext, Product, ProductId, Provider,
    ProviderContext, ProviderError, RegisterProviderError, SourceId, SourceInput, SourceInputId,
};
use vize_croquis::CroquisDocumentProduct;
use vize_module::ModuleSyntaxProduct;
use vize_relief::ReliefProduct;

use crate::batch::ImportRewriter;
use crate::virtual_ts::{VirtualTsCheckOptions, VirtualTsOptions};

use super::build::{
    descriptor_uses_jsx_script, prepend_vue_jsx_reference, virtual_ts_options_for_descriptor,
};
use super::document::{VueDocumentVirtualTs, VueDocumentVirtualTsOptions};
use super::vue_artifact_codegen::{VueArtifactInputs, generate_vue_virtual_ts_from_artifacts};
use super::vue_codegen::{GeneratedVueFile, VueCodegenOptions};

/// Per-source settings for Canon's editor-facing virtual document.
pub struct CanonVueDocumentSettingsInput;

impl SourceInput for CanonVueDocumentSettingsInput {
    type Value = VueDocumentVirtualTsOptions;

    const NAME: &'static str = "canon.vue-document.settings";
}

/// Rewritten virtual TypeScript generated from the shared SFC frontend products.
pub struct CanonVueDocumentProduct;

impl Product for CanonVueDocumentProduct {
    type Value = VueDocumentVirtualTs;

    const NAME: &'static str = "canon.vue-document";
}

/// Parser-free Canon provider for persistent editor compilations.
pub struct CanonVueDocumentProvider {
    rewriter: ImportRewriter,
}

impl CanonVueDocumentProvider {
    pub fn new() -> Self {
        Self {
            rewriter: ImportRewriter::new(),
        }
    }
}

impl Default for CanonVueDocumentProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for CanonVueDocumentProvider {
    type Product = CanonVueDocumentProduct;

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![SourceInputId::of::<CanonVueDocumentSettingsInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        context.source().name().ends_with(".vue")
    }

    fn dependencies(&self, context: &PlanningContext<'_>) -> Vec<ProductId> {
        let structure = sfc_source_structure(context.source().text());
        let mut dependencies = vec![
            ProductId::of::<SfcDescriptorProduct>(),
            ProductId::of::<CroquisDocumentProduct>(),
        ];
        if structure.has_template {
            dependencies.push(ProductId::of::<ReliefProduct>());
        }
        if structure.has_script {
            dependencies.push(ProductId::of::<SfcScriptSyntaxProduct>());
            dependencies.push(ProductId::of::<ModuleSyntaxProduct>());
        }
        dependencies
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<VueDocumentVirtualTs, ProviderError> {
        let structure = sfc_source_structure(context.source().text());
        let descriptor_artifact = context.get::<SfcDescriptorProduct>()?;
        let descriptor = descriptor_artifact
            .as_result()
            .map_err(|error| ProviderError::message(error.message.clone()))?;
        let semantics = context.get::<CroquisDocumentProduct>()?;
        let syntax = if structure.has_template {
            Some(context.get::<ReliefProduct>()?)
        } else {
            None
        };
        let modules = if structure.has_script {
            Some(context.get::<ModuleSyntaxProduct>()?)
        } else {
            None
        };
        let script_syntax = if structure.has_script {
            Some(context.get::<SfcScriptSyntaxProduct>()?)
        } else {
            None
        };
        let options = context
            .source_input::<CanonVueDocumentSettingsInput>()
            .copied()
            .unwrap_or_default();
        let path = Path::new(context.source().name());
        let base_options = VirtualTsOptions::default();
        let effective_options = virtual_ts_options_for_descriptor(&base_options, descriptor);
        let use_tsx_virtual = descriptor_uses_jsx_script(descriptor);
        let source_type = if use_tsx_virtual {
            SourceType::tsx()
        } else {
            SourceType::ts()
        };
        let GeneratedVueFile {
            mut code,
            mut mappings,
            ..
        } = generate_vue_virtual_ts_from_artifacts(
            path,
            context.source().text(),
            VueArtifactInputs {
                descriptor,
                syntax: syntax.as_deref().and_then(Option::as_ref),
                semantics: semantics.as_ref(),
                modules: modules.as_deref(),
                script_syntax: script_syntax.as_deref(),
                extra_template_referenced_names: None,
            },
            &effective_options,
            VueCodegenOptions {
                check_options: VirtualTsCheckOptions::default(),
                preserve_unused_diagnostics: false,
                options_api: options.options_api,
                legacy_vue2: options.legacy_vue2,
                dialect: vize_carton::config::VueVersion::default(),
                template_syntax: vize_relief::TemplateSyntaxMode::default(),
                hoist_shared_preamble: false,
            },
        )
        .map_err(graph_error)?;
        if use_tsx_virtual {
            prepend_vue_jsx_reference(&mut code, &mut mappings);
        }
        let rewritten = self.rewriter.rewrite(&code, source_type);
        Ok(VueDocumentVirtualTs {
            code: rewritten.code,
            pre_rewrite_code: code,
            mappings,
            import_source_map: rewritten.source_map,
            source_type,
            virtual_suffix: if use_tsx_virtual { ".tsx" } else { ".ts" },
            descriptor: descriptor_artifact,
            script_syntax,
        })
    }
}

pub fn register_canon_vue_document_provider(
    compilation: &mut Compilation,
) -> Result<(), RegisterProviderError> {
    if !compilation.has_provider::<CanonVueDocumentProduct>() {
        compilation.register_provider(CanonVueDocumentProvider::new())?;
    }
    Ok(())
}

pub fn install_canon_vue_document_options(
    compilation: &mut Compilation,
    source: SourceId,
    options: VueDocumentVirtualTsOptions,
) -> Result<(), CompilationInputError> {
    compilation
        .set_source_input::<CanonVueDocumentSettingsInput>(source, options)
        .map(|_| ())
}

fn graph_error(error: impl std::fmt::Display) -> ProviderError {
    ProviderError::message(vize_carton::cstr!("{error}"))
}
