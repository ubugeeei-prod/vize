use super::{is_nuxt_config_filename, script_rule_may_match};
use crate::linter::script_rules::registry::{
    RULE_NO_ARROW_FUNCTIONS_IN_WATCH, RULE_NO_IMPORT_COMPILER_MACROS, RULE_VALID_DEFINE_PROPS,
};

#[test]
fn recognizes_supported_nuxt_config_paths() {
    for extension in [
        "js", "jsx", "ts", "tsx", "cjs", "cjsx", "cts", "ctsx", "mjs", "mjsx", "mts", "mtsx",
    ] {
        let root = format!("apps/web/nuxt.config.{extension}");
        let hidden = format!("apps/web/.config/nuxt.{extension}");
        assert!(is_nuxt_config_filename(&root), "{root}");
        assert!(is_nuxt_config_filename(&hidden), "{hidden}");
    }

    assert!(is_nuxt_config_filename(r"apps\web\nuxt.config.js"));
    assert!(is_nuxt_config_filename(r"apps\web\.config\nuxt.mts"));
}

#[test]
fn rejects_unrelated_or_lookalike_paths() {
    for filename in [
        "vitest.config.ts",
        "vize.config.ts",
        "components/DataTable.vue",
        "nuxt.config.d.ts",
        "nuxt.config.json",
        "config/nuxt.ts",
        ".config/not-nuxt.ts",
        "my-nuxt.config.ts",
    ] {
        assert!(!is_nuxt_config_filename(filename), "{filename}");
    }
}

#[test]
fn skips_common_script_setup_imports_without_macro_imports() {
    assert!(!script_rule_may_match(
        RULE_NO_IMPORT_COMPILER_MACROS,
        "import { ref, computed } from 'vue'"
    ));
    assert!(script_rule_may_match(
        RULE_NO_IMPORT_COMPILER_MACROS,
        "import { defineProps } from 'vue'"
    ));
    assert!(script_rule_may_match(
        RULE_NO_IMPORT_COMPILER_MACROS,
        "import { defineProps } from\n'vue'"
    ));
    assert!(script_rule_may_match(
        RULE_NO_IMPORT_COMPILER_MACROS,
        "import { defineProps } from /* compiler macros */ 'vue'"
    ));
}

#[test]
fn keeps_export_default_with_trivia_for_component_options_rules() {
    assert!(script_rule_may_match(
        RULE_NO_ARROW_FUNCTIONS_IN_WATCH,
        "export /* component */ default { watch: { value: () => {} } }"
    ));
}

#[test]
fn setup_dupe_candidates_do_not_widen_other_options_rule_prefilters() {
    let source = "const props = defineProps<{ foo: string }>(); const foo = 1;";
    assert!(script_rule_may_match(super::RULE_NO_DUPE_KEYS, source));
    assert!(!script_rule_may_match(super::RULE_NO_RESERVED_KEYS, source));
    assert!(!script_rule_may_match(
        super::RULE_NO_POTENTIAL_COMPONENT_OPTION_TYPO,
        source
    ));
    assert!(!script_rule_may_match(
        super::RULE_NO_DUPE_KEYS,
        "const foo = 1;"
    ));
}

#[test]
fn skips_define_props_rule_without_define_props_call() {
    assert!(!script_rule_may_match(
        RULE_VALID_DEFINE_PROPS,
        "import { ref } from 'vue'\nconst count = ref(0)"
    ));
    assert!(script_rule_may_match(
        RULE_VALID_DEFINE_PROPS,
        "defineProps<{ value: string }>()"
    ));
}
