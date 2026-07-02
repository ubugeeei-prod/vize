use napi_derive::napi;
use std::path::PathBuf;

/// Lint options for NAPI
#[napi(object)]
#[derive(Default)]
pub struct LintOptionsNapi {
    /// Output format: "text", "ansi", "plain", "json", "stylish", "markdown", "html", or "agent"
    pub format: Option<String>,
    /// Maximum number of warnings before failing
    pub max_warnings: Option<u32>,
    /// Quiet mode - only show summary
    pub quiet: Option<bool>,
    /// Automatically fix problems when diagnostics provide safe text edits
    pub fix: Option<bool>,
    /// Help display level: "full", "short", "none"
    pub help_level: Option<String>,
    /// Lint preset: "general-recommended", "essential", "incremental", "ecosystem", "opinionated", or "nuxt"
    pub preset: Option<String>,
    /// Enable native type-aware lint rules
    pub type_aware: Option<bool>,
    /// Path to the Corsa executable used by type-aware lint rules
    pub corsa_path: Option<String>,
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
    /// Lint preset: "general-recommended", "essential", "incremental", "ecosystem", "opinionated", or "nuxt"
    pub preset: Option<String>,
    /// Optional list of Patina rule names to enable
    pub enabled_rules: Option<Vec<String>>,
    /// Enable native type-aware lint rules
    pub type_aware: Option<bool>,
    /// Path to the Corsa executable used by type-aware lint rules
    pub corsa_path: Option<String>,
}

pub(super) enum PatinaPresetSelection {
    Builtin(vize_patina::LintPreset),
    Ecosystem,
}

pub(super) fn patina_locale_from_option(locale: Option<&str>) -> vize_patina::Locale {
    locale
        .and_then(vize_patina::Locale::parse)
        .unwrap_or_default()
}

pub(super) fn patina_help_level_from_option(help_level: Option<&str>) -> vize_patina::HelpLevel {
    match help_level {
        Some("none") => vize_patina::HelpLevel::None,
        Some("short") => vize_patina::HelpLevel::Short,
        _ => vize_patina::HelpLevel::Full,
    }
}

pub(super) fn patina_preset_from_option(preset: Option<&str>) -> PatinaPresetSelection {
    match preset {
        Some("general-recommended" | "GeneralRecommended" | "generalRecommended")
        | Some("happy-path" | "happy_path" | "happy" | "default" | "recommended") => {
            PatinaPresetSelection::Builtin(vize_patina::LintPreset::HappyPath)
        }
        Some("essential" | "Essential") => {
            PatinaPresetSelection::Builtin(vize_patina::LintPreset::Essential)
        }
        Some("incremental" | "Incremental") => {
            PatinaPresetSelection::Builtin(vize_patina::LintPreset::Incremental)
        }
        Some("ecosystem" | "Ecosystem" | "eco" | "Eco") => PatinaPresetSelection::Ecosystem,
        Some("opinionated" | "Opinionated" | "Opnionated" | "opnionated" | "strict" | "all") => {
            PatinaPresetSelection::Builtin(vize_patina::LintPreset::Opinionated)
        }
        Some("nuxt" | "Nuxt") => PatinaPresetSelection::Builtin(vize_patina::LintPreset::Nuxt),
        _ => PatinaPresetSelection::Builtin(vize_patina::LintPreset::default()),
    }
}

pub(super) fn create_patina_linter(preset: PatinaPresetSelection) -> vize_patina::Linter {
    match preset {
        PatinaPresetSelection::Builtin(preset) => vize_patina::Linter::with_preset(preset),
        PatinaPresetSelection::Ecosystem => vize_patina::Linter::with_ecosystem(),
    }
}

pub(super) fn configure_type_aware_lint(
    linter: vize_patina::Linter,
    type_aware: Option<bool>,
    corsa_path: Option<String>,
) -> vize_patina::Linter {
    linter
        .with_type_aware_lint(type_aware.unwrap_or(false))
        .with_corsa_path(corsa_path.map(PathBuf::from))
}
