//! Lint text-edit application for the native API.

use std::{fs, path::Path};
use vize_s0::{String, ToCompactString};

pub(super) fn lint_file_with_optional_fix(
    linter: &vize_patina::Linter,
    path: &Path,
    should_fix: bool,
) -> Option<(String, String, vize_patina::LintResult)> {
    let mut source: String = fs::read_to_string(path).ok()?.into();
    let filename = path.to_string_lossy().to_compact_string();
    let original = source.clone();
    let mut result = lint_source(linter, &source, &filename);
    if should_fix {
        // Match ESLint's repeated fix passes. This matters when an outer Nuxt
        // config sort overlaps nested `$environment` sorts: the outer edit is
        // selected first, then the nested objects converge on the next pass.
        for _ in 0..10 {
            let Some(fixed_source) = apply_lint_fixes(&source, &result) else {
                break;
            };
            if fixed_source == source {
                break;
            }
            source = fixed_source;
            result = lint_source(linter, &source, &filename);
        }
        // Replace the file only once its successor is fully written, so a
        // failed write leaves the original config on disk instead of a
        // truncated one.
        if source != original && vize::source_write::atomic_write(path, source.as_bytes()).is_err()
        {
            source = original;
            result = lint_source(linter, &source, &filename);
        }
    }
    Some((filename, source, result))
}

pub(super) fn lint_source(
    linter: &vize_patina::Linter,
    source: &str,
    filename: &str,
) -> vize_patina::LintResult {
    if is_standalone_html_filename(filename) {
        linter.lint_standalone_html(source, filename)
    } else if is_script_filename(filename) {
        linter.lint_script(source, filename)
    } else {
        linter.lint_sfc(source, filename)
    }
}

pub(super) fn is_standalone_html_filename(filename: &str) -> bool {
    filename.ends_with(".html") || filename.ends_with(".htm")
}

pub(super) fn is_lintable_extension(extension: &str) -> bool {
    matches!(extension, "vue" | "html" | "htm")
}

fn is_script_filename(filename: &str) -> bool {
    Path::new(filename)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension,
                "js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx" | "mts" | "cts"
            )
        })
}

fn apply_lint_fixes(source: &str, result: &vize_patina::LintResult) -> Option<String> {
    let mut edits: Vec<&vize_patina::TextEdit> = result
        .diagnostics
        .iter()
        .filter_map(|diagnostic| diagnostic.fix.as_ref())
        .flat_map(|fix| fix.edits.iter())
        .filter(|edit| {
            let start = edit.start as usize;
            let end = edit.end as usize;
            start <= end
                && end <= source.len()
                && source.is_char_boundary(start)
                && source.is_char_boundary(end)
        })
        .collect();

    if edits.is_empty() {
        return None;
    }

    edits.sort_by_key(|edit| (edit.start, edit.end));
    let mut selected = Vec::with_capacity(edits.len());
    let mut last_end = 0u32;
    for edit in edits {
        if edit.start < last_end {
            continue;
        }
        last_end = edit.end;
        selected.push(edit);
    }

    if selected.is_empty() {
        return None;
    }

    let mut fixed = source.to_compact_string();
    for edit in selected.into_iter().rev() {
        fixed.replace_range(edit.start as usize..edit.end as usize, &edit.new_text);
    }
    Some(fixed)
}

#[cfg(test)]
mod tests {
    use super::{
        is_lintable_extension, is_script_filename, lint_file_with_optional_fix, lint_source,
    };
    use std::fs;
    use vize_patina::{LintPreset, Linter};

    #[test]
    fn standalone_typescript_is_linted_as_a_script() {
        let linter = Linter::with_preset(LintPreset::Nuxt);
        let result = lint_source(
            &linter,
            "export default defineNuxtConfig({ test: true })",
            "nuxt.config.ts",
        );

        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.rule_name == "nuxt/no-nuxt-config-test-key" })
        );
    }

    #[test]
    fn script_extensions_do_not_expand_directory_collection() {
        for (filename, extension) in [
            ("nuxt.config.js", "js"),
            ("nuxt.config.jsx", "jsx"),
            ("nuxt.config.mjs", "mjs"),
            ("nuxt.config.cjs", "cjs"),
            ("nuxt.config.ts", "ts"),
            ("nuxt.config.tsx", "tsx"),
            ("nuxt.config.mts", "mts"),
            ("nuxt.config.cts", "cts"),
        ] {
            assert!(is_script_filename(filename));
            assert!(!is_lintable_extension(extension));
        }
    }

    #[test]
    fn overlapping_config_order_fixes_converge_before_writing() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("nuxt.config.ts");
        fs::write(
            &path,
            "export default { ssr: true, $test: { ssr: true, modules: [] }, modules: [] }",
        )
        .unwrap();
        let linter = Linter::with_preset(LintPreset::Nuxt);
        let (_, source, result) = lint_file_with_optional_fix(&linter, &path, true).unwrap();
        assert_eq!(
            source,
            "export default { modules: [], $test: { modules: [], ssr: true, }, ssr: true, }"
        );
        assert!(
            result
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.rule_name != "nuxt/nuxt-config-keys-order")
        );
        assert_eq!(fs::read_to_string(path).unwrap(), source);
    }
}
