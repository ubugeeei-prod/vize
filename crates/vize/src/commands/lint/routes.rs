//! Project-level Vue Router route typing (P4-10a) for `vize lint --cross-file`.
//!
//! Every linted script module and SFC becomes one [`ProjectSources`] module;
//! the Vue Router provider's route facts are computed once and the
//! route-typing consumer's findings are merged into the file they point at.

use vize_croquis_cf::providers::ProjectSources;
use vize_croquis_cf::providers::vue_router::typing::{self, RouteDiagnostic};
use vize_l0::CompactString;
use vize_l0::diag::{PartKind, Severity};
use vize_patina::{HelpLevel, LintDiagnostic, LintResult};

pub(super) mod config;

use super::cross_file::{CliLintFileResult, apply_sfc_cross_file_lint, merge_lint_result};
use super::entry_rules::ResolvedLinterRuleGroups;
use config::{CrossFileRuleSettings, configure};

/// The whole `--cross-file` lane: the SFC cross-file analyzer, then route
/// typing over every linted file. Returns the analyzer's report.
pub(super) fn apply_cross_file_lint(
    results: &mut [CliLintFileResult],
    help_level: HelpLevel,
    (args, files, resolved): (
        &super::LintArgs,
        &[std::path::PathBuf],
        &ResolvedLinterRuleGroups,
    ),
) -> Option<vize_l0::String> {
    let settings = CrossFileRuleSettings::new(files, resolved).unwrap_or_else(|error| {
        eprintln!("[vize] {error}");
        std::process::exit(2);
    });
    let (tree, complexity) = (args.cross_file_tree, args.cross_file_complexity);
    let report = apply_sfc_cross_file_lint(results, help_level, tree, complexity, &settings);
    apply_route_typing(results, help_level, Some(&settings));
    report
}

/// Run route typing over every linted file and merge its findings.
fn apply_route_typing(
    results: &mut [CliLintFileResult],
    help_level: HelpLevel,
    settings: Option<&CrossFileRuleSettings<'_>>,
) {
    let mut project = ProjectSources::new();
    let mut indexes = Vec::new();
    for (index, (path, _, source, _)) in results.iter().enumerate() {
        if project.add(&path.to_string_lossy(), source).is_some() {
            indexes.push(index);
        }
    }
    let Ok(findings) = typing::run(&project) else {
        return;
    };
    for finding in findings {
        let Some((path, filename, source, result)) = indexes
            .get(finding.module.index())
            .and_then(|&index| results.get_mut(index))
        else {
            continue;
        };
        let Some(diagnostic) = configure(
            settings,
            path,
            finding.code,
            to_lint(&finding, source.len(), help_level),
        ) else {
            continue;
        };
        let extra = LintResult {
            filename: filename.clone(),
            error_count: usize::from(diagnostic.severity == vize_patina::Severity::Error),
            warning_count: usize::from(diagnostic.severity != vize_patina::Severity::Error),
            diagnostics: vec![diagnostic],
        };
        merge_lint_result(result, extra);
    }
}

fn to_lint(finding: &RouteDiagnostic, source_len: usize, help_level: HelpLevel) -> LintDiagnostic {
    let diagnostic = &finding.diagnostic;
    let limit = source_len as u32;
    let start = diagnostic.span.start.min(limit);
    let end = diagnostic.span.end.clamp(start, limit);
    let message = diagnostic.message.as_str();
    let mut lint = match diagnostic.severity() {
        Severity::Error => LintDiagnostic::error(finding.code, message, start, end),
        Severity::Warning | Severity::Info | Severity::Hint => {
            LintDiagnostic::warn(finding.code, message, start, end)
        }
    };
    let mut help = CompactString::default();
    for part in &diagnostic.parts {
        match part.kind {
            PartKind::Help => {
                if !help.is_empty() {
                    help.push('\n');
                }
                help.push_str(&part.message);
            }
            PartKind::Primary | PartKind::Secondary => {
                let start = part.span.start.min(limit);
                lint = lint.with_label(
                    part.message.as_str(),
                    start,
                    part.span.end.clamp(start, limit),
                );
            }
            PartKind::Suggestion => {}
        }
    }
    if let Some(help) = help_level.process(&help) {
        lint = lint.with_help(help.as_str());
    }
    lint
}

#[cfg(test)]
mod tests {
    use super::apply_route_typing;
    use std::path::Path;
    use vize_patina::{HelpLevel, LintResult};

    #[test]
    fn the_demo_app_findings_land_in_the_files_they_point_at() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/_fixtures/davinci-routes/demo-app");
        let files = [
            "src/router/index.ts",
            "src/router/admin.ts",
            "src/components/UserCard.vue",
            "src/composables/useAdminNav.ts",
            "src/views/HomeView.vue",
            "src/views/UserPost.vue",
            // Authored actual app installation binds the original navigation
            // carriers; every old route fact, finding and byte span stays exact.
            "src/App.vue",
            "src/main.ts",
        ];
        let mut results: Vec<_> = files
            .iter()
            .map(|file| {
                let source = std::fs::read_to_string(root.join(file)).unwrap();
                let result = LintResult {
                    filename: (*file).into(),
                    diagnostics: Vec::new(),
                    error_count: 0,
                    warning_count: 0,
                };
                (
                    Path::new(file).to_path_buf(),
                    (*file).into(),
                    source.into(),
                    result,
                )
            })
            .collect();
        apply_route_typing(&mut results, HelpLevel::None, None);
        let rows: Vec<_> = results
            .iter()
            .flat_map(|(_, filename, _, result)| {
                result.diagnostics.iter().map(move |diagnostic| {
                    let span = (diagnostic.start, diagnostic.end);
                    (filename.as_str(), diagnostic.rule_name, span)
                })
            })
            .collect();
        assert_eq!(
            rows,
            [
                // `"user-posts"`, `{ id: user.id }`, `id`, `['vue', 'fes']`
                (
                    "src/components/UserCard.vue",
                    "ecosystem/vue-router-unknown-route",
                    (247, 259)
                ),
                (
                    "src/components/UserCard.vue",
                    "ecosystem/vue-router-missing-param",
                    (391, 406)
                ),
                (
                    "src/components/UserCard.vue",
                    "ecosystem/vue-router-extra-param",
                    (393, 395)
                ),
                (
                    "src/components/UserCard.vue",
                    "ecosystem/vue-router-param-type",
                    (627, 641)
                ),
                // `tab`
                (
                    "src/composables/useAdminNav.ts",
                    "ecosystem/vue-router-extra-param",
                    (150, 153)
                ),
                // `"user"`
                (
                    "src/views/UserPost.vue",
                    "ecosystem/vue-router-missing-param",
                    (147, 153)
                ),
            ]
        );
        let counts: Vec<_> = results
            .iter()
            .map(|(_, _, _, result)| (result.error_count, result.warning_count))
            .collect();
        assert_eq!(
            counts,
            [
                (0, 0),
                (0, 0),
                (3, 1),
                (1, 0),
                (0, 0),
                (0, 1),
                (0, 0),
                (0, 0)
            ]
        );
    }
}
