use super::apply_rule_options;
use vize_patina::{
    HelpLevel, LintDiagnostic, LintResult, Linter, Locale, RuleRegistry,
    rules::vue::{MustacheInterpolationSpacing, ValidVOn},
};

#[test]
fn lsp_custom_modifiers_reach_the_configured_rule() {
    let options = serde_json::from_str(r#"{"vue/valid-v-on":{"modifiers":["custom"]}}"#).unwrap();
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(ValidVOn));
    let linter = apply_rule_options(Linter::with_registry(registry), &options, false, false)
        .with_locale(Locale::En)
        .with_help_level(HelpLevel::Full);
    let result = linter.lint_template(
        r#"<div @click.custom="handler" @click.unknown="handler" />"#,
        "test.vue",
    );
    if let Some(directory) = std::env::var_os("VIZE_NATIVE_STRICT_CAPTURE_DIR") {
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            std::path::Path::new(&directory).join("lsp-custom-modifier.txt"),
            vize_l0::cstr!("{result:#?}").as_bytes(),
        )
        .unwrap();
    }
    let diagnostic = LintDiagnostic::error(
        "vue/valid-v-on",
        "v-on does not support the modifier 'unknown'",
        36,
        43,
    )
    .with_help("Use a built-in event modifier, keyboard alias, or a configured custom modifier.");
    let expected = LintResult {
        filename: "test.vue".into(),
        diagnostics: vec![diagnostic],
        error_count: 1,
        warning_count: 0,
    };
    assert_eq!(
        vize_l0::cstr!("{result:#?}"),
        vize_l0::cstr!("{expected:#?}"),
        "complete LSP-configured modifier diagnostic, including receiver and fix fields"
    );
    let empty = apply_rule_options(
        Linter::with_registry(RuleRegistry::new()),
        &options,
        false,
        false,
    );
    assert!(!empty.registry().has_rule("vue/valid-v-on"));
}

#[test]
fn lsp_custom_macro_order_reaches_the_configured_rule() {
    let options = serde_json::from_str(
        r#"{"script/define-macros-order":{"order":["defineOptions","defineProps","defineEmits"]}}"#,
    )
    .unwrap();
    let linter = apply_rule_options(
        Linter::new().with_enabled_rules(Some(vec!["script/define-macros-order".into()])),
        &options,
        false,
        false,
    );
    let result = linter.lint_sfc(
        r#"<script setup>
defineOptions({ inheritAttrs: false })
const props = defineProps()
const emit = defineEmits()
</script>"#,
        "App.vue",
    );
    assert!(result.diagnostics.is_empty(), "{result:#?}");
}

#[test]
fn lsp_mustache_spacing_reaches_the_configured_rule() {
    let options =
        serde_json::from_str(r#"{"vue/mustache-interpolation-spacing":"never"}"#).unwrap();
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(MustacheInterpolationSpacing::default()));
    let result = apply_rule_options(Linter::with_registry(registry), &options, false, false)
        .lint_template("<div>{{ value }}</div>", "App.vue");
    assert_eq!(result.warning_count, 2, "{result:#?}");
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name == "vue/mustache-interpolation-spacing")
    );
    let empty = apply_rule_options(
        Linter::with_registry(RuleRegistry::new()),
        &options,
        false,
        false,
    );
    assert!(
        !empty
            .registry()
            .has_rule("vue/mustache-interpolation-spacing")
    );
}
