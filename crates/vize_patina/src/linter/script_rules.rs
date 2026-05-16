use super::{LintResult, Linter};
use crate::rules::script::{
    NoGetCurrentInstance, NoNextTick, NoOptionsApi, PiniaPreferStoreToRefs, ScriptRule,
    VueRouterPreferNamedPush, VueTestUtilsNoHtmlSnapshot,
};
use vize_atelier_sfc::{parse_sfc, SfcDescriptor, SfcParseOptions};
use vize_carton::profile;

pub(crate) const RULE_NO_OPTIONS_API: &str = "script/no-options-api";
pub(crate) const RULE_NO_GET_CURRENT_INSTANCE: &str = "script/no-get-current-instance";
pub(crate) const RULE_NO_NEXT_TICK: &str = "script/no-next-tick";
pub(crate) const RULE_PINIA_PREFER_STORE_TO_REFS: &str = "ecosystem/pinia-prefer-store-to-refs";
pub(crate) const RULE_VUE_ROUTER_PREFER_NAMED_PUSH: &str = "ecosystem/vue-router-prefer-named-push";
pub(crate) const RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT: &str =
    "ecosystem/vue-test-utils-no-html-snapshot";
const OPINIONATED_SCRIPT_PRESETS: &[&str] = &["opinionated", "nuxt"];
const OPT_IN_SCRIPT_PRESETS: &[&str] = &[];
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
            presets: OPT_IN_SCRIPT_PRESETS,
        },
        BuiltinScriptRuleMeta {
            name: vue_router_prefer_named_push_meta.name,
            description: vue_router_prefer_named_push_meta.description,
            category: "Ecosystem",
            fixable: false,
            default_severity: vue_router_prefer_named_push_meta.default_severity,
            presets: OPT_IN_SCRIPT_PRESETS,
        },
        BuiltinScriptRuleMeta {
            name: vue_test_utils_no_html_snapshot_meta.name,
            description: vue_test_utils_no_html_snapshot_meta.description,
            category: "Ecosystem",
            fixable: false,
            default_severity: vue_test_utils_no_html_snapshot_meta.default_severity,
            presets: OPT_IN_SCRIPT_PRESETS,
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

    append_builtin_script_rule(
        linter,
        descriptor,
        result,
        RULE_NO_OPTIONS_API,
        "patina.script_rule.no_options_api",
        NoOptionsApi,
    );
    append_builtin_script_rule(
        linter,
        descriptor,
        result,
        RULE_NO_GET_CURRENT_INSTANCE,
        "patina.script_rule.no_get_current_instance",
        NoGetCurrentInstance,
    );
    append_builtin_script_rule(
        linter,
        descriptor,
        result,
        RULE_NO_NEXT_TICK,
        "patina.script_rule.no_next_tick",
        NoNextTick,
    );
    append_builtin_script_rule(
        linter,
        descriptor,
        result,
        RULE_PINIA_PREFER_STORE_TO_REFS,
        "patina.script_rule.pinia_prefer_store_to_refs",
        PiniaPreferStoreToRefs,
    );
    append_builtin_script_rule(
        linter,
        descriptor,
        result,
        RULE_VUE_ROUTER_PREFER_NAMED_PUSH,
        "patina.script_rule.vue_router_prefer_named_push",
        VueRouterPreferNamedPush,
    );
    append_builtin_script_rule(
        linter,
        descriptor,
        result,
        RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT,
        "patina.script_rule.vue_test_utils_no_html_snapshot",
        VueTestUtilsNoHtmlSnapshot,
    );
}

fn merge_script_result(
    result: &mut LintResult,
    script_result: crate::rules::script::ScriptLintResult,
) {
    result.error_count += script_result.error_count;
    result.warning_count += script_result.warning_count;
    result.diagnostics.extend(script_result.diagnostics);
}

fn append_builtin_script_rule<'a, R: ScriptRule>(
    linter: &Linter,
    descriptor: &SfcDescriptor<'a>,
    result: &mut LintResult,
    rule_name: &str,
    profile_name: &'static str,
    rule: R,
) {
    if !linter.is_rule_enabled(rule_name) || !linter.script_rules.contains(&rule_name) {
        return;
    }

    if let Some(script) = descriptor.script.as_ref() {
        let mut lint = crate::rules::script::ScriptLintResult::default();
        profile!(
            profile_name,
            rule.check(script.content.as_ref(), script.loc.start, &mut lint)
        );
        merge_script_result(result, lint);
    }
    if let Some(script_setup) = descriptor.script_setup.as_ref() {
        let mut lint = crate::rules::script::ScriptLintResult::default();
        profile!(
            profile_name,
            rule.check(
                script_setup.content.as_ref(),
                script_setup.loc.start,
                &mut lint,
            )
        );
        merge_script_result(result, lint);
    }
}

#[inline]
fn sfc_parse_options(filename: &str) -> SfcParseOptions {
    SfcParseOptions {
        filename: filename.into(),
        ..Default::default()
    }
}
