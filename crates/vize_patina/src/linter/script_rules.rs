use super::{LintResult, Linter};
use crate::rules::script::{
    NoGetCurrentInstance, NoNextTick, NoOptionsApi, PiniaPreferStoreToRefs, ScriptRule,
    VueRouterPreferNamedPush, VueTestUtilsNoHtmlSnapshot, script_source_type,
};
use memchr::memmem;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use vize_atelier_sfc::{SfcDescriptor, SfcParseOptions, parse_sfc};
use vize_carton::profile;

/// A built-in script rule paired with its registry name and profiling label.
///
/// All built-in script rules are AST-based, so a single oxc parse per script
/// block can be shared across every enabled rule.
struct BuiltinScriptRuleEntry {
    rule_name: &'static str,
    profile_name: &'static str,
    rule: &'static (dyn ScriptRule + 'static),
}

/// The full ordered set of built-in script rules.
///
/// Order matches the previous sequence of `append_builtin_script_rule` calls so
/// emitted diagnostics keep the same ordering.
static BUILTIN_SCRIPT_RULES: &[BuiltinScriptRuleEntry] = &[
    BuiltinScriptRuleEntry {
        rule_name: RULE_NO_OPTIONS_API,
        profile_name: "patina.script_rule.no_options_api",
        rule: &NoOptionsApi,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_NO_GET_CURRENT_INSTANCE,
        profile_name: "patina.script_rule.no_get_current_instance",
        rule: &NoGetCurrentInstance,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_NO_NEXT_TICK,
        profile_name: "patina.script_rule.no_next_tick",
        rule: &NoNextTick,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_PINIA_PREFER_STORE_TO_REFS,
        profile_name: "patina.script_rule.pinia_prefer_store_to_refs",
        rule: &PiniaPreferStoreToRefs,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_VUE_ROUTER_PREFER_NAMED_PUSH,
        profile_name: "patina.script_rule.vue_router_prefer_named_push",
        rule: &VueRouterPreferNamedPush,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT,
        profile_name: "patina.script_rule.vue_test_utils_no_html_snapshot",
        rule: &VueTestUtilsNoHtmlSnapshot,
    },
];

pub(crate) const RULE_NO_OPTIONS_API: &str = "script/no-options-api";
pub(crate) const RULE_NO_GET_CURRENT_INSTANCE: &str = "script/no-get-current-instance";
pub(crate) const RULE_NO_NEXT_TICK: &str = "script/no-next-tick";
pub(crate) const RULE_PINIA_PREFER_STORE_TO_REFS: &str = "ecosystem/pinia-prefer-store-to-refs";
pub(crate) const RULE_VUE_ROUTER_PREFER_NAMED_PUSH: &str = "ecosystem/vue-router-prefer-named-push";
pub(crate) const RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT: &str =
    "ecosystem/vue-test-utils-no-html-snapshot";
const OPINIONATED_SCRIPT_PRESETS: &[&str] = &["opinionated", "nuxt"];
const ECOSYSTEM_SCRIPT_PRESETS: &[&str] = &["ecosystem"];
const ALL_BUILTIN_SCRIPT_RULE_NAMES: &[&str] = &[
    RULE_NO_OPTIONS_API,
    RULE_NO_GET_CURRENT_INSTANCE,
    RULE_NO_NEXT_TICK,
    RULE_PINIA_PREFER_STORE_TO_REFS,
    RULE_VUE_ROUTER_PREFER_NAMED_PUSH,
    RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT,
];
#[cfg(test)]
const OPT_IN_SCRIPT_RULE_NAMES: &[&str] = &[
    RULE_PINIA_PREFER_STORE_TO_REFS,
    RULE_VUE_ROUTER_PREFER_NAMED_PUSH,
    RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT,
];

pub struct BuiltinScriptRuleMeta {
    pub name: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub fixable: bool,
    pub default_severity: crate::Severity,
    pub presets: &'static [&'static str],
}

pub fn builtin_script_rules() -> [BuiltinScriptRuleMeta; 6] {
    let no_options_api = NoOptionsApi;
    let no_options_api_meta = no_options_api.meta();
    let no_get_current_instance = NoGetCurrentInstance;
    let no_get_current_instance_meta = no_get_current_instance.meta();
    let no_next_tick = NoNextTick;
    let no_next_tick_meta = no_next_tick.meta();
    let pinia_prefer_store_to_refs = PiniaPreferStoreToRefs;
    let pinia_prefer_store_to_refs_meta = pinia_prefer_store_to_refs.meta();
    let vue_router_prefer_named_push = VueRouterPreferNamedPush;
    let vue_router_prefer_named_push_meta = vue_router_prefer_named_push.meta();
    let vue_test_utils_no_html_snapshot = VueTestUtilsNoHtmlSnapshot;
    let vue_test_utils_no_html_snapshot_meta = vue_test_utils_no_html_snapshot.meta();

    [
        BuiltinScriptRuleMeta {
            name: no_options_api_meta.name,
            description: no_options_api_meta.description,
            category: "Vapor",
            fixable: false,
            default_severity: no_options_api_meta.default_severity,
            presets: OPINIONATED_SCRIPT_PRESETS,
        },
        BuiltinScriptRuleMeta {
            name: no_get_current_instance_meta.name,
            description: no_get_current_instance_meta.description,
            category: "Vapor",
            fixable: false,
            default_severity: no_get_current_instance_meta.default_severity,
            presets: OPINIONATED_SCRIPT_PRESETS,
        },
        BuiltinScriptRuleMeta {
            name: no_next_tick_meta.name,
            description: no_next_tick_meta.description,
            category: "Vapor",
            fixable: false,
            default_severity: no_next_tick_meta.default_severity,
            presets: OPINIONATED_SCRIPT_PRESETS,
        },
        BuiltinScriptRuleMeta {
            name: pinia_prefer_store_to_refs_meta.name,
            description: pinia_prefer_store_to_refs_meta.description,
            category: "Ecosystem",
            fixable: false,
            default_severity: pinia_prefer_store_to_refs_meta.default_severity,
            presets: ECOSYSTEM_SCRIPT_PRESETS,
        },
        BuiltinScriptRuleMeta {
            name: vue_router_prefer_named_push_meta.name,
            description: vue_router_prefer_named_push_meta.description,
            category: "Ecosystem",
            fixable: false,
            default_severity: vue_router_prefer_named_push_meta.default_severity,
            presets: ECOSYSTEM_SCRIPT_PRESETS,
        },
        BuiltinScriptRuleMeta {
            name: vue_test_utils_no_html_snapshot_meta.name,
            description: vue_test_utils_no_html_snapshot_meta.description,
            category: "Ecosystem",
            fixable: false,
            default_severity: vue_test_utils_no_html_snapshot_meta.default_severity,
            presets: ECOSYSTEM_SCRIPT_PRESETS,
        },
    ]
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
    linter
        .script_rules
        .iter()
        .copied()
        .any(|rule_name| linter.is_rule_enabled(rule_name))
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
    if linter.script_rules.is_empty() {
        return;
    }
    if linter
        .script_rules
        .iter()
        .copied()
        .all(is_ecosystem_script_rule)
        && !descriptor_scripts_may_match_ecosystem_rule(descriptor)
    {
        return;
    }

    // Parse each block at most once and reuse the program across every rule.
    // A block is only parsed if at least one enabled rule could match it (the
    // same `is_rule_enabled` + `script_rules.contains` + `script_rule_may_match`
    // gate the previous per-rule path applied before parsing). Diagnostics are
    // emitted rule-major / block-minor to preserve the exact ordering of the
    // previous per-rule call sequence (which iterated `script` then
    // `script_setup` inside each rule).
    let script_alloc = Allocator::default();
    let script = descriptor
        .script
        .as_ref()
        .filter(|block| block_has_active_rule(linter, block.content.as_ref()))
        .map(|block| {
            let source = block.content.as_ref();
            let parsed = profile!(
                "patina.script_rule.parse",
                Parser::new(&script_alloc, source, script_source_type()).parse()
            );
            (source, block.loc.start, parsed)
        });

    let setup_alloc = Allocator::default();
    let script_setup = descriptor
        .script_setup
        .as_ref()
        .filter(|block| block_has_active_rule(linter, block.content.as_ref()))
        .map(|block| {
            let source = block.content.as_ref();
            let parsed = profile!(
                "patina.script_rule.parse",
                Parser::new(&setup_alloc, source, script_source_type()).parse()
            );
            (source, block.loc.start, parsed)
        });

    for entry in BUILTIN_SCRIPT_RULES {
        if !linter.is_rule_enabled(entry.rule_name)
            || !linter.script_rules.contains(&entry.rule_name)
        {
            continue;
        }
        if let Some((source, offset, parsed)) = script.as_ref() {
            run_builtin_script_rule_on_parsed(entry, source, *offset, parsed, result);
        }
        if let Some((source, offset, parsed)) = script_setup.as_ref() {
            run_builtin_script_rule_on_parsed(entry, source, *offset, parsed, result);
        }
    }
}

/// Whether any enabled built-in script rule could match `source`.
///
/// Mirrors the per-rule `is_rule_enabled` + `script_rules.contains` +
/// `script_rule_may_match` gate so a block matching no rule is never parsed.
fn block_has_active_rule(linter: &Linter, source: &str) -> bool {
    BUILTIN_SCRIPT_RULES.iter().any(|entry| {
        linter.is_rule_enabled(entry.rule_name)
            && linter.script_rules.contains(&entry.rule_name)
            && script_rule_may_match(entry.rule_name, source)
    })
}

/// Run a single built-in script rule against a pre-parsed script block.
///
/// Applies the same `script_rule_may_match` byte prefilter and parse-failure
/// guard the per-rule `check` used to apply, then dispatches to `check_program`
/// with the shared program.
fn run_builtin_script_rule_on_parsed(
    entry: &BuiltinScriptRuleEntry,
    source: &str,
    offset: usize,
    parsed: &oxc_parser::ParserReturn<'_>,
    result: &mut LintResult,
) {
    if parsed.panicked || !parsed.errors.is_empty() {
        return;
    }
    if !script_rule_may_match(entry.rule_name, source) {
        return;
    }
    let mut lint = crate::rules::script::ScriptLintResult::default();
    profile!(
        entry.profile_name,
        entry
            .rule
            .check_program(&parsed.program, source, offset, &mut lint)
    );
    merge_script_result(result, lint);
}

pub(crate) fn append_builtin_script_diagnostics_from_html(
    linter: &Linter,
    source: &str,
    result: &mut LintResult,
) {
    if linter.script_rules.is_empty() {
        return;
    }

    for (script, offset) in extract_inline_scripts(source) {
        append_builtin_script_rules_for_source(linter, script, offset, result);
    }
}

fn merge_script_result(
    result: &mut LintResult,
    script_result: crate::rules::script::ScriptLintResult,
) {
    result.error_count += script_result.error_count;
    result.warning_count += script_result.warning_count;
    result.diagnostics.extend(script_result.diagnostics);
}

/// Run every enabled built-in script rule against a single script block,
/// sharing **one** oxc parse across all of them.
///
/// Mirrors the previous per-rule flow exactly: each rule is gated on
/// `is_rule_enabled` + `script_rules.contains` and on its `script_rule_may_match`
/// byte prefilter, runs into its own [`ScriptLintResult`], and is merged in the
/// original rule order. The only change is that the oxc parse happens once here
/// instead of once inside every rule's `check`.
fn append_builtin_script_rules_for_source(
    linter: &Linter,
    source: &str,
    offset: usize,
    result: &mut LintResult,
) {
    // Skip parsing entirely when no enabled rule could match this block.
    if !block_has_active_rule(linter, source) {
        return;
    }

    // Parse the script block exactly once and share the program with each rule.
    let allocator = Allocator::default();
    let parsed = profile!(
        "patina.script_rule.parse",
        Parser::new(&allocator, source, script_source_type()).parse()
    );

    for entry in BUILTIN_SCRIPT_RULES {
        if !linter.is_rule_enabled(entry.rule_name)
            || !linter.script_rules.contains(&entry.rule_name)
        {
            continue;
        }
        run_builtin_script_rule_on_parsed(entry, source, offset, &parsed, result);
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
