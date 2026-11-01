//! Refuse a formatter/linter policy cycle before writing any selected file.

use crate::config::{LoadedFormatterSnapshot, formatter_lint_plan::FormatterLinterPlan};
use std::path::{Path, PathBuf};
use vize_glyph::FormatOptions;
use vize_l0::{String, cstr};
use vize_patina::Linter;

pub(super) fn validate_selected_files(
    files: &[PathBuf],
    snapshot: &LoadedFormatterSnapshot,
    options: &FormatOptions,
    config_dir: &Path,
) -> Result<(), String> {
    if !options.normalize_directive_shorthands {
        return Ok(());
    }
    let plan = FormatterLinterPlan::new(snapshot.linter_plan.clone(), config_dir);
    for path in files {
        if !path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("vue"))
        {
            continue;
        }
        if let Some(resolved) = plan.resolve(path) {
            Linter::validate_directive_normalization(
                &resolved.config,
                &resolved.rule_options,
                options.normalize_directive_shorthands,
            )
            .map_err(|error| cstr!("{}: {error}", path.display()))?;
        }
    }
    Ok(())
}
