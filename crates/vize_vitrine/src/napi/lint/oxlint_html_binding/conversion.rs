//! Final complete DTO conversion; no source or authority is read here.
use super::super::file_collection::oxlint_html_profile as profile;
use super::super::oxlint_html::Operation;
use super::dto::{
    OxlintHtmlCompleted, OxlintHtmlDiagnostic, OxlintHtmlFile, OxlintHtmlLabel, OxlintHtmlOptions,
    OxlintHtmlOriginal, OxlintHtmlProjection, OxlintHtmlRule, OxlintHtmlSettings, OxlintHtmlSource,
};
use super::report::Rendered;
use std::path::Path;
use vize_patina::Severity;

fn path_text(path: &Path) -> Result<String, String> {
    path.to_str()
        .map(Into::into)
        .ok_or_else(|| "non-UTF8 path in admitted HTML result".into())
}

fn severity(value: Severity) -> &'static str {
    match value {
        Severity::Error => "error",
        Severity::Warning => "warning",
    }
}

pub(super) fn completed(
    operation: Operation,
    options: OxlintHtmlOptions,
    rendered: Rendered,
) -> Result<OxlintHtmlCompleted, String> {
    let selection = operation.selection;
    let originals = selection
        .originals
        .into_iter()
        .map(|original| {
            Ok(OxlintHtmlOriginal {
                path: path_text(&original.path)?,
                cwd_relative: path_text(&original.cwd_relative)?,
                origin: match original.origin {
                    profile::Origin::DirectoryDiscovery => "DirectoryDiscovery",
                    profile::Origin::ExplicitFile => "ExplicitFile",
                }
                .into(),
                bytes: original.bytes,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let sources = selection
        .sources
        .into_iter()
        .map(|source| {
            Ok(OxlintHtmlSource {
                path: path_text(&source.path)?,
                role: match source.role {
                    profile::SourceRole::RootJson => "RootJson",
                    profile::SourceRole::GitIgnore => "GitIgnore",
                    profile::SourceRole::GitInfoExclude => "GitInfoExclude",
                    profile::SourceRole::CustomIgnore => "CustomIgnore",
                }
                .into(),
                bytes: source.bytes,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let files = operation
        .files
        .into_iter()
        .map(|file| {
            let diagnostics = file
                .result
                .diagnostics
                .into_iter()
                .map(|diagnostic| {
                    Ok(OxlintHtmlDiagnostic {
                        rule_name: diagnostic.rule_name.into(),
                        severity: severity(diagnostic.severity).into(),
                        message: diagnostic.message.as_str().into(),
                        start: diagnostic.start,
                        end: diagnostic.end,
                        help: diagnostic.help.map(|value| value.as_str().into()),
                        labels: diagnostic
                            .labels
                            .into_iter()
                            .map(|label| OxlintHtmlLabel {
                                message: label.message.as_str().into(),
                                start: label.start,
                                end: label.end,
                            })
                            .collect(),
                        fix: diagnostic
                            .fix
                            .map(serde_json::to_value)
                            .transpose()
                            .map_err(|_| "HTML fix conversion failed")?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(OxlintHtmlFile {
                path: path_text(&file.path)?,
                filename: file.result.filename.as_str().into(),
                error_count: file.result.error_count as f64,
                warning_count: file.result.warning_count as f64,
                diagnostics,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let projection = operation.projection;
    Ok(OxlintHtmlCompleted {
        host_profile: options.host_profile,
        cwd: path_text(&selection.cwd)?,
        literal_target: selection.literal_target.as_str().into(),
        target: path_text(&selection.target)?,
        repository: path_text(&selection.repository)?,
        root_json: path_text(&selection.root_json)?,
        no_ignore: selection.no_ignore,
        cli_ignore_patterns: selection
            .cli_ignore_patterns
            .iter()
            .map(|value| value.as_str().into())
            .collect(),
        custom_ignore_filename: selection.custom_ignore_filename.as_str().into(),
        root_decision: match selection.root_decision {
            profile::RootDecision::Eligible => "Eligible",
            profile::RootDecision::CliOrCustomExcluded => "CliOrCustomExcluded",
            profile::RootDecision::VcsExcluded => "VcsExcluded",
        }
        .into(),
        originals,
        sources,
        files,
        projection: OxlintHtmlProjection {
            rules: projection
                .rules
                .into_iter()
                .map(|rule| OxlintHtmlRule {
                    name: rule.name.as_str().into(),
                    severity: rule.severity.map(|value| severity(value).into()),
                    active: rule.active,
                    authored_options: rule.authored_options,
                })
                .collect(),
            settings: OxlintHtmlSettings {
                locale: projection.settings.locale.as_str().into(),
                help_level: projection.settings.help_level.as_str().into(),
                preset: projection.settings.preset.as_str().into(),
            },
            deny_warnings: projection.deny_warnings,
        },
        executed_file_count: operation.executed_file_count as f64,
        elapsed_seconds: operation.elapsed.as_secs_f64(),
        errors: rendered.errors as f64,
        warnings: rendered.warnings as f64,
        output: rendered.output,
        json_diagnostics: rendered.json_diagnostics,
        format: options.format,
        presentation: options.presentation,
        engine_config_validation: "not-performed".into(),
    })
}
