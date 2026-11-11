use super::BlockLang;
use crate::linter::Linter;
use crate::preset::LintPreset;
use crate::rule::RuleRegistry;
use vize_l0::config::ConfigLintRuleOptions;

fn configured(source: &str) -> Vec<String> {
    let options: ConfigLintRuleOptions = serde_json::from_value(serde_json::json!({
        "vue/block-lang": { "script": { "lang": "ts", "allowNoLang": false } }
    }))
    .unwrap();
    let result = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec!["vue/block-lang".into()]))
        .with_config_rule_options(&options)
        .lint_sfc(source, "Lang.vue");
    result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "vue/block-lang")
        .map(|diagnostic| format!("{}:{}", diagnostic.start, diagnostic.message))
        .collect()
}

#[test]
fn configured_script_lang_matches_the_issue_messages() {
    let no_lang = "<script setup>\nconst label = \"a\";\n</script>\n";
    let js_lang = "<script setup lang=\"js\">\nconst label = \"a\";\n</script>\n";
    let ts_lang = "<script setup lang=\"ts\">\nconst label = \"a\";\n</script>\n";
    assert_eq!(
        configured(no_lang),
        vec![format!("0:The 'lang' attribute of '<script>' is missing.")]
    );
    let lang_start = js_lang.find("lang").unwrap();
    assert_eq!(
        configured(js_lang),
        vec![format!(
            "{lang_start}:Only 'ts' can be used for the 'lang' attribute of '<script>'."
        )]
    );
    assert!(configured(ts_lang).is_empty(), "{ts_lang}");
}

#[test]
fn the_default_policy_reports_explicit_languages_only() {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(BlockLang::default()));
    let linter = Linter::with_registry(registry);
    let missing = linter.lint_sfc("<script setup>\nconst label = \"a\";\n</script>\n", "A.vue");
    assert!(missing.diagnostics.is_empty(), "{:?}", missing.diagnostics);
    let javascript = linter.lint_sfc("<script lang=\"js\"></script>\n", "A.vue");
    assert_eq!(
        javascript.diagnostics[0].message.as_str(),
        "Do not explicitly specify the default language for the 'lang' attribute of '<script>'."
    );
    let typescript = linter.lint_sfc("<script lang=\"ts\"></script>\n", "A.vue");
    assert_eq!(
        typescript.diagnostics[0].message.as_str(),
        "Do not specify the 'lang' attribute of '<script>'."
    );
}

#[test]
fn an_empty_options_object_checks_nothing_and_required_reports_the_block() {
    let empty: ConfigLintRuleOptions =
        serde_json::from_value(serde_json::json!({ "vue/block-lang": {} })).unwrap();
    let silent = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec!["vue/block-lang".into()]))
        .with_config_rule_options(&empty)
        .lint_sfc("<script lang=\"js\"></script>\n", "A.vue");
    assert!(
        silent
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != "vue/block-lang")
    );
    let required: ConfigLintRuleOptions = serde_json::from_value(serde_json::json!({
        "vue/block-lang": { "script": { "required": true } }
    }))
    .unwrap();
    let result = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec!["vue/block-lang".into()]))
        .with_config_rule_options(&required)
        .lint_sfc("<template><p /></template>\n", "A.vue");
    let messages: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "vue/block-lang")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert_eq!(messages, ["The '<script>' block is missing."]);
}

#[test]
fn several_allowed_languages_keep_config_order() {
    let options: ConfigLintRuleOptions = serde_json::from_value(serde_json::json!({
        "vue/block-lang": { "script": { "lang": ["ts", "tsx"], "allowNoLang": false } }
    }))
    .unwrap();
    let result = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec!["vue/block-lang".into()]))
        .with_config_rule_options(&options)
        .lint_sfc("<script lang=\"js\"></script>\n", "A.vue");
    assert_eq!(
        result.diagnostics[0].message.as_str(),
        "Only 'ts', and 'tsx' can be used for the 'lang' attribute of '<script>'."
    );
}
