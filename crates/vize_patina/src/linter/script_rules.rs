use super::{LintResult, Linter};
use crate::rules::script::{ScriptLintResult, script_source_type};
use memchr::memmem;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use vize_atelier_sfc::{SfcDescriptor, SfcParseOptions, parse_sfc};
use vize_carton::profile;

mod registry;

pub use registry::BuiltinScriptRuleMeta;
use registry::{
    ALL_BUILTIN_SCRIPT_RULE_NAMES, BUILTIN_SCRIPT_RULES, BuiltinScriptRuleEntry,
    RULE_PINIA_PREFER_STORE_TO_REFS, RULE_PREFER_COMPUTED, RULE_VUE_ROUTER_PREFER_NAMED_PUSH,
    RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT,
};

#[cfg(test)]
use registry::OPT_IN_SCRIPT_RULE_NAMES;

pub fn builtin_script_rules() -> Vec<BuiltinScriptRuleMeta> {
    BUILTIN_SCRIPT_RULES
        .iter()
        .map(|entry| entry.meta())
        .collect()
}

#[inline]
pub(crate) const fn all_builtin_script_rule_names() -> &'static [&'static str] {
    ALL_BUILTIN_SCRIPT_RULE_NAMES
}

#[cfg(test)]
#[inline]
pub(crate) const fn opt_in_script_rule_names() -> &'static [&'static str] {
    OPT_IN_SCRIPT_RULE_NAMES
}

#[inline]
pub(crate) fn has_active_builtin_script_rules(linter: &Linter) -> bool {
    active_builtin_script_rule_entries(linter).next().is_some()
}

fn active_builtin_script_rule_entries(
    linter: &Linter,
) -> impl Iterator<Item = &'static BuiltinScriptRuleEntry> + '_ {
    linter
        .script_rules
        .iter()
        .copied()
        .filter(|rule_name| linter.is_rule_enabled(rule_name))
        .filter_map(builtin_script_rule_entry)
}

fn builtin_script_rule_entry(rule_name: &str) -> Option<&'static BuiltinScriptRuleEntry> {
    BUILTIN_SCRIPT_RULES
        .iter()
        .find(|entry| entry.rule_name == rule_name)
}

#[inline]
pub(crate) fn parse_sfc_for_lint<'a>(
    source: &'a str,
    filename: &str,
) -> Result<SfcDescriptor<'a>, vize_atelier_sfc::SfcError> {
    profile!(
        "patina.sfc.parse_for_lint",
        parse_sfc(source, sfc_parse_options(filename))
    )
}

pub(crate) fn lint_with_descriptor<'a>(
    linter: &Linter,
    filename: &str,
    descriptor: &SfcDescriptor<'a>,
) -> LintResult {
    let mut result = profile!(
        "patina.sfc.descriptor.template_lint",
        linter.lint_sfc_template_with_descriptor(filename, descriptor)
    );

    append_builtin_script_diagnostics(linter, descriptor, &mut result);
    result
}

pub(crate) fn append_builtin_script_diagnostics<'a>(
    linter: &Linter,
    descriptor: &SfcDescriptor<'a>,
    result: &mut LintResult,
) {
    if linter.script_rules.is_empty() || !has_active_builtin_script_rules(linter) {
        return;
    }
    if has_only_active_ecosystem_script_rules(linter)
        && !descriptor_scripts_may_match_ecosystem_rule(descriptor)
    {
        return;
    }

    // Parse each block at most once and only when an active AST rule could
    // match it. Byte rules run directly against the source. Diagnostics are
    // emitted rule-major / block-minor to preserve the previous ordering.
    let script = descriptor
        .script
        .as_ref()
        .map(|block| (block.content.as_ref(), block.loc.start))
        .filter(|(source, _)| block_has_active_rule(linter, source));
    let script_alloc = Allocator::default();
    let script_parsed = script
        .filter(|(source, _)| block_has_active_ast_rule(linter, source))
        .map(|(source, _)| {
            let parsed = profile!(
                "patina.script_rule.parse",
                Parser::new(&script_alloc, source, script_source_type()).parse()
            );
            parsed
        });

    let script_setup = descriptor
        .script_setup
        .as_ref()
        .map(|block| (block.content.as_ref(), block.loc.start))
        .filter(|(source, _)| block_has_active_rule(linter, source));
    let setup_alloc = Allocator::default();
    let script_setup_parsed = script_setup
        .filter(|(source, _)| block_has_active_ast_rule(linter, source))
        .map(|(source, _)| {
            let parsed = profile!(
                "patina.script_rule.parse",
                Parser::new(&setup_alloc, source, script_source_type()).parse()
            );
            parsed
        });

    for entry in active_builtin_script_rule_entries(linter) {
        if let Some((source, offset)) = script {
            run_builtin_script_rule(entry, source, offset, script_parsed.as_ref(), result);
        }
        if let Some((source, offset)) = script_setup.filter(|_| entry.rule.runs_on_script_setup()) {
            run_builtin_script_rule(entry, source, offset, script_setup_parsed.as_ref(), result);
        }
    }
}

/// Whether any enabled built-in script rule could match `source`.
///
/// Mirrors the per-rule `is_rule_enabled` + `script_rules.contains` +
/// `script_rule_may_match` gate so a block matching no rule is never parsed.
fn block_has_active_rule(linter: &Linter, source: &str) -> bool {
    active_builtin_script_rule_entries(linter)
        .any(|entry| script_rule_may_match(entry.rule_name, source))
}

/// Whether any enabled AST-based built-in script rule could match `source`.
fn block_has_active_ast_rule(linter: &Linter, source: &str) -> bool {
    active_builtin_script_rule_entries(linter)
        .any(|entry| entry.rule.uses_ast() && script_rule_may_match(entry.rule_name, source))
}

/// Run a single built-in script rule against a script block.
///
/// AST rules consume the shared parse when available. Byte rules run their
/// source-level `check`, preserving the same rule-major ordering.
fn run_builtin_script_rule(
    entry: &BuiltinScriptRuleEntry,
    source: &str,
    offset: usize,
    parsed: Option<&oxc_parser::ParserReturn<'_>>,
    result: &mut LintResult,
) {
    if !script_rule_may_match(entry.rule_name, source) {
        return;
    }
    let mut lint = ScriptLintResult::default();
    if entry.rule.uses_ast() {
        let Some(parsed) = parsed else {
            return;
        };
        if parsed.panicked || !parsed.errors.is_empty() {
            return;
        }
        profile!(
            entry.profile_name,
            entry
                .rule
                .check_program(&parsed.program, source, offset, &mut lint)
        );
    } else {
        profile!(
            entry.profile_name,
            entry.rule.check(source, offset, &mut lint)
        );
    }
    merge_script_result(result, lint);
}

pub(crate) fn append_builtin_script_diagnostics_from_html(
    linter: &Linter,
    source: &str,
    result: &mut LintResult,
) {
    if linter.script_rules.is_empty() || !has_active_builtin_script_rules(linter) {
        return;
    }

    for (script, offset) in extract_inline_scripts(source) {
        append_builtin_script_rules_for_source(linter, script, offset, result);
    }
}

fn merge_script_result(result: &mut LintResult, script_result: ScriptLintResult) {
    result.error_count += script_result.error_count;
    result.warning_count += script_result.warning_count;
    result.diagnostics.extend(script_result.diagnostics);
}

/// Run every enabled built-in script rule against a single script block.
///
/// Mirrors the previous per-rule flow exactly: each rule is gated on
/// `is_rule_enabled` + `script_rules.contains` and on its `script_rule_may_match`
/// byte prefilter, runs into its own [`ScriptLintResult`], and is merged in the
/// original rule order. Active AST rules share one oxc parse; byte rules run
/// directly against the source.
fn append_builtin_script_rules_for_source(
    linter: &Linter,
    source: &str,
    offset: usize,
    result: &mut LintResult,
) {
    // Skip work entirely when no enabled rule could match this block.
    if !block_has_active_rule(linter, source) {
        return;
    }

    let allocator = Allocator::default();
    let parsed = block_has_active_ast_rule(linter, source).then(|| {
        profile!(
            "patina.script_rule.parse",
            Parser::new(&allocator, source, script_source_type()).parse()
        )
    });

    for entry in active_builtin_script_rule_entries(linter) {
        run_builtin_script_rule(entry, source, offset, parsed.as_ref(), result);
    }
}

fn script_rule_may_match(rule_name: &str, source: &str) -> bool {
    let bytes = source.as_bytes();
    match rule_name {
        RULE_PINIA_PREFER_STORE_TO_REFS => memmem::find(bytes, b"Store").is_some(),
        RULE_VUE_ROUTER_PREFER_NAMED_PUSH => {
            (memmem::find(bytes, b".push").is_some() || memmem::find(bytes, b".replace").is_some())
                && (memmem::find(bytes, b"'/").is_some() || memmem::find(bytes, b"\"/").is_some())
                && (memmem::find(bytes, b"router").is_some()
                    || memmem::find(bytes, b"Router").is_some())
        }
        RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT => {
            memmem::find(bytes, b"toMatchSnapshot").is_some()
                && memmem::find(bytes, b".html").is_some()
        }
        // `watch` also appears in any aliased import (`watch as observe`), so
        // this never skips a block the AST check could flag.
        RULE_PREFER_COMPUTED => memmem::find(bytes, b"watch").is_some(),
        _ => true,
    }
}

fn descriptor_scripts_may_match_ecosystem_rule(descriptor: &SfcDescriptor<'_>) -> bool {
    descriptor
        .script
        .as_ref()
        .is_some_and(|script| source_may_match_ecosystem_rule(script.content.as_ref()))
        || descriptor
            .script_setup
            .as_ref()
            .is_some_and(|script| source_may_match_ecosystem_rule(script.content.as_ref()))
}

fn is_ecosystem_script_rule(rule_name: &str) -> bool {
    matches!(
        rule_name,
        RULE_PINIA_PREFER_STORE_TO_REFS
            | RULE_VUE_ROUTER_PREFER_NAMED_PUSH
            | RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT
    )
}

fn has_only_active_ecosystem_script_rules(linter: &Linter) -> bool {
    active_builtin_script_rule_entries(linter)
        .all(|entry| is_ecosystem_script_rule(entry.rule_name))
}

fn source_may_match_ecosystem_rule(source: &str) -> bool {
    let bytes = source.as_bytes();
    memmem::find(bytes, b"Store").is_some()
        || ((memmem::find(bytes, b".push").is_some() || memmem::find(bytes, b".replace").is_some())
            && (memmem::find(bytes, b"'/").is_some() || memmem::find(bytes, b"\"/").is_some())
            && (memmem::find(bytes, b"router").is_some()
                || memmem::find(bytes, b"Router").is_some()))
        || (memmem::find(bytes, b"toMatchSnapshot").is_some()
            && memmem::find(bytes, b".html").is_some())
}

#[inline]
fn sfc_parse_options(filename: &str) -> SfcParseOptions {
    SfcParseOptions {
        filename: filename.into(),
        ..Default::default()
    }
}

fn extract_inline_scripts(source: &str) -> Vec<(&str, usize)> {
    let mut scripts = Vec::new();
    let mut cursor = 0;

    while let Some(open_start) = find_script_open(source, cursor) {
        let Some(open_end) = find_tag_end(source, open_start) else {
            break;
        };

        let content_start = open_end + 1;
        let Some(close_start) = find_ascii_case_insensitive(source, "</script", content_start)
        else {
            break;
        };

        let content = &source[content_start..close_start];
        if !content.trim().is_empty() {
            scripts.push((content, content_start));
        }

        cursor = find_tag_end(source, close_start).map_or(close_start + 9, |end| end + 1);
    }

    scripts
}

fn find_script_open(source: &str, from: usize) -> Option<usize> {
    let mut cursor = from;
    while let Some(index) = find_ascii_case_insensitive(source, "<script", cursor) {
        let boundary = source.as_bytes().get(index + 7).copied();
        if matches!(
            boundary,
            None | Some(b'>' | b'/' | b' ' | b'\n' | b'\r' | b'\t' | b'\x0c')
        ) {
            return Some(index);
        }
        cursor = index + 7;
    }
    None
}

fn find_tag_end(source: &str, from: usize) -> Option<usize> {
    let mut quote = None;
    for (relative, byte) in source.as_bytes()[from..].iter().copied().enumerate() {
        match (quote, byte) {
            (Some(current), value) if value == current => quote = None,
            (None, b'"' | b'\'') => quote = Some(byte),
            (None, b'>') => return Some(from + relative),
            _ => {}
        }
    }
    None
}

fn find_ascii_case_insensitive(source: &str, needle: &str, from: usize) -> Option<usize> {
    let haystack = source.as_bytes();
    let needle = needle.as_bytes();
    if needle.is_empty() || from >= haystack.len() {
        return None;
    }

    haystack[from..]
        .windows(needle.len())
        .position(|window| window.eq_ignore_ascii_case(needle))
        .map(|index| from + index)
}

#[cfg(test)]
mod standalone_html_tests {
    use super::extract_inline_scripts;

    #[test]
    fn extracts_inline_scripts_from_standalone_html() {
        let source = r##"<!doctype html>
<html>
<head>
  <script src="https://unpkg.com/vue@3/dist/vue.global.js"></script>
</head>
<body>
  <script>
Vue.createApp({ data() { return { count: 0 } } }).mount("#app")
  </script>
</body>
</html>"##;

        let scripts = extract_inline_scripts(source);
        assert_eq!(scripts.len(), 1);
        assert!(scripts[0].0.contains("Vue.createApp"));
        assert_eq!(&source[scripts[0].1..scripts[0].1 + 3], "\nVu");
    }
}
