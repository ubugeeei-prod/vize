use super::{LintResult, Linter};
use crate::rules::script::{
    NoAsyncInComputed, NoDeepDestructureInProps, NoGetCurrentInstance, NoImportCompilerMacros,
    NoInternalImports, NoNextTick, NoOptionsApi, NoReactiveDestructure, NoReservedIdentifiers,
    NoTopLevelRefInScript, NoWithDefaults, PiniaPreferStoreToRefs, PreferComputed,
    PreferImportFromVue, PreferRefOverReactive, PreferUseAttrs, PreferUseId, PreferUseSlots,
    PreferUseTemplateRef, RequireFunctionReturnType, RequireSymbolProvide, ScriptRule,
    VueRouterPreferNamedPush, VueTestUtilsNoHtmlSnapshot, script_source_type,
};
use memchr::memmem;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use vize_atelier_sfc::{SfcDescriptor, SfcParseOptions, parse_sfc};
use vize_carton::profile;

/// A built-in script rule paired with its registry name and profiling label.
///
/// AST-based rules share a single oxc parse per script block. Byte-based rules
/// run directly against the source via [`ScriptRule::check`].
struct BuiltinScriptRuleEntry {
    rule_name: &'static str,
    profile_name: &'static str,
    category: &'static str,
    fixable: bool,
    presets: &'static [&'static str],
    rule: &'static (dyn ScriptRule + 'static),
}

impl BuiltinScriptRuleEntry {
    #[inline]
    fn meta(&self) -> BuiltinScriptRuleMeta {
        let meta = self.rule.meta();
        BuiltinScriptRuleMeta {
            name: meta.name,
            description: meta.description,
            category: self.category,
            fixable: self.fixable,
            default_severity: meta.default_severity,
            presets: self.presets,
        }
    }
}

const BUILTIN_SCRIPT_RULE_COUNT: usize = 23;
static NO_DEEP_DESTRUCTURE_IN_PROPS_RULE: NoDeepDestructureInProps =
    NoDeepDestructureInProps { max_depth: 1 };

/// The full ordered set of built-in script rules.
///
/// The original 6 engine-reachable rules stay first so existing default
/// diagnostic ordering is preserved. The remaining script rules follow as
/// opt-in built-ins and are reachable through explicit rule selection.
static BUILTIN_SCRIPT_RULES: [BuiltinScriptRuleEntry; BUILTIN_SCRIPT_RULE_COUNT] = [
    BuiltinScriptRuleEntry {
        rule_name: RULE_NO_OPTIONS_API,
        profile_name: "patina.script_rule.no_options_api",
        category: "Vapor",
        fixable: false,
        presets: OPINIONATED_SCRIPT_PRESETS,
        rule: &NoOptionsApi,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_NO_GET_CURRENT_INSTANCE,
        profile_name: "patina.script_rule.no_get_current_instance",
        category: "Vapor",
        fixable: false,
        presets: OPINIONATED_SCRIPT_PRESETS,
        rule: &NoGetCurrentInstance,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_NO_NEXT_TICK,
        profile_name: "patina.script_rule.no_next_tick",
        category: "Vapor",
        fixable: false,
        presets: OPINIONATED_SCRIPT_PRESETS,
        rule: &NoNextTick,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_PINIA_PREFER_STORE_TO_REFS,
        profile_name: "patina.script_rule.pinia_prefer_store_to_refs",
        category: "Ecosystem",
        fixable: false,
        presets: ECOSYSTEM_SCRIPT_PRESETS,
        rule: &PiniaPreferStoreToRefs,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_VUE_ROUTER_PREFER_NAMED_PUSH,
        profile_name: "patina.script_rule.vue_router_prefer_named_push",
        category: "Ecosystem",
        fixable: false,
        presets: ECOSYSTEM_SCRIPT_PRESETS,
        rule: &VueRouterPreferNamedPush,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT,
        profile_name: "patina.script_rule.vue_test_utils_no_html_snapshot",
        category: "Ecosystem",
        fixable: false,
        presets: ECOSYSTEM_SCRIPT_PRESETS,
        rule: &VueTestUtilsNoHtmlSnapshot,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_PREFER_COMPUTED,
        profile_name: "patina.script_rule.prefer_computed",
        category: "Script",
        fixable: false,
        presets: OPT_IN_SCRIPT_PRESETS,
        rule: &PreferComputed,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_NO_ASYNC_IN_COMPUTED,
        profile_name: "patina.script_rule.no_async_in_computed",
        category: "Script",
        fixable: false,
        presets: OPT_IN_SCRIPT_PRESETS,
        rule: &NoAsyncInComputed,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_NO_REACTIVE_DESTRUCTURE,
        profile_name: "patina.script_rule.no_reactive_destructure",
        category: "Script",
        fixable: false,
        presets: OPT_IN_SCRIPT_PRESETS,
        rule: &NoReactiveDestructure,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_NO_TOP_LEVEL_REF_IN_SCRIPT,
        profile_name: "patina.script_rule.no_top_level_ref_in_script",
        category: "Script",
        fixable: false,
        presets: OPT_IN_SCRIPT_PRESETS,
        rule: &NoTopLevelRefInScript,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_PREFER_REF_OVER_REACTIVE,
        profile_name: "patina.script_rule.prefer_ref_over_reactive",
        category: "Script",
        fixable: false,
        presets: OPT_IN_SCRIPT_PRESETS,
        rule: &PreferRefOverReactive,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_PREFER_USE_TEMPLATE_REF,
        profile_name: "patina.script_rule.prefer_use_template_ref",
        category: "Script",
        fixable: false,
        presets: OPT_IN_SCRIPT_PRESETS,
        rule: &PreferUseTemplateRef,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_PREFER_USE_SLOTS,
        profile_name: "patina.script_rule.prefer_use_slots",
        category: "Script",
        fixable: false,
        presets: OPT_IN_SCRIPT_PRESETS,
        rule: &PreferUseSlots,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_PREFER_USE_ATTRS,
        profile_name: "patina.script_rule.prefer_use_attrs",
        category: "Script",
        fixable: false,
        presets: OPT_IN_SCRIPT_PRESETS,
        rule: &PreferUseAttrs,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_PREFER_USE_ID,
        profile_name: "patina.script_rule.prefer_use_id",
        category: "Script",
        fixable: false,
        presets: OPT_IN_SCRIPT_PRESETS,
        rule: &PreferUseId,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_PREFER_IMPORT_FROM_VUE,
        profile_name: "patina.script_rule.prefer_import_from_vue",
        category: "Script",
        fixable: true,
        presets: OPT_IN_SCRIPT_PRESETS,
        rule: &PreferImportFromVue,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_NO_WITH_DEFAULTS,
        profile_name: "patina.script_rule.no_with_defaults",
        category: "Script",
        fixable: false,
        presets: OPT_IN_SCRIPT_PRESETS,
        rule: &NoWithDefaults,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_NO_DEEP_DESTRUCTURE_IN_PROPS,
        profile_name: "patina.script_rule.no_deep_destructure_in_props",
        category: "Script",
        fixable: false,
        presets: OPT_IN_SCRIPT_PRESETS,
        rule: &NO_DEEP_DESTRUCTURE_IN_PROPS_RULE,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_NO_INTERNAL_IMPORTS,
        profile_name: "patina.script_rule.no_internal_imports",
        category: "Script",
        fixable: false,
        presets: OPT_IN_SCRIPT_PRESETS,
        rule: &NoInternalImports,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_NO_IMPORT_COMPILER_MACROS,
        profile_name: "patina.script_rule.no_import_compiler_macros",
        category: "Script",
        fixable: false,
        presets: OPT_IN_SCRIPT_PRESETS,
        rule: &NoImportCompilerMacros,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_NO_RESERVED_IDENTIFIERS,
        profile_name: "patina.script_rule.no_reserved_identifiers",
        category: "Script",
        fixable: false,
        presets: OPT_IN_SCRIPT_PRESETS,
        rule: &NoReservedIdentifiers,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_REQUIRE_SYMBOL_PROVIDE,
        profile_name: "patina.script_rule.require_symbol_provide",
        category: "Script",
        fixable: false,
        presets: OPT_IN_SCRIPT_PRESETS,
        rule: &RequireSymbolProvide,
    },
    BuiltinScriptRuleEntry {
        rule_name: RULE_REQUIRE_FUNCTION_RETURN_TYPE,
        profile_name: "patina.script_rule.require_function_return_type",
        category: "Script",
        fixable: false,
        presets: OPT_IN_SCRIPT_PRESETS,
        rule: &RequireFunctionReturnType,
    },
];

pub(crate) const RULE_NO_OPTIONS_API: &str = "script/no-options-api";
pub(crate) const RULE_NO_GET_CURRENT_INSTANCE: &str = "script/no-get-current-instance";
pub(crate) const RULE_NO_NEXT_TICK: &str = "script/no-next-tick";
pub(crate) const RULE_PINIA_PREFER_STORE_TO_REFS: &str = "ecosystem/pinia-prefer-store-to-refs";
pub(crate) const RULE_VUE_ROUTER_PREFER_NAMED_PUSH: &str = "ecosystem/vue-router-prefer-named-push";
pub(crate) const RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT: &str =
    "ecosystem/vue-test-utils-no-html-snapshot";
pub(crate) const RULE_PREFER_COMPUTED: &str = "script/prefer-computed";
pub(crate) const RULE_NO_ASYNC_IN_COMPUTED: &str = "script/no-async-in-computed";
pub(crate) const RULE_NO_REACTIVE_DESTRUCTURE: &str = "script/no-reactive-destructure";
pub(crate) const RULE_NO_TOP_LEVEL_REF_IN_SCRIPT: &str = "script/no-top-level-ref-in-script";
pub(crate) const RULE_PREFER_REF_OVER_REACTIVE: &str = "script/prefer-ref-over-reactive";
pub(crate) const RULE_PREFER_USE_TEMPLATE_REF: &str = "script/prefer-use-template-ref";
pub(crate) const RULE_PREFER_USE_SLOTS: &str = "script/prefer-use-slots";
pub(crate) const RULE_PREFER_USE_ATTRS: &str = "script/prefer-use-attrs";
pub(crate) const RULE_PREFER_USE_ID: &str = "script/prefer-use-id";
pub(crate) const RULE_PREFER_IMPORT_FROM_VUE: &str = "script/prefer-import-from-vue";
pub(crate) const RULE_NO_WITH_DEFAULTS: &str = "script/no-with-defaults";
pub(crate) const RULE_NO_DEEP_DESTRUCTURE_IN_PROPS: &str = "script/no-deep-destructure-in-props";
pub(crate) const RULE_NO_INTERNAL_IMPORTS: &str = "script/no-internal-imports";
pub(crate) const RULE_NO_IMPORT_COMPILER_MACROS: &str = "script/no-import-compiler-macros";
pub(crate) const RULE_NO_RESERVED_IDENTIFIERS: &str = "script/no-reserved-identifiers";
pub(crate) const RULE_REQUIRE_SYMBOL_PROVIDE: &str = "script/require-symbol-provide";
pub(crate) const RULE_REQUIRE_FUNCTION_RETURN_TYPE: &str = "script/require-function-return-type";
const OPINIONATED_SCRIPT_PRESETS: &[&str] = &["opinionated", "nuxt"];
const ECOSYSTEM_SCRIPT_PRESETS: &[&str] = &["ecosystem"];
const OPT_IN_SCRIPT_PRESETS: &[&str] = &[];
const ALL_BUILTIN_SCRIPT_RULE_NAMES: &[&str] = &[
    RULE_NO_OPTIONS_API,
    RULE_NO_GET_CURRENT_INSTANCE,
    RULE_NO_NEXT_TICK,
    RULE_PINIA_PREFER_STORE_TO_REFS,
    RULE_VUE_ROUTER_PREFER_NAMED_PUSH,
    RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT,
    RULE_PREFER_COMPUTED,
    RULE_NO_ASYNC_IN_COMPUTED,
    RULE_NO_REACTIVE_DESTRUCTURE,
    RULE_NO_TOP_LEVEL_REF_IN_SCRIPT,
    RULE_PREFER_REF_OVER_REACTIVE,
    RULE_PREFER_USE_TEMPLATE_REF,
    RULE_PREFER_USE_SLOTS,
    RULE_PREFER_USE_ATTRS,
    RULE_PREFER_USE_ID,
    RULE_PREFER_IMPORT_FROM_VUE,
    RULE_NO_WITH_DEFAULTS,
    RULE_NO_DEEP_DESTRUCTURE_IN_PROPS,
    RULE_NO_INTERNAL_IMPORTS,
    RULE_NO_IMPORT_COMPILER_MACROS,
    RULE_NO_RESERVED_IDENTIFIERS,
    RULE_REQUIRE_SYMBOL_PROVIDE,
    RULE_REQUIRE_FUNCTION_RETURN_TYPE,
];
#[cfg(test)]
const OPT_IN_SCRIPT_RULE_NAMES: &[&str] = &[
    RULE_PINIA_PREFER_STORE_TO_REFS,
    RULE_VUE_ROUTER_PREFER_NAMED_PUSH,
    RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT,
    RULE_PREFER_COMPUTED,
    RULE_NO_ASYNC_IN_COMPUTED,
    RULE_NO_REACTIVE_DESTRUCTURE,
    RULE_NO_TOP_LEVEL_REF_IN_SCRIPT,
    RULE_PREFER_REF_OVER_REACTIVE,
    RULE_PREFER_USE_TEMPLATE_REF,
    RULE_PREFER_USE_SLOTS,
    RULE_PREFER_USE_ATTRS,
    RULE_PREFER_USE_ID,
    RULE_PREFER_IMPORT_FROM_VUE,
    RULE_NO_WITH_DEFAULTS,
    RULE_NO_DEEP_DESTRUCTURE_IN_PROPS,
    RULE_NO_INTERNAL_IMPORTS,
    RULE_NO_IMPORT_COMPILER_MACROS,
    RULE_NO_RESERVED_IDENTIFIERS,
    RULE_REQUIRE_SYMBOL_PROVIDE,
    RULE_REQUIRE_FUNCTION_RETURN_TYPE,
];

pub struct BuiltinScriptRuleMeta {
    pub name: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub fixable: bool,
    pub default_severity: crate::Severity,
    pub presets: &'static [&'static str],
}

pub fn builtin_script_rules() -> [BuiltinScriptRuleMeta; BUILTIN_SCRIPT_RULE_COUNT] {
    std::array::from_fn(|index| BUILTIN_SCRIPT_RULES[index].meta())
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
    let index = match rule_name {
        RULE_NO_OPTIONS_API => 0,
        RULE_NO_GET_CURRENT_INSTANCE => 1,
        RULE_NO_NEXT_TICK => 2,
        RULE_PINIA_PREFER_STORE_TO_REFS => 3,
        RULE_VUE_ROUTER_PREFER_NAMED_PUSH => 4,
        RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT => 5,
        RULE_PREFER_COMPUTED => 6,
        RULE_NO_ASYNC_IN_COMPUTED => 7,
        RULE_NO_REACTIVE_DESTRUCTURE => 8,
        RULE_NO_TOP_LEVEL_REF_IN_SCRIPT => 9,
        RULE_PREFER_REF_OVER_REACTIVE => 10,
        RULE_PREFER_USE_TEMPLATE_REF => 11,
        RULE_PREFER_USE_SLOTS => 12,
        RULE_PREFER_USE_ATTRS => 13,
        RULE_PREFER_USE_ID => 14,
        RULE_PREFER_IMPORT_FROM_VUE => 15,
        RULE_NO_WITH_DEFAULTS => 16,
        RULE_NO_DEEP_DESTRUCTURE_IN_PROPS => 17,
        RULE_NO_INTERNAL_IMPORTS => 18,
        RULE_NO_IMPORT_COMPILER_MACROS => 19,
        RULE_NO_RESERVED_IDENTIFIERS => 20,
        RULE_REQUIRE_SYMBOL_PROVIDE => 21,
        RULE_REQUIRE_FUNCTION_RETURN_TYPE => 22,
        _ => return None,
    };
    Some(&BUILTIN_SCRIPT_RULES[index])
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
        if let Some((source, offset)) = script_setup {
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
    let mut lint = crate::rules::script::ScriptLintResult::default();
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

fn merge_script_result(
    result: &mut LintResult,
    script_result: crate::rules::script::ScriptLintResult,
) {
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
