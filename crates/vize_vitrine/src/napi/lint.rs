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

use glob::glob;
use napi::{
    bindgen_prelude::{Error, Object, Result, Status},
    Env,
};
use napi_derive::napi;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::{
    fs,
    sync::atomic::{AtomicUsize, Ordering},
};
use vize_carton::append;

struct PatinaRuleMetaNapi<'a> {
    name: &'a str,
    description: &'a str,
    category: &'a str,
    fixable: bool,
    default_severity: &'a str,
    presets: Vec<&'static str>,
}

/// Lint options for NAPI
#[napi(object)]
#[derive(Default)]
pub struct LintOptionsNapi {
    /// Output format: "text" or "json"
    pub format: Option<String>,
    /// Maximum number of warnings before failing
    pub max_warnings: Option<u32>,
    /// Quiet mode - only show summary
    pub quiet: Option<bool>,
    /// Automatically fix problems (not yet implemented)
    pub fix: Option<bool>,
    /// Help display level: "full", "short", "none"
    pub help_level: Option<String>,
    /// Lint preset: "general-recommended", "essential", "incremental", "opinionated", or "nuxt"
    pub preset: Option<String>,
}

/// Lint result for NAPI
#[napi(object)]
pub struct LintResultNapi {
    /// Formatted output string
    pub output: String,
    /// Total number of errors
    pub error_count: u32,
    /// Total number of warnings
    pub warning_count: u32,
    /// Number of files linted
    pub file_count: u32,
    /// Time in milliseconds
    pub time_ms: f64,
}

/// Single-file Patina lint options for NAPI
#[napi(object)]
#[derive(Default)]
pub struct PatinaLintOptionsNapi {
    /// Filename used for diagnostics
    pub filename: Option<String>,
    /// Locale code: "en", "ja", or "zh"
    pub locale: Option<String>,
    /// Help display level: "full", "short", or "none"
    pub help_level: Option<String>,
    /// Lint preset: "general-recommended", "essential", "incremental", "opinionated", or "nuxt"
    pub preset: Option<String>,
    /// Optional list of Patina rule names to enable
    pub enabled_rules: Option<Vec<String>>,
}

fn patina_locale_from_option(locale: Option<&str>) -> vize_patina::Locale {
    locale
        .and_then(vize_patina::Locale::parse)
        .unwrap_or_default()
}

fn patina_help_level_from_option(help_level: Option<&str>) -> vize_patina::HelpLevel {
    match help_level {
        Some("none") => vize_patina::HelpLevel::None,
        Some("short") => vize_patina::HelpLevel::Short,
        _ => vize_patina::HelpLevel::Full,
    }
}

fn patina_preset_from_option(preset: Option<&str>) -> vize_patina::LintPreset {
    match preset {
        Some("general-recommended" | "GeneralRecommended" | "generalRecommended")
        | Some("happy-path" | "happy_path" | "happy" | "default" | "recommended") => {
            vize_patina::LintPreset::HappyPath
        }
        Some("essential" | "Essential") => vize_patina::LintPreset::Essential,
        Some("incremental" | "Incremental") => vize_patina::LintPreset::Incremental,
        Some("opinionated" | "Opinionated" | "Opnionated" | "opnionated" | "strict" | "all") => {
            vize_patina::LintPreset::Opinionated
        }
        Some("nuxt" | "Nuxt") => vize_patina::LintPreset::Nuxt,
        _ => vize_patina::LintPreset::default(),
    }
}

#[inline]
const fn plugin_preset_name(preset: vize_patina::LintPreset) -> &'static str {
    match preset {
        vize_patina::LintPreset::HappyPath => "general-recommended",
        vize_patina::LintPreset::Opinionated => "opinionated",
        vize_patina::LintPreset::Essential => "essential",
        vize_patina::LintPreset::Incremental => "incremental",
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
    use vize_carton::FxHashSet;
    use vize_patina::{builtin_script_rules, LintPreset, Linter, RuleRegistry};

    let linter = Linter::with_preset(LintPreset::Opinionated);
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

    let mut rules: Vec<_> = linter
        .rules()
        .iter()
        .map(|rule| {
            let meta = rule.meta();
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
            presets.push(plugin_preset_name(LintPreset::Opinionated));

            PatinaRuleMetaNapi {
                name: meta.name,
                description: meta.description,
                category: rule_category_name(meta.category),
                fixable: meta.fixable,
                default_severity: severity_name(meta.default_severity),
                presets,
            }
        })
        .collect();

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

fn create_position_object(env: Env, line: u32, column: u32, offset: u32) -> Result<Object> {
    let mut obj = env.create_object()?;
    obj.set("line", line)?;
    obj.set("column", column)?;
    obj.set("offset", offset)?;
    Ok(obj)
}

fn create_location_object(
    env: Env,
    start_line: u32,
    start_column: u32,
    start_offset: u32,
    end_line: u32,
    end_column: u32,
    end_offset: u32,
) -> Result<Object> {
    let mut obj = env.create_object()?;
    obj.set(
        "start",
        create_position_object(env, start_line, start_column, start_offset)?,
    )?;
    obj.set(
        "end",
        create_position_object(env, end_line, end_column, end_offset)?,
    )?;
    Ok(obj)
}

/// Lint a single Vue SFC with Patina and return structured diagnostics.
#[napi(js_name = "lintPatinaSfc")]
pub fn lint_patina_sfc(
    env: Env,
    source: String,
    options: Option<PatinaLintOptionsNapi>,
) -> Result<Object> {
    use vize_patina::{Linter, LspEmitter, Severity};

    let opts = options.unwrap_or_default();
    let filename = opts.filename.unwrap_or_else(|| "anonymous.vue".to_string());
    let locale = patina_locale_from_option(opts.locale.as_deref());
    let help_level = patina_help_level_from_option(opts.help_level.as_deref());
    let preset = patina_preset_from_option(opts.preset.as_deref());
    let enabled_rules = opts
        .enabled_rules
        .map(|rules| rules.into_iter().map(Into::into).collect());
    let linter = Linter::with_preset(preset)
        .with_locale(locale)
        .with_help_level(help_level)
        .with_enabled_rules(enabled_rules);
    let result = linter.lint_sfc(&source, &filename);
    let lsp_diagnostics = LspEmitter::to_lsp_diagnostics_with_source(&result, &source);

    if result.diagnostics.len() != lsp_diagnostics.len() {
        return Err(Error::new(
            Status::GenericFailure,
            "Patina diagnostic conversion produced mismatched location metadata".to_string(),
        ));
    }

    let mut output = env.create_object()?;
    let result_filename: &str = result.filename.as_ref();
    output.set("filename", result_filename)?;
    output.set("errorCount", result.error_count as u32)?;
    output.set("warningCount", result.warning_count as u32)?;

    let mut diagnostics = env.create_array(result.diagnostics.len() as u32)?;
    for (index, (diagnostic, lsp)) in result
        .diagnostics
        .iter()
        .zip(lsp_diagnostics.iter())
        .enumerate()
    {
        let mut obj = env.create_object()?;
        obj.set("rule", diagnostic.rule_name)?;
        obj.set(
            "severity",
            match diagnostic.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
            },
        )?;
        let message: &str = diagnostic.message.as_ref();
        obj.set("message", message)?;
        obj.set(
            "location",
            create_location_object(
                env,
                lsp.range.start.line + 1,
                lsp.range.start.character + 1,
                diagnostic.start,
                lsp.range.end.line + 1,
                lsp.range.end.character + 1,
                diagnostic.end,
            )?,
        )?;
        if let Some(help) = diagnostic.help.as_ref() {
            let help_text: &str = help.as_ref();
            obj.set("help", help_text)?;
        } else {
            obj.set("help", env.get_null()?)?;
        }
        diagnostics.set(index as u32, obj)?;
    }

    output.set("diagnostics", diagnostics)?;
    Ok(output)
}

/// Get Patina's currently registered rule metadata.
#[napi(js_name = "getPatinaRules")]
pub fn get_patina_rules(env: Env) -> Result<napi::bindgen_prelude::Array> {
    let rule_metadata = collect_patina_rule_metadata();
    let mut rules = env.create_array(rule_metadata.len() as u32)?;

    for (index, rule) in rule_metadata.iter().enumerate() {
        let mut obj = env.create_object()?;
        obj.set("name", rule.name)?;
        obj.set("description", rule.description)?;
        obj.set("category", rule.category)?;
        obj.set("fixable", rule.fixable)?;
        obj.set("defaultSeverity", rule.default_severity)?;
        let mut presets = env.create_array(rule.presets.len() as u32)?;
        for (preset_index, preset) in rule.presets.iter().enumerate() {
            presets.set(preset_index as u32, *preset)?;
        }
        obj.set("presets", presets)?;
        rules.set(index as u32, obj)?;
    }

    Ok(rules)
}

/// Lint Vue SFC files matching patterns (native multithreading, .gitignore-aware)
#[napi]
pub fn lint(patterns: Vec<String>, options: Option<LintOptionsNapi>) -> Result<LintResultNapi> {
    use ignore::Walk;
    use std::time::Instant;
    use vize_patina::{format_results, format_summary, HelpLevel, Linter, OutputFormat};

    let opts = options.unwrap_or_default();
    let start = Instant::now();

    // Collect .vue files using glob patterns or directory walking
    let files: Vec<std::path::PathBuf> = patterns
        .iter()
        .flat_map(|pattern| {
            if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
                // Use glob for pattern matching
                glob(pattern)
                    .ok()
                    .into_iter()
                    .flatten()
                    .filter_map(|r| r.ok())
                    .filter(|p| {
                        p.extension().is_some_and(|ext| ext == "vue")
                            && !p.components().any(|c| c.as_os_str() == "node_modules")
                    })
                    .collect::<Vec<_>>()
            } else {
                // Use directory walking for paths (respects .gitignore)
                Walk::new(pattern)
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().is_some_and(|ext| ext == "vue"))
                    .map(|e| e.path().to_path_buf())
                    .collect::<Vec<_>>()
            }
        })
        .collect();

    if files.is_empty() {
        return Ok(LintResultNapi {
            output: format!("No .vue files found matching patterns: {:?}", patterns),
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
    let linter = Linter::with_preset(preset).with_help_level(help_level);
    let error_count = AtomicUsize::new(0);
    let warning_count = AtomicUsize::new(0);

    // Lint all files in parallel and collect results
    let results: Vec<_> = files
        .par_iter()
        .filter_map(|path| {
            let source = match fs::read_to_string(path) {
                Ok(s) => s,
                Err(_) => return None,
            };

            let filename = path.to_string_lossy().to_string();
            let result = linter.lint_sfc(&source, &filename);

            error_count.fetch_add(result.error_count, Ordering::Relaxed);
            warning_count.fetch_add(result.warning_count, Ordering::Relaxed);

            Some((filename, source, result))
        })
        .collect();

    let total_errors = error_count.load(Ordering::Relaxed);
    let total_warnings = warning_count.load(Ordering::Relaxed);

    let format = match opts.format.as_deref() {
        Some("json") => OutputFormat::Json,
        _ => OutputFormat::Text,
    };

    let quiet = opts.quiet.unwrap_or(false);

    // Format output
    let mut output = vize_carton::CompactString::default();
    if !quiet || total_errors > 0 || total_warnings > 0 {
        let lint_results: Vec<_> = results.iter().map(|(_, _, r)| r).cloned().collect();
        let sources: Vec<_> = results
            .iter()
            .map(|(f, s, _)| {
                (
                    vize_carton::CompactString::from(f.as_str()),
                    vize_carton::CompactString::from(s.as_str()),
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
mod tests {
    use super::collect_patina_rule_metadata;

    #[test]
    fn patina_rule_metadata_includes_happy_path_membership() {
        let rules = collect_patina_rule_metadata();
        let require_scoped_style = rules
            .iter()
            .find(|rule| rule.name == "vue/require-scoped-style")
            .expect("vue/require-scoped-style should be exposed");

        assert_eq!(
            require_scoped_style.presets,
            vec!["general-recommended", "nuxt", "opinionated"]
        );
    }

    #[test]
    fn patina_rule_metadata_includes_opinionated_script_rules() {
        let rules = collect_patina_rule_metadata();
        let no_options_api = rules
            .iter()
            .find(|rule| rule.name == "script/no-options-api")
            .expect("script/no-options-api should be exposed");

        assert_eq!(no_options_api.presets, vec!["opinionated", "nuxt"]);
        assert_eq!(no_options_api.default_severity, "error");
    }
}
