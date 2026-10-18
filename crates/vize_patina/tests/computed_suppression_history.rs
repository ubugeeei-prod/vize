//! ReKa's authored ESLint alias must suppress only the selected computed write.
use vize_patina::{HelpLevel, LintDiagnostic, LintPreset, LintResult, Linter};

const RULE: &str = "script/no-side-effects-in-computed-properties";
const ESLINT_RULE: &str = "vue/no-side-effects-in-computed-properties";

fn assert_reports(source: &str, filename: &str, writes: &[&str]) {
    let diagnostics = writes
        .iter()
        .map(|write| {
            let start = source.find(*write).unwrap() as u32;
            let end = start + write.len() as u32;
            LintDiagnostic::error(
                RULE,
                "Unexpected side effect in a computed property: an assignment to external state.",
                start,
                end,
            )
            .with_label("side effect inside computed getter", start, end)
            .with_help("A computed getter must be pure: derive and return a value without mutating component state. Move the mutation to a method or a watch handler.")
        })
        .collect();
    let expected = LintResult {
        filename: filename.into(),
        error_count: writes.len(),
        warning_count: 0,
        diagnostics,
    };
    let actual = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_help_level(HelpLevel::Full)
        .lint_sfc(source, filename);
    assert_eq!(format!("{actual:#?}"), format!("{expected:#?}"), "{source}");
}

fn component(body: &str) -> std::string::String {
    format!(
        "<script setup lang=\"ts\">\nimport {{ computed, ref }} from 'vue';\nconst state = ref(0);\nconst derived = computed(() => {{\n{body}\nreturn state.value;\n}});\n</script>\n<template>{{{{ derived }}}}</template>\n"
    )
}

#[test]
fn original_reka_source_respects_its_directive_without_hiding_the_underlying_write() {
    let source = include_str!(
        "../../../tests/_fixtures/differential/linter/computed-suppression-reka/NavigationMenuContentImpl.vue.fixture"
    );
    let filename = "NavigationMenuContentImpl.vue";
    let directive = format!("// eslint-disable-next-line {ESLINT_RULE}");
    assert_eq!(source.matches(&directive).count(), 1);
    assert_reports(source, filename, &[]);
    let unsuppressed = source.replace(&directive, "// suppression removed for the control");
    assert_reports(
        &unsuppressed,
        filename,
        &["prevMotionAttributeRef.value = attribute"],
    );
    assert_reports(source, filename, &[]);
}

#[test]
fn next_line_and_inline_aliases_suppress_only_the_selected_write() {
    for tool in ["eslint", "oxlint"] {
        for rule in [
            ESLINT_RULE,
            RULE,
            "vize/script/no-side-effects-in-computed-properties",
        ] {
            let next_line = component(&format!(
                "// {tool}-disable-next-line {rule} -- intentional control\nstate.value = 1;\nstate.value = 2;"
            ));
            assert_reports(&next_line, "NextLine.vue", &["state.value = 2"]);
            let inline = component(&format!(
                "state.value = 1; // {tool}-disable-line {rule}\nstate.value = 2;"
            ));
            assert_reports(&inline, "Inline.vue", &["state.value = 2"]);
        }
    }
}

#[test]
fn alias_and_canonical_region_pragmas_enable_the_same_rule() {
    for tool in ["eslint", "oxlint"] {
        for (disable, enable) in [(ESLINT_RULE, RULE), (RULE, ESLINT_RULE)] {
            let source = component(&format!(
                "/* {tool}-disable {disable} */\nstate.value = 1;\n/* {tool}-enable {enable} */\nstate.value = 2;"
            ));
            assert_reports(&source, "Region.vue", &["state.value = 2"]);
        }
    }
}

#[test]
fn unrelated_rules_wrong_lines_and_pragma_text_do_not_suppress_computed_writes() {
    for before in [
        "// eslint-disable-next-line vue/no-dupe-keys",
        "// eslint-disable-next-line vue/no-side-effects-in-computed-property",
        "// eslint-disable-next-line vue/no-side-effects-in-computed-properties\n",
        "const note = '// eslint-disable-next-line vue/no-side-effects-in-computed-properties';",
    ] {
        let source = component(&format!("{before}\nstate.value = 1;"));
        assert_reports(&source, "Unsuppressed.vue", &["state.value = 1"]);
    }
}
