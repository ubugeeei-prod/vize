use super::Linter;
use crate::LintPreset;

#[test]
fn test_plugin_prefixed_and_oxlint_comments_suppress_only_the_named_script_rule() {
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = |comment: &str| {
        format!(
            "<script setup lang=\"ts\">\nconst props = defineProps<{{ items: string[] }}>()\n{comment}\nObject.assign(props.items, [])\n</script>"
        )
    };
    let mutation_count = |comment: &str| {
        linter
            .lint_sfc(&source(comment), "App.vue")
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == "vue/no-mutating-props")
            .count()
    };

    assert_eq!(mutation_count("// unrelated comment"), 1);
    assert_eq!(
        mutation_count("// eslint-disable-next-line vize/vue/no-mutating-props"),
        0
    );
    assert_eq!(
        mutation_count("// oxlint-disable-next-line vize/vue/no-mutating-props"),
        0
    );
    assert_eq!(
        mutation_count("// oxlint-disable-next-line vue/no-mutating-props"),
        0
    );
    assert_eq!(
        mutation_count("// oxlint-disable-next-line vize/vue/no-unused-vars"),
        1,
        "a different prefixed rule must not suppress no-mutating-props"
    );
}

#[test]
fn test_plugin_prefixed_region_enable_restores_script_rule() {
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const props = defineProps<{ items: string[] }>()
// oxlint-disable vize/vue/no-mutating-props
Object.assign(props.items, [])
// oxlint-enable vize/vue/no-mutating-props
Object.assign(props.items, [])
</script>"#;
    assert_eq!(
        linter
            .lint_sfc(source, "App.vue")
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == "vue/no-mutating-props")
            .count(),
        1
    );
}

#[test]
fn test_plugin_prefixed_comments_suppress_template_rule() {
    let linter = Linter::new();
    for marker in ["eslint", "oxlint"] {
        let source = format!(
            "<!-- {marker}-disable-next-line vize/vue/require-v-for-key -->\n<ul><li v-for=\"item in items\">{{{{ item }}}}</li></ul>"
        );
        let result = linter.lint_template(&source, "App.vue");
        assert!(
            result
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.rule_name != "vue/require-v-for-key"),
            "{marker}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_lint_markers_in_template_text_and_attributes_are_not_comments() {
    let linter = Linter::new();
    for marker in ["eslint-disable", "oxlint-disable"] {
        let source = format!(
            "<p>{marker}</p>\n<div data-note=\"{marker}\"></div>\n<ul><li v-for=\"item in items\">{{{{ item }}}}</li></ul>"
        );
        let result = linter.lint_template(&source, "App.vue");
        assert_eq!(
            result
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.rule_name == "vue/require-v-for-key")
                .count(),
            1,
            "{marker} in ordinary markup must not suppress the rule"
        );
    }
}

#[test]
fn test_named_enable_after_disable_all_restores_only_named_template_rule() {
    let linter = Linter::new();
    let source = r#"<!-- oxlint-disable -->
<ul><li v-for="item in items">{{ item }}</li></ul>
<!-- oxlint-enable vize/vue/require-v-for-key -->
<ul><li v-for="item in items">{{ item }}</li></ul>
<a :href="url">link</a>"#;
    let result = linter.lint_template(source, "App.vue");
    assert_eq!(
        result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == "vue/require-v-for-key")
            .count(),
        1,
        "named enable must restore its rule after disable-all"
    );
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != "vue/no-unsafe-url"),
        "other rules must remain disabled"
    );
}

#[test]
fn test_named_enable_after_disable_all_restores_only_named_script_rule() {
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const props = defineProps<{ items: string[] }>()
// oxlint-disable
Object.assign(props.items, [])
// oxlint-enable vize/vue/no-mutating-props
Object.assign(props.items, [])
// oxlint-disable vize/vue/no-mutating-props
Object.assign(props.items, [])
</script>"#;
    let result = linter.lint_sfc(source, "App.vue");
    assert_eq!(
        result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == "vue/no-mutating-props")
            .count(),
        1,
        "only the mutation after named enable should report"
    );
}
