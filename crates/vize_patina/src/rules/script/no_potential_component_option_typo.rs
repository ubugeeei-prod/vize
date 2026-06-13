//! script/no-potential-component-option-typo
//!
//! Flag likely typos in Options API component option names. A top-level key that
//! is *not* a known Vue option but is within Levenshtein edit-distance 1 of one
//! (e.g. `method` → `methods`, `prop` → `props`, `computes` → `computed`) is
//! almost certainly a typo: Vue silently ignores the misspelled option, so it
//! never takes effect. Keys far from every known option (distance ≥ 2) are left
//! alone — they may be intentional custom options consumed by a plugin or mixin.
//!
//! The options object is resolved exactly as in [`crate::rules::script::NoDupeKeys`]
//! (`export default {...}` / `defineComponent({...})` / identifier-bound), via
//! [`vize_croquis::script_parser::collect_options_descriptor`].

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use vize_carton::CompactString;
use vize_croquis::script_parser::collect_options_descriptor;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-potential-component-option-typo",
    description: "Flag likely typos in Options API component option names",
    default_severity: Severity::Error,
};

/// Known valid Vue component option names (Options API), including the standard
/// lifecycle hooks. A top-level key matching one of these is never flagged.
const KNOWN_OPTIONS: &[&str] = &[
    // Asset / composition options.
    "name",
    "components",
    "directives",
    "mixins",
    "extends",
    "provide",
    "inject",
    // State / behavior / rendering options.
    "props",
    "emits",
    "data",
    "computed",
    "methods",
    "watch",
    "expose",
    "setup",
    "render",
    "template",
    // Less-common-but-valid options that sit close to others (avoids false positives).
    "inheritAttrs",
    "model",
    "delimiters",
    "compilerOptions",
    "__name",
    // Lifecycle hooks.
    "beforeCreate",
    "created",
    "beforeMount",
    "mounted",
    "beforeUpdate",
    "updated",
    "beforeUnmount",
    "unmounted",
    "beforeDestroy",
    "destroyed",
    "activated",
    "deactivated",
    "errorCaptured",
    "renderTracked",
    "renderTriggered",
    "serverPrefetch",
];

/// Flag likely typos in component option names.
pub struct NoPotentialComponentOptionTypo;

impl ScriptRule for NoPotentialComponentOptionTypo {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    #[inline]
    fn check_program<'a>(
        &self,
        program: &'a oxc_ast::ast::Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        let Some(descriptor) = collect_options_descriptor(program) else {
            return;
        };

        for key in &descriptor.option_keys {
            let name = key.name.as_str();
            // Exact known options are fine; keys far from any known option
            // (distance ≥ 2) are assumed to be intentional custom options.
            if is_known_option(name) {
                continue;
            }
            let Some(suggestion) = closest_known_option(name) else {
                continue;
            };

            let start = offset as u32 + key.start;
            let end = offset as u32 + key.end;
            result.add_diagnostic(build_diagnostic(name, suggestion, start, end));
        }
    }
}

/// Whether `name` is an exact known Vue component option.
fn is_known_option(name: &str) -> bool {
    KNOWN_OPTIONS.contains(&name)
}

/// The known option within edit-distance 1 of `name`, if any. Returns `None`
/// when every known option is at distance ≥ 2. On a distance-1 tie the first in
/// declaration order wins, keeping the suggestion deterministic.
fn closest_known_option(name: &str) -> Option<&'static str> {
    for &option in KNOWN_OPTIONS {
        // Exact matches are filtered by the caller; never suggest a name for itself.
        if option == name {
            return None;
        }
        if within_edit_distance_one(name, option) {
            return Some(option);
        }
    }
    None
}

/// Whether `a` and `b` are within Levenshtein edit-distance 1 (one insertion,
/// deletion, or substitution). Implemented directly so the common
/// length-differs-by-more-than-one case exits immediately.
fn within_edit_distance_one(a: &str, b: &str) -> bool {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let (a_len, b_len) = (a_bytes.len(), b_bytes.len());

    // Identical strings have distance 0; treat as "within 1".
    if a == b {
        return true;
    }

    match a_len.abs_diff(b_len) {
        // Equal length -> exactly one substitution allowed.
        0 => {
            let mut diffs = 0usize;
            for (x, y) in a_bytes.iter().zip(b_bytes.iter()) {
                if x != y {
                    diffs += 1;
                    if diffs > 1 {
                        return false;
                    }
                }
            }
            diffs == 1
        }
        // Length differs by one -> exactly one insertion/deletion allowed; the
        // shorter must be a subsequence of the longer with a single skip.
        1 => {
            let (shorter, longer) = if a_len < b_len {
                (a_bytes, b_bytes)
            } else {
                (b_bytes, a_bytes)
            };
            let (mut i, mut j) = (0usize, 0usize);
            let mut skipped = false;
            while i < shorter.len() && j < longer.len() {
                if shorter[i] == longer[j] {
                    i += 1;
                    j += 1;
                } else if skipped {
                    return false;
                } else {
                    // Skip one character in the longer string.
                    skipped = true;
                    j += 1;
                }
            }
            true
        }
        // Length differs by two or more -> distance is at least 2.
        _ => false,
    }
}

/// Build the typo diagnostic for `name`, suggesting `suggestion`.
fn build_diagnostic(name: &str, suggestion: &str, start: u32, end: u32) -> LintDiagnostic {
    let mut message = CompactString::with_capacity(name.len() + suggestion.len() + 48);
    message.push('\'');
    message.push_str(name);
    message.push_str("' is not a known component option; did you mean '");
    message.push_str(suggestion);
    message.push_str("'?");

    let mut help = CompactString::with_capacity(suggestion.len() + 40);
    help.push_str("Rename this option to '");
    help.push_str(suggestion);
    help.push_str("'.");

    LintDiagnostic::error(META.name, message, start, end)
        .with_label("likely typo", start, end)
        .with_help(help)
}

#[cfg(test)]
mod tests;
