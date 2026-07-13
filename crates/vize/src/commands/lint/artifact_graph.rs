//! Atlas-backed production lint queries for Vue documents.

use std::{path::Path, sync::Mutex};

use vize_atlas::{Compilation, CompilationSnapshot, Shared, SourceId};
use vize_carton::config::VueVersion;
#[cfg(test)]
use vize_croquis::{CroquisDocument, CroquisDocumentProduct};
use vize_patina::{LintResult, Linter, PatinaDocumentReportProduct};
use vize_relief::VueDialectInput;

pub(super) struct ArtifactLintOutcome {
    pub(super) result: LintResult,
    #[cfg(test)]
    pub(super) semantics: Option<Shared<CroquisDocument>>,
}

pub(super) struct LintArtifactGraph {
    compilation: Mutex<Compilation>,
    snapshot: CompilationSnapshot,
    sources: Vec<Option<SourceId>>,
}

impl LintArtifactGraph {
    pub(super) fn new<'a>(
        linter: Shared<Linter>,
        dialect: VueVersion,
        inputs: impl IntoIterator<Item = (&'a Path, &'a str)>,
    ) -> Result<Self, vize_carton::String> {
        let mut compilation = configured_compilation(linter, dialect)?;
        let sources = inputs
            .into_iter()
            .map(|(path, source)| {
                is_artifact_path(path)
                    .then(|| compilation.add_source(path.to_string_lossy().as_ref(), source))
                    .transpose()
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| vize_carton::cstr!("failed to add lint source: {error}"))?;
        let snapshot = compilation.snapshot();
        Ok(Self {
            compilation: Mutex::new(compilation),
            snapshot,
            sources,
        })
    }

    pub(super) fn query(&self, index: usize) -> Result<ArtifactLintOutcome, vize_carton::String> {
        let source =
            self.sources.get(index).copied().flatten().ok_or_else(|| {
                vize_carton::cstr!("source {index} is not an Atlas lint document")
            })?;
        query_snapshot(&self.snapshot, source)
    }

    pub(super) fn query_revised(
        &self,
        index: usize,
        text: &str,
    ) -> Result<ArtifactLintOutcome, vize_carton::String> {
        let source =
            self.sources.get(index).copied().flatten().ok_or_else(|| {
                vize_carton::cstr!("source {index} is not an Atlas lint document")
            })?;
        let snapshot = {
            let mut compilation = self
                .compilation
                .lock()
                .map_err(|_| vize_carton::cstr!("lint compilation lock was poisoned"))?;
            compilation
                .update_source(source, text)
                .map_err(|error| vize_carton::cstr!("failed to update lint source: {error}"))?;
            compilation.snapshot()
        };
        query_snapshot(&snapshot, source)
    }
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
    vize_atelier_jsx::register_atlas_providers(&mut compilation)
        .map_err(|error| vize_carton::cstr!("failed to register JSX providers: {error}"))?;
    vize_patina::register_shared_document_lint_recipe(&mut compilation, linter)
        .map_err(|error| vize_carton::cstr!("failed to register Patina provider: {error}"))?;
    Ok(compilation)
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
    })
}

pub(super) fn is_vue_path(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("vue")
}

pub(super) fn is_artifact_path(path: &Path) -> bool {
    if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.ends_with(".stories.jsx")
                || name.ends_with(".stories.tsx")
                || name.ends_with(".story.jsx")
                || name.ends_with(".story.tsx")
        })
    {
        return false;
    }
    is_vue_path(path)
        || matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("jsx" | "tsx")
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use vize_atelier_jsx::JsxSyntaxProduct;
    use vize_relief::ReliefProduct;
    use vize_relief::TransformedReliefProduct;

    #[test]
    fn configured_graph_preserves_the_project_vue_dialect() {
        let compilation =
            configured_compilation(Shared::new(Linter::new()), VueVersion::V2).unwrap();

        assert_eq!(
            compilation.input::<VueDialectInput>(),
            Some(&VueVersion::V2)
        );
    }

    #[test]
    fn production_graph_requests_parse_and_complete_semantic_products() {
        let mut compilation =
            configured_compilation(Shared::new(Linter::new()), VueVersion::V3).unwrap();
        let source = compilation
            .add_source(
                "Component.vue",
                "<script setup>const value = 1</script><template>{{ value }}</template>",
            )
            .unwrap();

        let plan = compilation
            .plan_for::<PatinaDocumentReportProduct>(source)
            .unwrap();

        assert!(plan.contains::<vize_relief::ReliefProduct>());
        assert!(plan.contains::<CroquisDocumentProduct>());
        assert!(!plan.contains::<TransformedReliefProduct>());
        let outcome = query_snapshot(&compilation.snapshot(), source).unwrap();
        assert_eq!(outcome.semantics.unwrap().sources().len(), 2);
    }

    #[test]
    fn malformed_sfc_is_cached_once_and_still_produces_patina_diagnostics() {
        let mut compilation =
            configured_compilation(Shared::new(Linter::new()), VueVersion::V3).unwrap();
        let source = compilation
            .add_source(
                "Malformed.vue",
                "<template><div /></template><template><span /></template>",
            )
            .unwrap();
        let snapshot = compilation.snapshot();
        let mut session = snapshot.query_session();

        let lint = session
            .query::<PatinaDocumentReportProduct>(source)
            .unwrap();
        assert!(lint.value().error_count > 0);
        assert!(
            lint.value()
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_name == "parser/sfc")
        );
        assert_eq!(
            session
                .counters()
                .for_product::<vize_atelier_sfc::SfcDescriptorProduct>()
                .executions(),
            1
        );
        session
            .query::<vize_atelier_sfc::SfcDescriptorProduct>(source)
            .unwrap();
        let counters = session
            .counters()
            .for_product::<vize_atelier_sfc::SfcDescriptorProduct>();
        assert_eq!(counters.executions(), 1);
        assert_eq!(counters.cache_hits(), 1);
    }

    #[test]
    fn jsx_graph_uses_owned_syntax_and_never_plans_relief() {
        let mut compilation =
            configured_compilation(Shared::new(Linter::new()), VueVersion::V3).unwrap();
        let source = compilation
            .add_source("View.tsx", "const View = (): JSX.Element => <img />;")
            .unwrap();
        let plan = compilation
            .plan_for::<PatinaDocumentReportProduct>(source)
            .unwrap();

        assert!(plan.contains::<JsxSyntaxProduct>());
        assert!(plan.contains::<CroquisDocumentProduct>());
        assert!(!plan.contains::<ReliefProduct>());
        let outcome = query_snapshot(&compilation.snapshot(), source).unwrap();
        assert!(outcome.semantics.is_some());
    }
}
