//! Vue 3 removal checks that every built-in bundle enables by default.

use crate::{HelpLevel, LintPreset, Linter};
use vize_l0::config::VueVersion;

const TEMPLATE_RULES: [&str; 4] = [
    "vue/no-deprecated-filter",
    "vue/no-deprecated-html-element-is",
    "vue/no-deprecated-inline-template",
    "vue/no-deprecated-router-link-tag-prop",
];

const SCRIPT_RULES: [&str; 10] = [
    "script/no-deprecated-data-object-declaration",
    "script/no-deprecated-destroyed-lifecycle",
    "script/no-deprecated-dollar-listeners-api",
    "script/no-deprecated-dollar-scopedslots-api",
    "script/no-deprecated-events-api",
    "script/no-deprecated-props-default-this",
    "script/no-deprecated-delete-set",
    "script/no-deprecated-model-definition",
    "script/no-deprecated-vue-config-keycodes",
    "script/require-slots-as-functions",
];

const REMOVED: &str = r#"<script>
export default {
  model: { prop: 'value', event: 'input' },
  data: { ready: false },
  props: {
    label: { type: String, default() { return this.fallback } },
  },
  beforeDestroy() {
    this.$off('ready')
    this.$set(this, 'ready', true)
    Vue.config.keyCodes.enter = 13
  },
  methods: {
    forward() {
      return [this.$listeners, this.$scopedSlots, this.$slots.default]
    },
  },
}
</script>

<template>
  <section>
    <p>{{ label | capitalize }}</p>
    <div is="LegacyPanel" />
    <legacy-panel inline-template><span>inline</span></legacy-panel>
    <router-link to="/" tag="li">Home</router-link>
  </section>
</template>
"#;

const CURRENT: &str = r#"<script>
export default {
  data() {
    return { ready: false }
  },
  props: {
    label: { type: String, default: 'label' },
  },
  beforeUnmount() {
    this.stop()
  },
  methods: {
    forward() {
      return [this.$attrs, this.$slots]
    },
  },
}
</script>

<template>
  <section>
    <p>{{ capitalize(label) || fallback }}</p>
    <div is="vue:LegacyPanel" />
    <component :is="panel" />
    <router-link to="/">Home</router-link>
  </section>
</template>
"#;

fn reported(linter: &Linter, source: &str) -> Vec<&'static str> {
    let result = linter.lint_sfc(source, "Removed.vue");
    TEMPLATE_RULES
        .into_iter()
        .chain(SCRIPT_RULES)
        .filter(|rule| {
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_name == *rule)
        })
        .collect()
}

fn all_rules() -> Vec<&'static str> {
    TEMPLATE_RULES.into_iter().chain(SCRIPT_RULES).collect()
}

#[test]
fn every_bundle_reports_removed_vue_3_syntax_and_apis() {
    for preset in LintPreset::ALL {
        let linter = Linter::with_preset(preset).with_help_level(HelpLevel::None);
        let expected = if preset == LintPreset::Incremental {
            Vec::new()
        } else {
            all_rules()
        };
        assert_eq!(reported(&linter, REMOVED), expected, "{}", preset.as_str());
        assert_eq!(
            reported(&linter, CURRENT),
            Vec::<&str>::new(),
            "{}",
            preset.as_str()
        );
    }
}

#[test]
fn removal_checks_follow_the_configured_vue_version() {
    for preset in LintPreset::ALL {
        if preset == LintPreset::Incremental {
            continue;
        }
        for version in [VueVersion::V2, VueVersion::V2_7, VueVersion::V3] {
            let linter = Linter::with_preset(preset)
                .with_help_level(HelpLevel::None)
                .with_vue_version(Some(version));
            let expected = if version.is_legacy() {
                Vec::new()
            } else {
                all_rules()
            };
            assert_eq!(
                reported(&linter, REMOVED),
                expected,
                "{} / {version:?}",
                preset.as_str()
            );
        }
    }
}

#[test]
fn removal_checks_can_be_disabled_one_by_one() {
    for rule in all_rules() {
        let linter = Linter::with_preset(LintPreset::Essential)
            .with_help_level(HelpLevel::None)
            .with_disabled_rules(vec![rule.into()]);
        let expected: Vec<_> = all_rules()
            .into_iter()
            .filter(|name| *name != rule)
            .collect();
        assert_eq!(reported(&linter, REMOVED), expected, "{rule}");
    }
}

#[test]
fn plain_modules_are_not_components() {
    for (source, filename) in [
        (
            "const config = { data: { items: [] } };\nexport default config;\n",
            "site.config.ts",
        ),
        (
            "export default class Lazy {\n  stop() { this.$off('load') }\n}\n",
            "lazy.js",
        ),
    ] {
        for preset in LintPreset::ALL {
            let result = Linter::with_preset(preset).lint_script(source, filename);
            assert!(
                result
                    .diagnostics
                    .iter()
                    .all(|diagnostic| !SCRIPT_RULES.contains(&diagnostic.rule_name)),
                "{filename} / {}",
                preset.as_str()
            );
        }
    }
}

#[test]
fn define_component_modules_are_components() {
    let source = "import { defineComponent } from 'vue'\nexport default defineComponent({ data: { ready: false } })\n";
    let result = Linter::with_preset(LintPreset::Essential).lint_script(source, "Panel.ts");
    assert!(
        result.diagnostics.iter().any(
            |diagnostic| diagnostic.rule_name == "script/no-deprecated-data-object-declaration"
        )
    );
}
