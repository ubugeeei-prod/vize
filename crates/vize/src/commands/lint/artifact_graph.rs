//! One persistent Atlas graph for the complete lint command run.

use std::{path::Path, sync::RwLock};

use vize_atelier_template::{TemplateCompileRequest, TemplateCompileSettingsInput};
use vize_atlas::{Compilation, CompilationSnapshot, Shared, SourceId};
use vize_carton::config::VueVersion;
#[cfg(test)]
use vize_croquis::{CroquisDocument, CroquisDocumentProduct};
use vize_croquis_cf::{
    CrossFileAnalysisArtifact, CrossFileAnalysisInput, CrossFileAnalysisProduct,
    CrossFileAnalysisRequest, CrossFileOptions,
};
use vize_patina::{
    LintResult, Linter, PatinaDocumentMode, PatinaDocumentReportProduct, PatinaTemplateLintRequest,
    install_document_mode, install_template_lint_request,
};
use vize_relief::VueDialectInput;

use super::collect::is_standalone_html_path;

pub(super) struct ArtifactLintOutcome {
    pub(super) result: LintResult,
    #[cfg(test)]
    pub(super) semantics: Option<Shared<CroquisDocument>>,
    #[cfg(test)]
    pub(super) trace: vize_atlas::ExecutionTrace,
    #[cfg(test)]
    pub(super) counters: vize_atlas::ExecutionCounters,
}

pub(super) struct ArtifactCrossFileOutcome {
    pub(super) artifact: Shared<CrossFileAnalysisArtifact>,
    #[cfg(test)]
    pub(super) trace: vize_atlas::ExecutionTrace,
    #[cfg(test)]
    pub(super) counters: vize_atlas::ExecutionCounters,
}

pub(super) struct LintArtifactGraph {
    snapshot: RwLock<CompilationSnapshot>,
    sources: Vec<SourceId>,
}

impl LintArtifactGraph {
    pub(super) fn new<'a>(
        linter: Shared<Linter>,
        dialect: VueVersion,
        inputs: impl IntoIterator<Item = (&'a Path, &'a str)>,
    ) -> Result<Self, vize_carton::String> {
        let mut compilation = configured_compilation(linter, dialect)?;
        let mut sources = Vec::new();
        for (path, text) in inputs {
            let filename = path.to_string_lossy();
            let atlas_name = if is_standalone_html_path(path) {
                vize_carton::cstr!("{filename}.vue-template")
            } else {
                filename.as_ref().into()
            };
            let source = compilation
                .add_source(atlas_name.as_str(), text)
                .map_err(|error| vize_carton::cstr!("failed to add lint source: {error}"))?;
            if is_standalone_html_path(path) {
                compilation
                    .set_source_input::<TemplateCompileSettingsInput>(
                        source,
                        TemplateCompileRequest::default(),
                    )
                    .map_err(|error| {
                        vize_carton::cstr!("failed to configure HTML frontend: {error}")
                    })?;
                install_template_lint_request(
                    &mut compilation,
                    source,
                    PatinaTemplateLintRequest::standalone_html(filename.as_ref()),
                )
                .map_err(|error| vize_carton::cstr!("failed to configure HTML lint: {error}"))?;
            }
            if is_storybook_csf_path(path) {
                install_document_mode(&mut compilation, source, PatinaDocumentMode::Disabled)
                    .map_err(|error| {
                        vize_carton::cstr!("failed to configure excluded lint source: {error}")
                    })?;
            }
            sources.push(source);
        }
        let snapshot = compilation.snapshot();
        Ok(Self {
            snapshot: RwLock::new(snapshot),
            sources,
        })
    }

    pub(super) fn query(&self, index: usize) -> Result<ArtifactLintOutcome, vize_carton::String> {
        let source = self.source(index)?;
        query_snapshot(&self.current_snapshot()?, source)
    }

    pub(super) fn revise_sources(
        &self,
        revisions: &[(usize, &str)],
    ) -> Result<(), vize_carton::String> {
        if revisions.is_empty() {
            return Ok(());
        }
        let mut compilation = self.current_snapshot()?.fork();
        for (index, text) in revisions {
            let source = self.source(*index)?;
            compilation
                .update_source(source, *text)
                .map_err(|error| vize_carton::cstr!("failed to update lint source: {error}"))?;
        }
        let mut snapshot = self
            .snapshot
            .write()
            .map_err(|_| vize_carton::cstr!("lint snapshot lock was poisoned"))?;
        *snapshot = compilation.snapshot();
        Ok(())
    }

    pub(super) fn query_cross_file(
        &self,
        anchor_index: usize,
    ) -> Result<ArtifactCrossFileOutcome, vize_carton::String> {
        let source = self.source(anchor_index)?;
        let snapshot = self.current_snapshot()?;
        let mut session = snapshot.query_session();
        let outcome = session
            .query::<CrossFileAnalysisProduct>(source)
            .map_err(|error| vize_carton::cstr!("Atlas cross-file query failed: {error}"))?;
        Ok(ArtifactCrossFileOutcome {
            artifact: outcome.shared(),
            #[cfg(test)]
            trace: outcome.trace().clone(),
            #[cfg(test)]
            counters: session.counters().clone(),
        })
    }

    pub(super) fn source(&self, index: usize) -> Result<SourceId, vize_carton::String> {
        self.sources
            .get(index)
            .copied()
            .ok_or_else(|| vize_carton::cstr!("lint source index {index} is not registered"))
    }

    fn current_snapshot(&self) -> Result<CompilationSnapshot, vize_carton::String> {
        self.snapshot
            .read()
            .map(|snapshot| snapshot.clone())
            .map_err(|_| vize_carton::cstr!("lint snapshot lock was poisoned"))
    }
}

fn is_storybook_csf_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.ends_with(".stories.jsx")
                || name.ends_with(".stories.tsx")
                || name.ends_with(".story.jsx")
                || name.ends_with(".story.tsx")
        })
}

fn configured_compilation(
    linter: Shared<Linter>,
    dialect: VueVersion,
) -> Result<Compilation, vize_carton::String> {
    let mut compilation = Compilation::new();
    compilation
        .set_input::<VueDialectInput>(dialect)
        .map_err(|error| vize_carton::cstr!("failed to configure Vue dialect: {error}"))?;
    vize_atelier_sfc::register_atlas_providers(&mut compilation)
        .map_err(|error| vize_carton::cstr!("failed to register SFC providers: {error}"))?;
    vize_module::register_raw_providers(&mut compilation)
        .map_err(|error| vize_carton::cstr!("failed to register Module providers: {error}"))?;
    vize_atelier_jsx::register_atlas_providers(&mut compilation)
        .map_err(|error| vize_carton::cstr!("failed to register JSX providers: {error}"))?;
    compilation
        .register_provider(vize_atelier_template::RawTemplateReliefProvider)
        .map_err(|error| vize_carton::cstr!("failed to register raw-template provider: {error}"))?;
    compilation
        .register_provider(vize_atelier_template::RawTemplateCroquisProvider)
        .map_err(|error| {
            vize_carton::cstr!("failed to register raw-template semantics provider: {error}")
        })?;
    vize_patina::register_shared_document_lint_recipe(&mut compilation, Shared::clone(&linter))
        .map_err(|error| vize_carton::cstr!("failed to register Patina provider: {error}"))?;
    vize_patina::register_shared_module_lint_recipe(&mut compilation, Shared::clone(&linter))
        .map_err(|error| {
            vize_carton::cstr!("failed to register Patina Module provider: {error}")
        })?;
    vize_patina::register_shared_template_lint_recipe(&mut compilation, linter).map_err(
        |error| vize_carton::cstr!("failed to register Patina template provider: {error}"),
    )?;
    vize_croquis_cf::register_atlas_provider(&mut compilation)
        .map_err(|error| vize_carton::cstr!("failed to register cross-file providers: {error}"))?;
    compilation
        .set_input::<CrossFileAnalysisInput>(
            CrossFileAnalysisRequest::new(patina_cross_file_options())
                .with_project_root(std::env::current_dir().unwrap_or_default()),
        )
        .map_err(|error| vize_carton::cstr!("failed to configure cross-file analysis: {error}"))?;
    Ok(compilation)
}

fn patina_cross_file_options() -> CrossFileOptions {
    CrossFileOptions::minimal()
        .with_provide_inject(true)
        .with_unique_ids(true)
        .with_server_client_boundary(true)
        .with_reactivity_tracking(true)
        .with_race_conditions(true)
}

fn query_snapshot(
    snapshot: &CompilationSnapshot,
    source: SourceId,
) -> Result<ArtifactLintOutcome, vize_carton::String> {
    let mut session = snapshot.query_session();
    #[cfg(test)]
    let is_vue = session
        .source(source)
        .is_some_and(|source| source.name().ends_with(".vue"));
    let outcome = session
        .query::<PatinaDocumentReportProduct>(source)
        .map_err(|error| vize_carton::cstr!("Atlas lint query failed: {error}"))?;
    #[cfg(test)]
    let has_valid_document = if is_vue {
        outcome
            .execution()
            .get::<vize_atelier_sfc::SfcDescriptorProduct>()
            .map_err(|error| vize_carton::cstr!("Atlas descriptor result failed: {error}"))?
            .is_some_and(|descriptor| descriptor.descriptor().is_some())
    } else {
        true
    };
    #[cfg(test)]
    let semantics = if has_valid_document {
        outcome
            .execution()
            .get::<CroquisDocumentProduct>()
            .map_err(|error| vize_carton::cstr!("Atlas Croquis result failed: {error}"))?
    } else {
        None
    };
    Ok(ArtifactLintOutcome {
        result: outcome.value().clone(),
        #[cfg(test)]
        semantics,
        #[cfg(test)]
        trace: outcome.trace().clone(),
        #[cfg(test)]
        counters: session.counters().clone(),
    })
}

#[cfg(test)]
#[path = "artifact_graph/tests.rs"]
mod tests;
