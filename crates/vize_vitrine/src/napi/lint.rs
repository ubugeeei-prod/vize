//! NAPI bindings for Vue SFC linting.
//!
//! Provides the `lint` function for linting Vue SFC files
//! with native multithreading and .gitignore awareness.
//!
//! FFI boundary code: uses std types for JavaScript interop.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use napi::bindgen_prelude::{Error, Result, Status};
use napi_derive::napi;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use serde_json::{Value, json};
use std::sync::atomic::{AtomicUsize, Ordering};
use vize_s0::append;

use super::lint_fix::{lint_file_with_optional_fix, lint_source};
mod empty_result;
mod file_collection;
mod lint_options;
use lint_options::{
    LintOptionsNapi, LintResultNapi, PatinaLintOptionsNapi, configure_type_aware_lint,
    create_patina_linter, patina_help_level_from_option, patina_locale_from_option,
    patina_preset_from_option,
};

struct PatinaRuleMetaNapi<'a> {
    name: &'a str,
    description: &'a str,
    category: &'a str,
    fixable: bool,
    default_severity: &'a str,
    presets: Vec<&'static str>,
}

#[inline]
const fn plugin_preset_name(preset: vize_patina::LintPreset) -> &'static str {
    match preset {
        vize_patina::LintPreset::HappyPath => "general-recommended",
        vize_patina::LintPreset::Opinionated => "opinionated",
        vize_patina::LintPreset::Essential => "essential",
        vize_patina::LintPreset::Incremental => "incremental",
        vize_patina::LintPreset::Ecosystem => "ecosystem",
        vize_patina::LintPreset::Nuxt => "nuxt",
    }
}

#[inline]
fn plugin_preset_name_from_raw(preset: &'static str) -> &'static str {
    match preset {
        "general-recommended" | "GeneralRecommended" | "generalRecommended" => {
            "general-recommended"
        }
        "happy-path" | "happy_path" | "happy" | "default" | "recommended" => "general-recommended",
        "essential" | "Essential" => "essential",
        "incremental" | "Incremental" => "incremental",
        "ecosystem" | "Ecosystem" | "eco" | "Eco" => "ecosystem",
        "opinionated" | "Opinionated" | "strict" | "all" | "opnionated" => "opinionated",
        "nuxt" | "Nuxt" => "nuxt",
        _ => preset,
    }
}

#[inline]
const fn rule_category_name(category: vize_patina::RuleCategory) -> &'static str {
    match category {
        vize_patina::RuleCategory::Essential => "Essential",
        vize_patina::RuleCategory::StronglyRecommended => "StronglyRecommended",
        vize_patina::RuleCategory::Recommended => "Recommended",
        vize_patina::RuleCategory::Vapor => "Vapor",
        vize_patina::RuleCategory::Musea => "Musea",
        vize_patina::RuleCategory::Accessibility => "Accessibility",
        vize_patina::RuleCategory::HtmlConformance => "HtmlConformance",
        vize_patina::RuleCategory::TypeAware => "TypeAware",
        vize_patina::RuleCategory::Ecosystem => "Ecosystem",
    }
}

#[inline]
const fn severity_name(severity: vize_patina::Severity) -> &'static str {
    match severity {
        vize_patina::Severity::Error => "error",
        vize_patina::Severity::Warning => "warning",
    }
}

fn collect_patina_rule_metadata() -> Vec<PatinaRuleMetaNapi<'static>> {
    use vize_patina::{LintPreset, RuleRegistry, builtin_script_rules};
    use vize_s0::FxHashSet;

    let template_rule_registries = [
        RuleRegistry::with_preset(LintPreset::Opinionated),
        RuleRegistry::with_ecosystem(),
        RuleRegistry::with_opt_in_rules(),
    ];
    let happy_path_rules: FxHashSet<&'static str> =
        RuleRegistry::with_preset(LintPreset::HappyPath)
            .rules()
            .iter()
            .map(|rule| rule.meta().name)
            .collect();
    let essential_rules: FxHashSet<&'static str> = RuleRegistry::with_preset(LintPreset::Essential)
        .rules()
        .iter()
        .map(|rule| rule.meta().name)
        .collect();
    let nuxt_rules: FxHashSet<&'static str> = RuleRegistry::with_preset(LintPreset::Nuxt)
        .rules()
        .iter()
        .map(|rule| rule.meta().name)
        .collect();
    let opinionated_rules: FxHashSet<&'static str> =
        RuleRegistry::with_preset(LintPreset::Opinionated)
            .rules()
            .iter()
            .map(|rule| rule.meta().name)
            .collect();
    let ecosystem_rules: FxHashSet<&'static str> = RuleRegistry::with_ecosystem()
        .rules()
        .iter()
        .map(|rule| rule.meta().name)
        .collect();

    let mut seen = FxHashSet::default();
    let mut rules = Vec::new();
    for registry in &template_rule_registries {
        for rule in registry.rules() {
            let meta = rule.meta();
            if !seen.insert(meta.name) {
                continue;
            }
            let mut presets = Vec::with_capacity(4);
            if essential_rules.contains(meta.name) {
                presets.push(plugin_preset_name(LintPreset::Essential));
            }
            if happy_path_rules.contains(meta.name) {
                presets.push(plugin_preset_name(LintPreset::HappyPath));
            }
            if nuxt_rules.contains(meta.name) {
                presets.push(plugin_preset_name(LintPreset::Nuxt));
            }
            if ecosystem_rules.contains(meta.name) {
                presets.push("ecosystem");
            }
            if opinionated_rules.contains(meta.name) {
                presets.push(plugin_preset_name(LintPreset::Opinionated));
            }

            rules.push(PatinaRuleMetaNapi {
                name: meta.name,
                description: meta.description,
                category: rule_category_name(meta.category),
                fixable: meta.fixable,
                default_severity: severity_name(meta.default_severity),
                presets,
            });
        }
    }

    for script_rule in builtin_script_rules() {
        rules.push(PatinaRuleMetaNapi {
            name: script_rule.name,
            description: script_rule.description,
            category: script_rule.category,
            fixable: script_rule.fixable,
            default_severity: severity_name(script_rule.default_severity),
            presets: script_rule
                .presets
                .iter()
                .map(|preset| plugin_preset_name_from_raw(preset))
                .collect(),
        });
    }

    rules
}

fn create_position_object(line: u32, column: u32, offset: u32) -> Value {
    json!({
        "line": line,
        "column": column,
        "offset": offset,
    })
}

fn create_location_object(
    start_line: u32,
    start_column: u32,
    start_offset: u32,
    end_line: u32,
    end_column: u32,
    end_offset: u32,
) -> Value {
    json!({
        "start": create_position_object(start_line, start_column, start_offset),
        "end": create_position_object(end_line, end_column, end_offset),
    })
}

/// Lint a single Vue SFC with Patina and return structured diagnostics.
#[napi(js_name = "lintPatinaSfc")]
pub fn lint_patina_sfc(source: String, options: Option<PatinaLintOptionsNapi>) -> Result<Value> {
    use vize_patina::{LspEmitter, Severity};

    let opts = options.unwrap_or_default();
    let filename = opts.filename.unwrap_or_else(|| "anonymous.vue".to_string());
    let locale = patina_locale_from_option(opts.locale.as_deref());
    let help_level = patina_help_level_from_option(opts.help_level.as_deref());
    let preset = patina_preset_from_option(opts.preset.as_deref());
    let enabled_rules = opts
        .enabled_rules
        .map(|rules| rules.into_iter().map(Into::into).collect());
    let linter = configure_type_aware_lint(
        create_patina_linter(preset)
            .with_locale(locale)
            .with_help_level(help_level),
        opts.type_aware,
        opts.corsa_path,
    )
    .with_enabled_rules(enabled_rules);
    let result = lint_source(&linter, &source, &filename);
    let lsp_diagnostics = LspEmitter::to_lsp_diagnostics_with_source(&result, &source);

    if result.diagnostics.len() != lsp_diagnostics.len() {
        return Err(Error::new(
            Status::GenericFailure,
            "Patina diagnostic conversion produced mismatched location metadata".to_string(),
        ));
    }

    let result_filename: &str = result.filename.as_ref();
    let diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .zip(lsp_diagnostics.iter())
        .map(|(diagnostic, lsp)| {
            let message: &str = diagnostic.message.as_ref();
            let help = diagnostic
                .help
                .as_ref()
                .map_or(Value::Null, |help| json!(help.as_ref() as &str));

            json!({
                "rule": diagnostic.rule_name,
                "severity": match diagnostic.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
            },
                "message": message,
                "location": create_location_object(
                    lsp.range.start.line + 1,
                    lsp.range.start.character + 1,
                    diagnostic.start,
                    lsp.range.end.line + 1,
                    lsp.range.end.character + 1,
                    diagnostic.end,
                ),
                "help": help,
            })
        })
        .collect();

    Ok(json!({
        "filename": result_filename,
        "errorCount": result.error_count as u32,
        "warningCount": result.warning_count as u32,
        "diagnostics": diagnostics,
    }))
}

/// Get Patina's currently registered rule metadata.
#[napi(js_name = "getPatinaRules")]
pub fn get_patina_rules() -> Result<Value> {
    let rule_metadata = collect_patina_rule_metadata();
    Ok(json!(
        rule_metadata
            .iter()
            .map(|rule| json!({
                "name": rule.name,
                "description": rule.description,
                "category": rule.category,
                "fixable": rule.fixable,
                "defaultSeverity": rule.default_severity,
                "presets": rule.presets,
            }))
            .collect::<Vec<_>>()
    ))
}

/// Lint Vue SFC files matching patterns (native multithreading, .gitignore-aware)
#[napi]
pub fn lint(patterns: Vec<String>, options: Option<LintOptionsNapi>) -> Result<LintResultNapi> {
    use std::time::Instant;
    use vize_patina::{HelpLevel, OutputFormat, format_results, format_summary};

    let opts = options.unwrap_or_default();
    let start = Instant::now();
    let format = opts
        .format
        .as_deref()
        .and_then(OutputFormat::parse)
        .unwrap_or(OutputFormat::Text);

    let files = file_collection::collect_lint_files(&patterns);

    if files.is_empty() {
        return Ok(LintResultNapi {
            output: empty_result::format_empty_lint_output(&patterns, format),
            error_count: 0,
            warning_count: 0,
            file_count: 0,
            time_ms: start.elapsed().as_secs_f64() * 1000.0,
        });
    }

    let help_level = match opts.help_level.as_deref() {
        Some("none") => HelpLevel::None,
        Some("short") => HelpLevel::Short,
        _ => HelpLevel::Full,
    };
    let preset = patina_preset_from_option(opts.preset.as_deref());
    let linter = configure_type_aware_lint(
        create_patina_linter(preset).with_help_level(help_level),
        opts.type_aware,
        opts.corsa_path,
    );
    let error_count = AtomicUsize::new(0);
    let warning_count = AtomicUsize::new(0);

    // Lint all files in parallel and collect results
    let should_fix = opts.fix.unwrap_or(false);
    let results: Vec<_> = files
        .par_iter()
        .filter_map(|path| {
            let item = lint_file_with_optional_fix(&linter, path, should_fix)?;
            error_count.fetch_add(item.2.error_count, Ordering::Relaxed);
            warning_count.fetch_add(item.2.warning_count, Ordering::Relaxed);
            Some(item)
        })
        .collect();

    let total_errors = error_count.load(Ordering::Relaxed);
    let total_warnings = warning_count.load(Ordering::Relaxed);

    let quiet = opts.quiet.unwrap_or(false);

    // Format output
    let mut output = vize_s0::CompactString::default();
    if format.renders_details_when_quiet() || !quiet || total_errors > 0 || total_warnings > 0 {
        let lint_results: Vec<_> = results.iter().map(|(_, _, r)| r).cloned().collect();
        let sources: Vec<_> = results
            .iter()
            .map(|(f, s, _)| {
                (
                    vize_s0::CompactString::from(f.as_str()),
                    vize_s0::CompactString::from(s.as_str()),
                )
            })
            .collect();

        let formatted = format_results(&lint_results, &sources, format);
        if !formatted.trim().is_empty() {
            output.push_str(&formatted);
        }
    }

    let elapsed = start.elapsed();
    if format == OutputFormat::Text {
        append!(
            output,
            "\n{}\n",
            format_summary(total_errors, total_warnings, files.len())
        );
        append!(output, "Linted {} files in {:.4?}", files.len(), elapsed);
    }

    Ok(LintResultNapi {
        output: output.into(),
        error_count: total_errors as u32,
        warning_count: total_warnings as u32,
        file_count: files.len() as u32,
        time_ms: elapsed.as_secs_f64() * 1000.0,
    })
}

#[cfg(test)]
mod tests;
