//! Suppression comments the checker cannot honor through the projection.
//!
//! A `// @ts-ignore` / `// @ts-expect-error` above a multi-line call covers
//! the whole authored call, but the projection may split it so the checker
//! reports a line the comment no longer precedes. The same holds for the
//! VueUse `useVModel(..., { passive: … as false })` overload the ecosystem
//! silences with an `@ts-expect-error` above `defaultValue:`.

use vize_carton::FxHashSet;

use super::{AssembledDiagnostic, AuthoredSource};

const UNUSED_TS_EXPECT_ERROR: u32 = 2578;

/// Drop diagnostics an authored suppression comment covers.
pub(super) fn filter<P>(
    authored: &AuthoredSource<'_>,
    diagnostics: &mut Vec<AssembledDiagnostic<P>>,
) {
    // Both rules need an authored `@ts-ignore`/`@ts-expect-error`; without one
    // the pass never computes the template range.
    if !authored.is_vue
        || !(authored.text.contains("@ts-ignore") || authored.text.contains("@ts-expect-error"))
        || !diagnostics.iter().any(|diagnostic| {
            is_use_vmodel_passive_false_overload(diagnostic)
                || is_multiline_directive_candidate(authored, diagnostic)
        })
    {
        return;
    }
    let lines: Vec<&str> = authored.text.lines().collect();
    let unused_expect_errors: FxHashSet<usize> = diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.code == Some(UNUSED_TS_EXPECT_ERROR)
                && diagnostic
                    .message
                    .contains("Unused '@ts-expect-error' directive")
        })
        .map(|diagnostic| line_of(authored.text, diagnostic.start))
        .collect();
    diagnostics.retain(|diagnostic| {
        let line = line_of(authored.text, diagnostic.start);
        if is_use_vmodel_passive_false_overload(diagnostic)
            && use_vmodel_passive_false_match(&lines, line)
                .is_some_and(|expect_error_line| !unused_expect_errors.contains(&expect_error_line))
        {
            return false;
        }
        !(is_multiline_directive_candidate(authored, diagnostic)
            && multiline_ts_directive_suppresses(&lines, line, &unused_expect_errors))
    });
}

/// 0-based authored line of `offset`.
fn line_of(text: &str, offset: usize) -> usize {
    text.as_bytes()[..offset.min(text.len())]
        .iter()
        .filter(|&&byte| byte == b'\n')
        .count()
}

fn is_multiline_directive_candidate<P>(
    authored: &AuthoredSource<'_>,
    diagnostic: &AssembledDiagnostic<P>,
) -> bool {
    diagnostic.code != Some(UNUSED_TS_EXPECT_ERROR)
        && !authored
            .template()
            .is_some_and(|template| template.contains(&diagnostic.start))
}

fn is_use_vmodel_passive_false_overload<P>(diagnostic: &AssembledDiagnostic<P>) -> bool {
    diagnostic.code == Some(2769) && diagnostic.message.contains("No overload matches this call")
}

/// The `@ts-expect-error` line guarding a `useVModel(..., { passive: … as
/// false })` call containing `diagnostic_line`.
pub(super) fn use_vmodel_passive_false_match(
    lines: &[&str],
    diagnostic_line: usize,
) -> Option<usize> {
    let (start, end) = containing_use_vmodel_call(lines, diagnostic_line)?;
    if !call_has_passive_false(lines, start, end) {
        return None;
    }
    expect_error_before_default_value(lines, start, end)
}

fn multiline_ts_directive_suppresses(
    lines: &[&str],
    diagnostic_line: usize,
    unused_expect_errors: &FxHashSet<usize>,
) -> bool {
    if diagnostic_line >= lines.len() {
        return false;
    }
    let lower_bound = diagnostic_line.saturating_sub(16);
    for directive_line in (lower_bound..=diagnostic_line).rev() {
        let line = lines[directive_line].trim();
        let expect_error = if line.contains("@ts-ignore") {
            false
        } else if line.contains("@ts-expect-error") {
            true
        } else {
            continue;
        };
        let Some(call_start) = next_non_empty_line(lines, directive_line + 1) else {
            continue;
        };
        let Some(call_end) = call_end_line(lines, call_start) else {
            continue;
        };
        if diagnostic_line >= call_start && diagnostic_line <= call_end {
            return !expect_error || !unused_expect_errors.contains(&directive_line);
        }
    }
    false
}

fn next_non_empty_line(lines: &[&str], start: usize) -> Option<usize> {
    lines
        .iter()
        .enumerate()
        .skip(start)
        .find_map(|(index, line)| (!line.trim().is_empty()).then_some(index))
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
            }
        }
        if saw_open && depth == 0 {
            return Some(index);
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

#[cfg(test)]
#[path = "authored_rules_tests.rs"]
mod tests;
