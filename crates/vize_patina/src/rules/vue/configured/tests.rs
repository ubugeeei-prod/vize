use crate::linter::Linter;
use crate::preset::LintPreset;
use vize_l0::config::ConfigLintRuleOptions;

const SHOWCASE: &str = r#"<script setup lang="ts">
import { onMounted } from "vue";

defineOptions({ inheritAttrs: false, name: "RestrictedShowcase" });

const { first = 1, second = 2, third = 3 } = defineProps<{
  first?: number;
  second?: number;
  third?: number;
}>();
const emit = defineEmits<{ legacyEvent: [] }>();
const model = defineModel<string>();

await Promise.resolve();
onMounted(() => {
  emit("legacyEvent");
});
</script>

<template>
  <div class="legacy">
    <marquee>{{ first }}</marquee>
    <section>
      <div>
        <span>{{ second }} {{ model }}</span>
      </div>
    </section>
    <button type="button" data-legacy="x" @legacy-click="$emit('legacyEvent')">{{ third }}</button>
  </div>
</template>

<i18n lang="json">{}</i18n>
"#;

const SETUP_AFTER_AWAIT: &str = r#"<script lang="ts">
import { defineComponent, onMounted } from "vue";

export default defineComponent({
  async setup() {
    await Promise.resolve();
    onMounted(() => {});
  },
});
</script>

<template>
  <p>after await</p>
</template>
"#;

const RULES: &[&str] = &[
    "vue/max-props",
    "vue/max-template-depth",
    "vue/no-restricted-block",
    "vue/no-restricted-call-after-await",
    "vue/no-restricted-class",
    "vue/no-restricted-component-names",
    "vue/no-restricted-component-options",
    "vue/no-restricted-custom-event",
    "vue/no-restricted-html-elements",
    "vue/no-restricted-props",
    "vue/no-restricted-static-attribute",
    "vue/no-restricted-v-on",
    "vue/no-unsupported-features",
];

fn options() -> ConfigLintRuleOptions {
    serde_json::from_value(serde_json::json!({
        "vue/max-props": { "maxProps": 2 },
        "vue/max-template-depth": { "maxDepth": 3 },
        "vue/no-restricted-block": ["i18n"],
        "vue/no-restricted-call-after-await": [{ "module": "vue", "path": "onMounted" }],
        "vue/no-restricted-class": ["legacy"],
        "vue/no-restricted-component-names": ["RestrictedShowcase"],
        "vue/no-restricted-component-options": ["inheritAttrs"],
        "vue/no-restricted-custom-event": ["legacyEvent"],
        "vue/no-restricted-html-elements": ["marquee"],
        "vue/no-restricted-props": ["third"],
        "vue/no-restricted-static-attribute": ["data-legacy"],
        "vue/no-restricted-v-on": ["legacy-click"],
        "vue/no-unsupported-features": { "version": "^3.3.0" }
    }))
    .unwrap()
}

fn reports(source: &str, parsed: &ConfigLintRuleOptions) -> Vec<(u32, u32, String, String)> {
    let result = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(
            RULES.iter().copied().map(vize_l0::String::from).collect(),
        ))
        .with_config_rule_options(parsed)
        .lint_sfc(source, "Fixture.vue");
    let mut reports: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| RULES.contains(&diagnostic.rule_name))
        .map(|diagnostic| {
            let (line, column) = line_col(source, diagnostic.start);
            (
                line,
                column,
                diagnostic.rule_name.to_string(),
                diagnostic.message.to_string(),
            )
        })
        .collect();
    reports.sort();
    reports
}

fn line_col(source: &str, offset: u32) -> (u32, u32) {
    let mut line = 1u32;
    let mut column = 1u32;
    for (index, byte) in source.bytes().enumerate() {
        if index == offset as usize {
            break;
        }
        if byte == b'\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}

#[test]
fn configured_rules_match_the_eslint_plugin_vue_reproduction() {
    let parsed = options();
    assert_eq!(
        reports(SHOWCASE, &parsed),
        vec![
            (
                4,
                17,
                "vue/no-restricted-component-options",
                "Using `inheritAttrs` is not allowed.".into()
            ),
            (
                4,
                44,
                "vue/no-restricted-component-names",
                "Using component name `RestrictedShowcase` is not allowed.".into()
            ),
            (
                6,
                46,
                "vue/max-props",
                "Component has too many props (3). Maximum allowed is 2.".into()
            ),
            (
                9,
                3,
                "vue/no-restricted-props",
                "Using `third` props is not allowed.".into()
            ),
            (
                12,
                15,
                "vue/no-unsupported-features",
                "`defineModel()` macros are not supported until Vue.js \"3.4.0\".".into()
            ),
            (
                21,
                14,
                "vue/no-restricted-class",
                "'legacy' class is not allowed.".into()
            ),
            (
                22,
                5,
                "vue/no-restricted-html-elements",
                "Unexpected use of forbidden element marquee.".into()
            ),
            (
                25,
                9,
                "vue/max-template-depth",
                "Element is nested too deeply (depth of 4, maximum allowed is 3).".into()
            ),
            (
                28,
                27,
                "vue/no-restricted-static-attribute",
                "Using `data-legacy` is not allowed.".into()
            ),
            (
                28,
                43,
                "vue/no-restricted-v-on",
                "Using `@legacy-click` is not allowed on this element.".into()
            ),
            (
                28,
                64,
                "vue/no-restricted-custom-event",
                "Using `legacyEvent` event is not allowed.".into()
            ),
            (
                32,
                1,
                "vue/no-restricted-block",
                "Using `<i18n>` is not allowed.".into()
            ),
        ]
        .into_iter()
        .map(|(line, column, rule, message)| (line, column, rule.to_string(), message))
        .collect::<Vec<_>>()
    );
    assert_eq!(
        reports(SETUP_AFTER_AWAIT, &parsed),
        vec![(
            7,
            5,
            "vue/no-restricted-call-after-await".into(),
            "`import(\"vue\").onMounted` is forbidden after an `await` expression.".into()
        )]
    );
}

#[test]
fn empty_options_and_a_bare_string_stay_within_the_configured_shape() {
    let enabled = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(
            RULES.iter().copied().map(vize_l0::String::from).collect(),
        ))
        .lint_sfc(SHOWCASE, "Fixture.vue");
    assert!(
        enabled
            .diagnostics
            .iter()
            .all(|diagnostic| !RULES.contains(&diagnostic.rule_name))
    );
    let empty: ConfigLintRuleOptions = serde_json::from_value(serde_json::json!({
        "vue/max-props": {},
        "vue/max-template-depth": {},
        "vue/no-restricted-block": [],
        "vue/no-unsupported-features": {}
    }))
    .unwrap();
    assert!(reports(SHOWCASE, &empty).is_empty());

    let bare: ConfigLintRuleOptions = serde_json::from_value(serde_json::json!({
        "vue/no-restricted-block": "i18n",
        "vue/no-restricted-call-after-await": { "module": "vue", "path": "onMounted" }
    }))
    .unwrap();
    let block = reports(SHOWCASE, &bare);
    assert_eq!(block.len(), 1);
    assert_eq!(block[0].0, 32);
    assert_eq!(block[0].2, "vue/no-restricted-block");
    let awaited = reports(SETUP_AFTER_AWAIT, &bare);
    assert_eq!(awaited.len(), 1);
    assert_eq!(awaited[0].2, "vue/no-restricted-call-after-await");
}

#[test]
fn a_supported_version_and_an_ignore_skip_define_model() {
    let supported: ConfigLintRuleOptions = serde_json::from_value(serde_json::json!({
        "vue/no-unsupported-features": { "version": "^3.4.0" }
    }))
    .unwrap();
    assert!(reports(SHOWCASE, &supported).is_empty());
    let ignored: ConfigLintRuleOptions = serde_json::from_value(serde_json::json!({
        "vue/no-unsupported-features": { "version": "^3.3.0", "ignores": ["define-model"] }
    }))
    .unwrap();
    assert!(reports(SHOWCASE, &ignored).is_empty());
}
