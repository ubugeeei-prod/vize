use super::Linter;
use crate::{LintPreset, Severity};

const ART_SOURCE: &str = r#"<art component="./Button.vue">
  <variant name="empty"></variant>
</art>
"#;

#[test]
fn default_presets_do_not_lint_art_files() {
    let result = Linter::new().lint_sfc(ART_SOURCE, "Button.art.vue");

    assert!(!result.has_diagnostics());
}

#[test]
fn explicitly_enabled_musea_rules_lint_art_files() {
    let linter = Linter::with_preset(LintPreset::Incremental).with_enabled_rules(Some(vec![
        "musea/require-title".into(),
        "musea/no-empty-variant".into(),
    ]));

    let result = linter.lint_sfc(ART_SOURCE, "Button.art.vue");
    let rules = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.rule_name)
        .collect::<Vec<_>>();

    assert_eq!(rules, ["musea/require-title", "musea/no-empty-variant"]);
    assert_eq!(result.error_count, 1);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn musea_rules_stay_scoped_to_art_vue_files() {
    let linter = Linter::with_preset(LintPreset::Incremental).with_enabled_rules(Some(vec![
        "musea/require-title".into(),
        "musea/no-empty-variant".into(),
    ]));

    let result = linter.lint_sfc(ART_SOURCE, "Button.vue");

    assert!(!result.has_diagnostics());
}

#[test]
fn musea_rule_severity_overrides_recount_results() {
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec!["musea/require-title".into()]))
        .with_rule_severity_overrides(vec![("musea/require-title".into(), Severity::Warning)]);

    let result = linter.lint_sfc(ART_SOURCE, "Button.art.vue");

    assert_eq!(result.error_count, 0);
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.diagnostics[0].severity, Severity::Warning);
}

#[test]
fn art_variant_markup_runs_template_rules() {
    let linter = Linter::new().with_enabled_rules(Some(vec![
        "a11y/click-events-have-key-events".into(),
        "vue/html-self-closing".into(),
    ]));
    let art = r#"<script setup lang="ts">
import MyButton from "./MyButton.vue";

function onClick() {}
</script>

<art>
  <variant name="Default">
    <div class="row" @click="onClick">
      <img src="/logo.png">
      <MyButton></MyButton>
    </div>
  </variant>
</art>"#;
    let plain = r#"<script setup lang="ts">
import MyButton from "./MyButton.vue";

function onClick() {}
</script>

<template>
  <div class="row" @click="onClick">
    <img src="/logo.png">
    <MyButton></MyButton>
  </div>
</template>"#;

    let art_result = linter.lint_sfc(art, "MyButton.art.vue");
    let plain_result = linter.lint_sfc(plain, "PlainCard.vue");
    let mut art_rules = rule_names(&art_result);
    let mut plain_rules = rule_names(&plain_result);
    art_rules.sort_unstable();
    plain_rules.sort_unstable();
    assert_eq!(art_rules, plain_rules, "art={art_rules:?}");
    assert!(art_rules.contains(&"a11y/click-events-have-key-events"));

    let mut registry = crate::rule::RuleRegistry::new();
    registry.register(Box::new(crate::rules::vue::HtmlSelfClosing::default()));
    let style = Linter::with_registry(registry);
    let art_style_result = style.lint_sfc(art, "MyButton.art.vue");
    let plain_style_result = style.lint_sfc(plain, "PlainCard.vue");
    let art_style = rule_names(&art_style_result);
    let plain_style = rule_names(&plain_style_result);
    assert_eq!(
        art_style, plain_style,
        "art={art_style:?} plain={plain_style:?}"
    );
    assert!(
        plain_style.contains(&"vue/html-self-closing"),
        "plain={plain_style:?}"
    );

    let click = art.find("@click").expect("click");
    assert!(
        art_result.diagnostics.iter().any(|diagnostic| {
            diagnostic.rule_name == "a11y/click-events-have-key-events"
                && (diagnostic.start as usize) <= click
                && click < diagnostic.end as usize
        }),
        "{:?}",
        art_result.diagnostics
    );

    let outside = linter.lint_sfc(art, "MyButton.vue");
    assert!(outside.diagnostics.is_empty(), "{:?}", outside.diagnostics);
}

fn rule_names(result: &crate::LintResult) -> Vec<&str> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.rule_name)
        .collect()
}

#[test]
fn musea_category_can_disable_enabled_musea_rules() {
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec!["musea/require-title".into()]))
        .with_disabled_categories(vec!["musea".into()]);

    let result = linter.lint_sfc(ART_SOURCE, "Button.art.vue");

    assert!(!result.has_diagnostics());
}
