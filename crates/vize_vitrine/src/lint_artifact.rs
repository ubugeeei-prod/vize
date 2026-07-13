//! Persistent Atlas graph shared by the NAPI and WASM lint hosts.

use std::sync::RwLock;

use vize_atlas::{Compilation, CompilationSnapshot, Shared, SourceId};
use vize_carton::{String, config::VueVersion, cstr};
use vize_patina::{LintResult, Linter, PatinaDocumentReportProduct, PatinaTemplateLintRequest};
use vize_relief::VueDialectInput;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(not(feature = "napi"), allow(dead_code))]
pub(crate) enum LintSourceKind {
    Sfc,
    #[cfg_attr(not(feature = "wasm"), allow(dead_code))]
    VueTemplate,
    StandaloneHtml,
}

pub(crate) struct LintGraphSource<'a> {
    pub(crate) name: &'a str,
    pub(crate) text: &'a str,
    pub(crate) kind: LintSourceKind,
}

pub(crate) struct LintArtifactOutcome {
    pub(crate) result: LintResult,
    #[cfg(test)]
    plan: vize_atlas::Plan,
    #[cfg(test)]
    counters: vize_atlas::ExecutionCounters,
    #[cfg(test)]
    status: vize_atlas::ProductStatus,
}

/// One source store, provider registry, and cache for a complete host request.
pub(crate) struct PatinaLintGraph {
    snapshot: RwLock<CompilationSnapshot>,
    sources: Vec<SourceId>,
}

impl PatinaLintGraph {
    pub(crate) fn new<'a>(
        linter: Shared<Linter>,
        inputs: impl IntoIterator<Item = LintGraphSource<'a>>,
    ) -> Result<Self, String> {
        let mut compilation = configured_compilation(Shared::clone(&linter))?;
        let mut sources = Vec::new();
        for (index, input) in inputs.into_iter().enumerate() {
            let source_name = match input.kind {
                LintSourceKind::Sfc => input.name.into(),
                LintSourceKind::VueTemplate | LintSourceKind::StandaloneHtml => {
                    cstr!("ffi-lint-{index}.vue-template")
                }
            };
            let source = compilation
                .add_source(source_name, input.text)
                .map_err(|error| cstr!("failed to add lint source: {error}"))?;
            if input.kind != LintSourceKind::Sfc {
                compilation
                    .set_source_input::<vize_atelier_template::TemplateCompileSettingsInput>(
                        source,
                        vize_atelier_template::TemplateCompileRequest::default(),
                    )
                    .map_err(|error| cstr!("failed to configure template lint: {error}"))?;
                let request = match input.kind {
                    LintSourceKind::VueTemplate => {
                        PatinaTemplateLintRequest::vue_template(input.name)
                    }
                    LintSourceKind::StandaloneHtml => {
                        PatinaTemplateLintRequest::standalone_html(input.name)
                    }
                    LintSourceKind::Sfc => unreachable!(),
                };
                vize_patina::install_template_lint_request(&mut compilation, source, request)
                    .map_err(|error| cstr!("failed to configure Patina template lint: {error}"))?;
            }
            sources.push(source);
        }
        Ok(Self {
            snapshot: RwLock::new(compilation.snapshot()),
            sources,
        })
    }

    pub(crate) fn query(&self, index: usize) -> Result<LintArtifactOutcome, String> {
        let source = self.source(index)?;
        let snapshot = self
            .snapshot
            .read()
            .map_err(|_| cstr!("lint compilation lock was poisoned"))?
            .clone();
        let mut session = snapshot.query_session();
        let outcome = session
            .query::<PatinaDocumentReportProduct>(source)
            .map_err(|error| cstr!("Atlas lint query failed: {error}"))?;
        Ok(LintArtifactOutcome {
            result: outcome.value().clone(),
            #[cfg(test)]
            plan: outcome.plan().clone(),
            #[cfg(test)]
            counters: session.counters().clone(),
            #[cfg(test)]
            status: outcome.status(),
        })
    }

    #[cfg(any(feature = "napi", test))]
    pub(crate) fn revise_source(&self, index: usize, text: &str) -> Result<(), String> {
        let source = self.source(index)?;
        let mut snapshot = self
            .snapshot
            .write()
            .map_err(|_| cstr!("lint compilation lock was poisoned"))?;
        let mut compilation = snapshot.fork();
        compilation
            .update_source(source, text)
            .map_err(|error| cstr!("failed to update lint source: {error}"))?;
        *snapshot = compilation.snapshot();
        Ok(())
    }

    fn source(&self, index: usize) -> Result<SourceId, String> {
        self.sources
            .get(index)
            .copied()
            .ok_or_else(|| cstr!("lint source index {index} is not registered"))
    }
}

fn configured_compilation(linter: Shared<Linter>) -> Result<Compilation, String> {
    let mut compilation = Compilation::new();
    compilation
        .set_input::<VueDialectInput>(VueVersion::V3)
        .map_err(|error| cstr!("failed to configure Vue dialect: {error}"))?;
    vize_atelier_sfc::register_atlas_providers(&mut compilation)
        .map_err(|error| cstr!("failed to register SFC providers: {error}"))?;
    compilation
        .register_provider(vize_atelier_template::RawTemplateReliefProvider)
        .map_err(|error| cstr!("failed to register raw-template Relief provider: {error}"))?;
    compilation
        .register_provider(vize_atelier_template::RawTemplateCroquisProvider)
        .map_err(|error| cstr!("failed to register raw-template Croquis provider: {error}"))?;
    vize_patina::register_shared_document_lint_recipe(&mut compilation, Shared::clone(&linter))
        .map_err(|error| cstr!("failed to register Patina document provider: {error}"))?;
    vize_patina::register_shared_template_lint_recipe(&mut compilation, linter)
        .map_err(|error| cstr!("failed to register Patina template provider: {error}"))?;
    Ok(compilation)
}

#[cfg(test)]
mod tests {
    use vize_atlas::ProductStatus;
    use vize_relief::ReliefProduct;

    use super::*;

    #[test]
    fn sfc_template_and_html_are_all_atlas_report_roots() {
        let graph = PatinaLintGraph::new(
            Shared::new(Linter::new()),
            [
                LintGraphSource {
                    name: "App.vue",
                    text: "<template><button v-for=\"item in items\">{{ item }}</button></template>",
                    kind: LintSourceKind::Sfc,
                },
                LintGraphSource {
                    name: "button.html",
                    text: "<button v-for=\"item in items\">{{ item }}</button>",
                    kind: LintSourceKind::VueTemplate,
                },
                LintGraphSource {
                    name: "index.html",
                    text: "<main v-for=\"item in items\">{{ item }}</main>",
                    kind: LintSourceKind::StandaloneHtml,
                },
            ],
        )
        .unwrap();

        for index in 0..3 {
            let outcome = graph.query(index).unwrap();
            assert!(outcome.plan.contains::<PatinaDocumentReportProduct>());
            assert_eq!(
                outcome
                    .counters
                    .for_product::<PatinaDocumentReportProduct>()
                    .executions(),
                1
            );
            if index > 0 {
                assert!(outcome.plan.contains::<ReliefProduct>());
            }
        }
    }

    #[test]
    fn autofix_revalidation_reuses_source_identity_and_invalidates_report() {
        let graph = PatinaLintGraph::new(
            Shared::new(Linter::new()),
            [LintGraphSource {
                name: "App.vue",
                text: "<template><div v-for=\"item in items\">{{ item }}</div></template>",
                kind: LintSourceKind::Sfc,
            }],
        )
        .unwrap();
        let first = graph.query(0).unwrap();
        assert_eq!(first.status, ProductStatus::Executed);
        assert_eq!(
            first
                .counters
                .for_product::<PatinaDocumentReportProduct>()
                .executions(),
            1
        );
        graph
            .revise_source(
                0,
                "<template><div v-for=\"item in items\" :key=\"item\">{{ item }}</div></template>",
            )
            .unwrap();
        let second = graph.query(0).unwrap();
        assert_eq!(second.status, ProductStatus::Executed);
        assert_eq!(
            second
                .counters
                .for_product::<PatinaDocumentReportProduct>()
                .executions(),
            1
        );
        assert!(
            second
                .result
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.rule_name != "vue/require-v-for-key")
        );
        let cached = graph.query(0).unwrap();
        assert_eq!(cached.status, ProductStatus::CacheHit);
        assert_eq!(
            cached
                .counters
                .for_product::<PatinaDocumentReportProduct>()
                .cache_hits(),
            1
        );
    }
}
