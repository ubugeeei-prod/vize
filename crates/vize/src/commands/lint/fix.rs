//! Lint source selection and fix application.

use std::path::Path;

use super::collect::{is_plain_script_path, is_standalone_html_path};
use vize_carton::{String, ToCompactString};
use vize_patina::{LintResult, Linter, TextEdit};

pub(super) fn lint_source(
    linter: &Linter,
    path: &Path,
    source: &str,
    filename: &str,
) -> LintResult {
    if is_standalone_html_path(path) {
        linter.lint_standalone_html(source, filename)
    } else if is_plain_script_path(path) {
        linter.lint_script(source, filename)
    } else if is_storybook_csf_path(path) {
        empty_lint_result(filename)
    } else {
        unreachable!("Vue and JSX lint must be served by the Atlas artifact graph")
    }
}

pub(super) fn apply_lint_fixes(source: &str, result: &LintResult) -> Option<String> {
    let mut edits: Vec<&TextEdit> = result
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

fn is_storybook_csf_path(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|file_name| file_name.to_str()) else {
        return false;
    };
    file_name.ends_with(".stories.jsx")
        || file_name.ends_with(".stories.tsx")
        || file_name.ends_with(".story.jsx")
        || file_name.ends_with(".story.tsx")
}

fn empty_lint_result(filename: &str) -> LintResult {
    LintResult {
        filename: filename.to_compact_string(),
        diagnostics: Vec::new(),
        error_count: 0,
        warning_count: 0,
    }
}
