//! Lint text-edit application for the native API.

use std::{fs, path::Path};
use vize_carton::{String, ToCompactString};

pub(super) fn lint_file_with_optional_fix(
    linter: &vize_patina::Linter,
    path: &Path,
    should_fix: bool,
) -> Option<(String, String, vize_patina::LintResult)> {
    let mut source: String = fs::read_to_string(path).ok()?.into();
    let filename = path.to_string_lossy().to_compact_string();
    let initial_result = lint_source(linter, &source, &filename);
    let result = if should_fix
        && let Some(fixed_source) = apply_lint_fixes(&source, &initial_result)
        && fixed_source != source
        && fs::write(path, fixed_source.as_bytes()).is_ok()
    {
        source = fixed_source;
        lint_source(linter, &source, &filename)
    } else {
        initial_result
    };
    Some((filename, source, result))
}

pub(super) fn lint_source(
    linter: &vize_patina::Linter,
    source: &str,
    filename: &str,
) -> vize_patina::LintResult {
    if is_standalone_html_filename(filename) {
        linter.lint_standalone_html(source, filename)
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
