//! Production Canon document recipe over one shared Atlas compilation.

use std::path::{Path, PathBuf};

use oxc_span::SourceType;
use rayon::prelude::*;
use vize_atelier_sfc::{
    SfcCompileOptions, SfcCompileRequest, SfcCompileSettings, SfcCroquisMode, SfcCroquisSettings,
    SfcDescriptorProduct, SfcResolvedPropsPolicy,
};
use vize_atlas::{
    Compilation, PlanningContext, ProductId, Provider, ProviderContext, ProviderError, Shared,
    SourceId,
};
use vize_carton::{FxHashMap, ToCompactString, cstr};
use vize_croquis::CroquisDocumentProduct;
use vize_flow::FlowProduct;
use vize_relief::{ReliefProduct, VueDialectInput};

use crate::batch::declaration_path::is_declaration_file;
use crate::batch::error::{CorsaError, CorsaResult};
use crate::batch::import_rewriter::ImportRewriter;
use crate::virtual_ts::{VirtualTsCheckOptions, VirtualTsOptions};

use super::VirtualProject;
use super::art_usage::{ArtTemplateUsageProduct, ArtTemplateUsageProvider};
use super::artifact_product::{CanonTypedDocumentArtifact, CanonTypedDocumentProduct};
use super::artifact_source::{RegisteredSource, is_jsx, is_vue};
use super::build::{
    RegisteredFile, VirtualBuildContext, build_script_registered_file,
    build_vue_registered_file_from_artifacts, source_type_for_path,
};
use super::jsx_build::build_jsx_registered_file;

#[derive(Clone)]
struct CanonGraphSettings {
    project_root: PathBuf,
    virtual_root: PathBuf,
    virtual_ts_options: VirtualTsOptions,
    virtual_ts_check_options: VirtualTsCheckOptions,
    preserve_unused_diagnostics: bool,
    options_api: bool,
    legacy_vue2: bool,
    jsx_typecheck: bool,
    dialect: vize_carton::config::VueVersion,
    template_syntax: vize_relief::TemplateSyntaxMode,
    experimental_in_tag_comments: bool,
    source_paths: FxHashMap<SourceId, PathBuf>,
    source_types: FxHashMap<SourceId, SourceType>,
}

impl CanonGraphSettings {
    fn from_project(project: &VirtualProject) -> Self {
        Self {
            project_root: project.project_root.clone(),
            virtual_root: project.virtual_root.clone(),
            virtual_ts_options: project.virtual_ts_options.clone(),
            virtual_ts_check_options: project.virtual_ts_check_options,
            preserve_unused_diagnostics: project.tsconfig_preserves_unused_diagnostics(),
            options_api: project.options_api,
            legacy_vue2: project.legacy_vue2,
            jsx_typecheck: project.jsx_typecheck,
            dialect: project.dialect,
            template_syntax: project.template_syntax,
            experimental_in_tag_comments: project.experimental_in_tag_comments,
            source_paths: FxHashMap::default(),
            source_types: FxHashMap::default(),
        }
    }

    fn context<'a>(&'a self, rewriter: &'a ImportRewriter) -> VirtualBuildContext<'a> {
        VirtualBuildContext {
            project_root: &self.project_root,
            virtual_root: &self.virtual_root,
            virtual_ts_options: &self.virtual_ts_options,
            virtual_ts_check_options: self.virtual_ts_check_options,
            preserve_unused_diagnostics: self.preserve_unused_diagnostics,
            options_api: self.options_api,
            legacy_vue2: self.legacy_vue2,
            dialect: self.dialect,
            template_syntax: self.template_syntax,
            rewriter,
        }
    }

    fn path(&self, source: SourceId) -> Option<&Path> {
        self.source_paths.get(&source).map(PathBuf::as_path)
    }

    fn croquis_mode(&self) -> SfcCroquisMode {
        if self.legacy_vue2
            || matches!(
                self.dialect,
                vize_carton::config::VueVersion::V2 | vize_carton::config::VueVersion::V2_7
            )
        {
            SfcCroquisMode::LegacyVue2
        } else if self.options_api {
            SfcCroquisMode::OptionsApi
        } else {
            SfcCroquisMode::Full
        }
    }
}

pub(super) fn build_registered_sources(
    project: &VirtualProject,
    sources: Vec<RegisteredSource>,
) -> CorsaResult<Vec<RegisteredFile>> {
    if sources.is_empty() {
        return Ok(Vec::new());
    }
    let (compilation, identities) = prepare_compilation(project, &sources)?;
    let snapshot = compilation.snapshot();
    identities
        .into_par_iter()
        .map(|source| {
            let mut session = snapshot.query_session();
            let outcome = session
                .query::<CanonTypedDocumentProduct>(source)
                .map_err(graph_error)?;
            outcome.value().to_corsa_result()
        })
        .collect()
}

fn prepare_compilation(
    project: &VirtualProject,
    sources: &[RegisteredSource],
) -> CorsaResult<(Compilation, Vec<SourceId>)> {
    let mut compilation = Compilation::new();
    let mut settings = CanonGraphSettings::from_project(project);
    let mut identities = Vec::with_capacity(sources.len());
    for source in sources {
        let id = compilation
            .add_source(
                source.path.to_string_lossy().as_ref(),
                source.content.as_str(),
            )
            .map_err(graph_error)?;
        settings.source_paths.insert(id, source.path.clone());
        if let Some(source_type) = source.source_type {
            settings.source_types.insert(id, source_type);
        }
        identities.push(id);
    }
    configure_compilation(&mut compilation, &settings, &identities)?;
    vize_atelier_sfc::register_atlas_providers(&mut compilation).map_err(graph_error)?;
    if settings.jsx_typecheck {
        vize_atelier_jsx::register_atlas_providers(&mut compilation).map_err(graph_error)?;
    }
    if settings.preserve_unused_diagnostics {
        compilation
            .register_provider(ArtTemplateUsageProvider::new(
                settings.template_syntax,
                settings.experimental_in_tag_comments,
            ))
            .map_err(graph_error)?;
    }
    compilation
        .register_provider(CanonTypedDocumentProvider::new(settings))
        .map_err(graph_error)?;
    Ok((compilation, identities))
}

fn configure_compilation(
    compilation: &mut Compilation,
    settings: &CanonGraphSettings,
    sources: &[SourceId],
) -> CorsaResult<()> {
    compilation
        .set_input::<VueDialectInput>(settings.dialect)
        .map_err(graph_error)?;
    let mut compile_settings = SfcCompileSettings::default();
    let mut croquis_settings = SfcCroquisSettings::new(settings.croquis_mode());
    for source in sources.iter().copied() {
        let Some(path) = settings.path(source) else {
            continue;
        };
        if !is_vue(path) {
            continue;
        }
        compile_settings.insert(source, compile_request(settings, path));
        croquis_settings.insert_resolved_filename_with_policy(
            source,
            path.to_string_lossy().as_ref(),
            SfcResolvedPropsPolicy::PreserveCanonAfterTemplate,
        );
    }
    compile_settings.install(compilation).map_err(graph_error)?;
    croquis_settings.install(compilation).map_err(graph_error)?;
    Ok(())
}

fn compile_request(settings: &CanonGraphSettings, path: &Path) -> SfcCompileRequest {
    let mut options = SfcCompileOptions::default();
    options.parse.filename = path.to_string_lossy().to_compact_string();
    options.template.dialect = settings.dialect;
    options.template.compiler_options = Some(vize_atelier_dom::DomCompilerOptions {
        comments: true,
        experimental_in_tag_comments: settings.experimental_in_tag_comments,
        dialect: settings.dialect,
        ..Default::default()
    });
    SfcCompileRequest::new(options, settings.template_syntax)
}

struct CanonTypedDocumentProvider {
    settings: Shared<CanonGraphSettings>,
    rewriter: ImportRewriter,
}

impl CanonTypedDocumentProvider {
    fn new(settings: CanonGraphSettings) -> Self {
        Self {
            settings: Shared::new(settings),
            rewriter: ImportRewriter::new(),
        }
    }
}

impl Provider for CanonTypedDocumentProvider {
    type Product = CanonTypedDocumentProduct;

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        self.settings.path(context.source().id()).is_some()
    }

    fn dependencies(&self, context: &PlanningContext<'_>) -> Vec<ProductId> {
        let Some(path) = self.settings.path(context.source().id()) else {
            return Vec::new();
        };
        if is_vue(path) {
            let mut dependencies = vec![
                ProductId::of::<SfcDescriptorProduct>(),
                ProductId::of::<ReliefProduct>(),
                ProductId::of::<CroquisDocumentProduct>(),
                ProductId::of::<FlowProduct>(),
            ];
            if self.settings.preserve_unused_diagnostics {
                dependencies.push(ProductId::of::<ArtTemplateUsageProduct>());
            }
            return dependencies;
        }
        if self.settings.jsx_typecheck && is_jsx(path) {
            return vec![ProductId::of::<vize_atelier_jsx::JsxSyntaxProduct>()];
        }
        Vec::new()
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<CanonTypedDocumentArtifact, ProviderError> {
        let path = self
            .settings
            .path(context.source().id())
            .ok_or_else(|| ProviderError::message("Canon source path is missing"))?;
        let build_context = self.settings.context(&self.rewriter);
        if is_vue(path) {
            let artifact = context.get::<SfcDescriptorProduct>()?;
            let descriptor = match artifact.as_result() {
                Ok(descriptor) => descriptor,
                Err(error) => {
                    return Ok(CanonTypedDocumentArtifact::SfcParse(error.message.clone()));
                }
            };
            let syntax = context.get::<ReliefProduct>()?;
            let semantics = context.get::<CroquisDocumentProduct>()?;
            let _flow = context.get::<FlowProduct>()?;
            let art_usage = if self.settings.preserve_unused_diagnostics {
                Some(context.get::<ArtTemplateUsageProduct>()?)
            } else {
                None
            };
            return Ok(CanonTypedDocumentArtifact::from_result(
                build_vue_registered_file_from_artifacts(
                    path,
                    context.source().text(),
                    descriptor,
                    syntax.as_ref().as_ref(),
                    semantics.as_ref(),
                    art_usage.as_deref(),
                    build_context,
                ),
            ));
        }
        if self.settings.jsx_typecheck && is_jsx(path) {
            let syntax = context.get::<vize_atelier_jsx::JsxSyntaxProduct>()?;
            return Ok(CanonTypedDocumentArtifact::from_result(
                build_jsx_registered_file(path, syntax.as_ref(), build_context),
            ));
        }
        let source_type = if let Some(source_type) = self
            .settings
            .source_types
            .get(&context.source().id())
            .copied()
        {
            source_type
        } else if is_declaration_file(path) {
            SourceType::ts()
        } else {
            match source_type_for_path(path) {
                Some(source_type) => source_type,
                None => {
                    return Ok(CanonTypedDocumentArtifact::Failed(cstr!(
                        "unsupported Canon source path: {}",
                        path.display()
                    )));
                }
            }
        };
        Ok(CanonTypedDocumentArtifact::from_result(
            build_script_registered_file(
                path,
                context.source().text(),
                source_type,
                (&self.settings.project_root, &self.settings.virtual_root),
                &self.rewriter,
            ),
        ))
    }
}

fn graph_error(error: impl std::fmt::Display) -> CorsaError {
    CorsaError::ArtifactGraph(cstr!("{error}"))
}

#[cfg(test)]
#[path = "artifact_recipe/tests.rs"]
mod tests;
