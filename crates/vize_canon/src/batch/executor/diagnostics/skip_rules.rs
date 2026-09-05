//! Code/message-level suppression rules shared by the LSP and CLI diagnostic
//! paths. These decide whether a TypeScript diagnostic is reportable at all,
//! independent of where it maps back to in the original source.

use std::{
    fs,
    path::{Path, PathBuf},
};

use super::{Diagnostic, OriginalPosition};
use vize_carton::FxHashSet;

type DiagnosticLineKey = (PathBuf, u32);

pub(crate) fn should_skip_diagnostic(code: Option<u32>, message: &str) -> bool {
    match code {
        // TS2666: virtual-TS generation injects helper bindings that can trip
        // this code outside the user's source — suppress to match vue-tsc.
        Some(2666) => true,
        // Native TypeScript currently exposes Node Buffer backing stores as
        // `ArrayBuffer | SharedArrayBuffer`, while projects pinned to older
        // TypeScript/@types/node combinations accepted `buffer.slice(...)` as
        // `ArrayBuffer`. Keep vize aligned with that project baseline until the
        // native checker can select the project's exact lib surface.
        Some(2322) if is_array_buffer_backing_store_lib_mismatch(message) => true,
        // TS7006/TS7043/TS7044 (noImplicitAny family) are user-facing errors
        // and must surface so `vize check` matches vue-tsc under
        // `noImplicitAny`/`strict`. They were previously suppressed (#966).
        _ => false,
    }
}

fn is_array_buffer_backing_store_lib_mismatch(message: &str) -> bool {
    message
        .contains("Type 'ArrayBuffer | SharedArrayBuffer' is not assignable to type 'ArrayBuffer'")
        && message.contains("SharedArrayBuffer")
}

pub(crate) fn should_skip_original_diagnostic(
    code: Option<u32>,
    original: &OriginalPosition,
) -> bool {
    code == Some(6133) && original.block_type.is_none() && is_vue_source(&original.path)
}

pub(crate) fn filter_authored_diagnostics(diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
    if !diagnostics
        .iter()
        .any(is_use_vmodel_passive_false_overload_diagnostic)
    {
        return diagnostics;
    }
    let unused_expect_errors = unused_expect_error_lines(&diagnostics);

    diagnostics
        .into_iter()
        .filter(|diagnostic| !should_skip_authored_diagnostic(diagnostic, &unused_expect_errors))
        .collect()
}

fn should_skip_authored_diagnostic(
    diagnostic: &Diagnostic,
    unused_expect_errors: &FxHashSet<DiagnosticLineKey>,
) -> bool {
    if !is_use_vmodel_passive_false_overload_diagnostic(diagnostic) {
        return false;
    }
    let Ok(source) = fs::read_to_string(&diagnostic.file) else {
        return false;
    };
    let Some(matched) = use_vmodel_passive_false_match(&source, diagnostic.line as usize) else {
        return false;
    };
    let key = (diagnostic.file.clone(), matched.expect_error_line as u32);
    !unused_expect_errors.contains(&key)
}

fn is_use_vmodel_passive_false_overload_diagnostic(diagnostic: &Diagnostic) -> bool {
    // The CLI parser sees only the headline before continuation lines are
    // attached, so the source context is the stable part of this parity rule.
    diagnostic.code == Some(2769)
        && is_vue_source(&diagnostic.file)
        && diagnostic.message.contains("No overload matches this call")
}

struct UseVModelPassiveFalseMatch {
    expect_error_line: usize,
}

fn use_vmodel_passive_false_match(
    source: &str,
    diagnostic_line: usize,
) -> Option<UseVModelPassiveFalseMatch> {
    let lines: Vec<_> = source.lines().collect();
    let (start, end) = containing_use_vmodel_call(&lines, diagnostic_line)?;
    if !call_has_passive_false(&lines, start, end) {
        return None;
    }
    let expect_error_line = expect_error_before_default_value(&lines, start, end)?;

    Some(UseVModelPassiveFalseMatch { expect_error_line })
}

fn containing_use_vmodel_call(lines: &[&str], diagnostic_line: usize) -> Option<(usize, usize)> {
    if diagnostic_line >= lines.len() {
        return None;
    }
    for start in (0..=diagnostic_line).rev() {
        if !lines[start].contains("useVModel(") {
            continue;
        }
        let Some(end) = call_end_line(lines, start) else {
            continue;
        };
        if diagnostic_line <= end {
            return Some((start, end));
        }
    }
    None
}

fn call_end_line(lines: &[&str], start: usize) -> Option<usize> {
    let mut depth = 0i32;
    let mut saw_open = false;
    for (index, line) in lines.iter().enumerate().skip(start) {
        for ch in line.chars() {
            if ch == '(' {
                saw_open = true;
                depth += 1;
            } else if ch == ')' && saw_open {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
        }
    }
    None
}

fn call_has_passive_false(lines: &[&str], start: usize, end: usize) -> bool {
    let mut saw_passive = false;
    for line in &lines[start..=end] {
        let line = line.trim();
        if line.contains("passive:") {
            saw_passive = true;
        }
        if saw_passive && line.contains("as false") {
            return true;
        }
    }
    false
}

fn expect_error_before_default_value(lines: &[&str], start: usize, end: usize) -> Option<usize> {
    let mut expect_error_line = None;
    for (index, line) in lines.iter().enumerate().take(end + 1).skip(start) {
        let line = line.trim();
        if line.contains("@ts-expect-error") {
            expect_error_line = Some(index);
        }
        if line.contains("defaultValue:") {
            return expect_error_line;
        }
    }
    None
}

fn unused_expect_error_lines(diagnostics: &[Diagnostic]) -> FxHashSet<DiagnosticLineKey> {
    diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.code == Some(2578)
                && diagnostic
                    .message
                    .contains("Unused '@ts-expect-error' directive")
        })
        .map(|diagnostic| (diagnostic.file.clone(), diagnostic.line))
        .collect()
}

fn is_vue_source(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "vue")
}

#[cfg(test)]
mod tests {
    use super::{filter_authored_diagnostics, use_vmodel_passive_false_match};
    use crate::batch::Diagnostic;
    use std::{
        fs,
        path::{Path, PathBuf},
    };
    use tempfile::TempDir;

    const PASSIVE_FALSE_OVERLOAD: &str = "No overload matches this call.\n\
The last overload gave the following error.\n\
Type 'false' is not assignable to type 'true'.";

    #[test]
    fn filters_use_vmodel_passive_false_overload_after_consumed_expect_error() {
        let source = r#"useVModel(props, "modelValue", emit, {
  // @ts-expect-error Missing infer for AcceptableValue
  defaultValue: props.defaultValue ?? (multiple.value ? [] : undefined),
  passive: (props.modelValue === undefined) as false,
  deep: true,
});
"#;
        let temp = TempDir::new().unwrap();
        let file = write_vue(&temp, source);
        let diagnostic = overload(&file, line_of(source, "passive:"));

        assert!(filter_authored_diagnostics(vec![diagnostic]).is_empty());
    }

    #[test]
    fn keeps_use_vmodel_passive_false_overload_without_expect_error() {
        let source = r#"useVModel(props, "modelValue", emit, {
  defaultValue: props.defaultValue,
  passive: (props.modelValue === undefined) as false,
  deep: true,
});
"#;

        assert!(
            use_vmodel_passive_false_match(source, line_of(source, "passive:") as usize).is_none()
        );
    }

    #[test]
    fn keeps_use_vmodel_overload_when_expect_error_is_unused() {
        let source = r#"useVModel(props, "modelValue", emit, {
  // @ts-expect-error Missing infer for AcceptableValue
  defaultValue: props.defaultValue ?? (multiple.value ? [] : undefined),
  passive: (props.modelValue === undefined) as false,
  deep: true,
});
"#;
        let temp = TempDir::new().unwrap();
        let file = write_vue(&temp, source);
        let diagnostics = filter_authored_diagnostics(vec![
            overload(&file, line_of(source, "passive:")),
            unused_directive(&file, line_of(source, "@ts-expect-error")),
        ]);
        let codes: Vec<_> = diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect();

        assert_eq!(codes, [Some(2769), Some(2578)]);
    }

    #[test]
    fn matches_multiline_passive_false_option() {
        let source = r#"useVModel(props, "modelValue", emit, {
  // @ts-expect-error Missing infer for AcceptableValue
  defaultValue: props.defaultValue ?? (multiple.value ? [] : undefined),
  passive:
    (props.modelValue === undefined)
      as false,
  deep: true,
});
"#;
        let matched =
            use_vmodel_passive_false_match(source, line_of(source, "passive:") as usize).unwrap();

        assert_eq!(
            matched.expect_error_line,
            line_of(source, "@ts-expect-error") as usize
        );
    }

    #[test]
    fn matches_use_vmodel_options_after_long_gap() {
        let source = r#"useVModel(props, "modelValue", emit, {
  option01: true,
  option02: true,
  option03: true,
  option04: true,
  option05: true,
  option06: true,
  option07: true,
  option08: true,
  option09: true,
  option10: true,
  option11: true,
  option12: true,
  option13: true,
  // @ts-expect-error Missing infer for AcceptableValue
  defaultValue: props.defaultValue ?? (multiple.value ? [] : undefined),
  passive: (props.modelValue === undefined) as false,
  deep: true,
});
"#;
        let matched =
            use_vmodel_passive_false_match(source, line_of(source, "passive:") as usize).unwrap();

        assert_eq!(
            matched.expect_error_line,
            line_of(source, "@ts-expect-error") as usize
        );
    }

    fn write_vue(temp: &TempDir, source: &str) -> PathBuf {
        let file = temp.path().join("Foo.vue");
        fs::write(&file, source).unwrap();
        file
    }

    fn overload(file: &Path, line: u32) -> Diagnostic {
        diagnostic(file, line, Some(2769), PASSIVE_FALSE_OVERLOAD)
    }

    fn unused_directive(file: &Path, line: u32) -> Diagnostic {
        diagnostic(
            file,
            line,
            Some(2578),
            "Unused '@ts-expect-error' directive.",
        )
    }

    fn diagnostic(file: &Path, line: u32, code: Option<u32>, message: &str) -> Diagnostic {
        Diagnostic {
            file: file.to_path_buf(),
            line,
            column: 2,
            message: message.into(),
            code,
            severity: 1,
            block_type: None,
        }
    }

    fn line_of(source: &str, needle: &str) -> u32 {
        source
            .lines()
            .position(|line| line.contains(needle))
            .unwrap() as u32
    }
}
