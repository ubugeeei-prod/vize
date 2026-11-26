use super::NoUndefinedRefs;
use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleCategory, RuleRegistry};

const RULE: &str = "vue/no-undefined-refs";

/// Lint a full SFC with only this rule enabled.
///
/// `vue/no-undefined-refs` is a member of `SEMANTIC_TEMPLATE_RULES`, so
/// enabling it alone still produces the croquis analysis the rule reads.
fn lint_sfc(sfc: &str) -> LintResult {
    Linter::new()
        .with_enabled_rules(Some(vec![RULE.into()]))
        .lint_sfc(sfc, "Probe.vue")
}

fn findings(result: &LintResult) -> Vec<(&'static str, Severity, u32, u32, &str)> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.rule_name,
                diagnostic.severity,
                diagnostic.start,
                diagnostic.end,
                diagnostic.message.as_str(),
            )
        })
        .collect()
}

fn none() -> Vec<(&'static str, Severity, u32, u32, &'static str)> {
    Vec::new()
}

fn undefined_at(sfc: &str, name: &str) -> (&'static str, Severity, u32, u32, std::string::String) {
    let start = sfc.rfind(name).expect("undefined identifier") as u32;
    (
        RULE,
        Severity::Warning,
        start,
        start + name.len() as u32,
        format!("Variable '{name}' is not defined"),
    )
}

#[test]
fn test_meta() {
    let rule = NoUndefinedRefs::default();
    assert_eq!(rule.meta().name, RULE);
    assert_eq!(rule.meta().category, RuleCategory::Recommended);
    assert_eq!(rule.meta().default_severity, Severity::Warning);
}

#[test]
fn default_presets_and_opt_in_registry_own_the_rule() {
    assert!(RuleRegistry::with_opt_in_rules().has_rule(RULE));
    assert!(RuleRegistry::with_preset(crate::preset::LintPreset::HappyPath).has_rule(RULE));
    assert!(RuleRegistry::with_preset(crate::preset::LintPreset::Ecosystem).has_rule(RULE));
    assert!(RuleRegistry::with_preset(crate::preset::LintPreset::Opinionated).has_rule(RULE));
}

#[test]
fn default_preset_reports_an_undefined_template_ref() {
    let sfc = r#"<script setup>
const known = 1
</script>
<template>
  <p>{{ known }} {{ missing }}</p>
</template>
"#;
    let result = Linter::new().lint_sfc(sfc, "Probe.vue");
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == RULE)
    );
}

#[test]
fn reports_an_undefined_interpolation_identifier() {
    let sfc = r#"<script setup>
const known = 1
</script>
<template>
  <p>{{ known }} {{ missing }}</p>
</template>
"#;
    let expected = undefined_at(sfc, "missing");
    assert_eq!(
        findings(&lint_sfc(sfc))
            .into_iter()
            .map(|(rule, severity, start, end, message)| {
                (rule, severity, start, end, message.to_string())
            })
            .collect::<Vec<_>>(),
        vec![expected]
    );
}

#[test]
fn additional_rules_can_enable_the_rule_without_replacing_the_preset() {
    let sfc = r#"<script setup>
const known = 1
</script>
<template>
  <p>{{ known }} {{ missing }}</p>
</template>
"#;
    let result = Linter::new()
        .with_additional_rules(vec![RULE.into()])
        .lint_sfc(sfc, "Probe.vue");
    let expected = undefined_at(sfc, "missing");
    assert_eq!(
        findings(&result)
            .into_iter()
            .filter(|(rule, _, _, _, _)| *rule == RULE)
            .map(|(rule, severity, start, end, message)| {
                (rule, severity, start, end, message.to_string())
            })
            .collect::<Vec<_>>(),
        vec![expected]
    );
}

#[test]
fn ignores_a_script_setup_binding() {
    let sfc = r#"<script setup>
const known = 1
</script>
<template>
  <p>{{ known }}</p>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_v_for_alias() {
    let sfc = r#"<script setup>
const items = ['a']
</script>
<template>
  <span v-for="item in items" :key="item">{{ item }}</span>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_javascript_global() {
    let sfc = r#"<script setup>
</script>
<template>
  <span>{{ Math.max(1, 2) }}</span>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn does_not_report_vue_router_route_or_router() {
    let sfc = r#"<script setup lang="ts">
const label = "Back";
</script>
<template>
  <a v-if="$route.name === 'home'" href="/" @click="() => $router.push('/list')">{{ label }} {{ missing }}</a>
</template>
"#;
    let expected = undefined_at(sfc, "missing");
    assert_eq!(
        findings(&lint_sfc(sfc))
            .into_iter()
            .map(|(rule, severity, start, end, message)| {
                (rule, severity, start, end, message.to_string())
            })
            .collect::<Vec<_>>(),
        vec![expected]
    );
}

#[test]
fn plugin_proxy_globals_do_not_hide_undefined_bare_identifiers() {
    let sfc = include_str!("../../../../tests/fixtures/no-undefined-refs/plugin-globals.vue");
    let expected = undefined_at(sfc, "missing");
    assert_eq!(
        findings(&lint_sfc(sfc))
            .into_iter()
            .map(|(rule, severity, start, end, message)| {
                (rule, severity, start, end, message.to_string())
            })
            .collect::<Vec<_>>(),
        vec![expected]
    );
}

#[test]
fn imported_props_shapes_are_not_assumed_to_be_closed() {
    let sfc = include_str!("../../../../tests/fixtures/no-undefined-refs/imported-props.vue");
    assert_eq!(findings(&lint_sfc(sfc)), none());
    for declaration in [
        "withDefaults(defineProps<StatusBoxProps>(), { status: 'default' });",
        "type Props = StatusBoxProps; defineProps<Props>();",
        "interface Props extends StatusBoxProps { label: string }; defineProps<Props>();",
        "defineProps<Pick<StatusBoxProps, 'status'>>();",
        "type Props = StatusBoxProps & { label: string }; defineProps<Props>();",
    ] {
        let source = sfc.replace("defineProps<StatusBoxProps>();", declaration);
        assert_eq!(findings(&lint_sfc(&source)), none(), "{declaration}");
    }
}

#[test]
fn imported_base_of_exported_props_interface_is_not_assumed_closed() {
    for (prop, required) in [("forceMount", false), ("value", true)] {
        let optional = if required { "" } else { "?" };
        let source = format!(
            r#"<script lang="ts">
import type {{ BaseProps }} from './base'
export interface Props extends Omit<
  BaseProps,
  'modelValue' | 'defaultValue'
> {{
  {prop}{optional}: boolean
}}
</script>
<script setup lang="ts">
const props = defineProps<Props>()
void props
</script>
<template>{{{{ {prop} }}}}</template>
"#
        );
        assert_eq!(findings(&lint_sfc(&source)), none(), "{prop}");
    }
}

#[test]
fn closed_props_shapes_keep_reporting_unknown_names() {
    for declaration in [
        "defineProps<{ status?: 'default' | 'error' }>();",
        "interface Props { status?: 'default' | 'error' }; defineProps<Props>();",
        "type Props = { status?: 'default' | 'error' }; defineProps<Props>();",
        "defineProps({ status: String });",
        "defineProps<{}>();",
        "interface Props {}; defineProps<Props>();",
        "interface Props { status?: StatusBoxProps }; defineProps<Props>();",
    ] {
        let source = format!(
            "<script setup lang=\"ts\">\nimport type {{ StatusBoxProps }} from './status-box.types';\n{declaration}\n</script>\n<template>{{{{ missing }}}}</template>\n"
        );
        let expected = undefined_at(&source, "missing");
        assert_eq!(
            findings(&lint_sfc(&source))
                .into_iter()
                .map(|(rule, severity, start, end, message)| {
                    (rule, severity, start, end, message.to_string())
                })
                .collect::<Vec<_>>(),
            vec![expected],
            "{declaration}"
        );
    }
}

/// Every message of the rule, sorted so the template walk order is not pinned.
fn names(result: &LintResult) -> Vec<&str> {
    let mut names = findings(result)
        .into_iter()
        .map(|(_, _, _, _, message)| message)
        .collect::<Vec<_>>();
    names.sort_unstable();
    names
}

#[test]
fn a_configured_global_is_defined_and_a_near_miss_is_still_reported() {
    let sfc = r#"<script setup lang="ts">
const count = 1
</script>
<template>
  <p :title="shout('a')" @click="formatCount(count)">{{ formatCount(count) }} {{ formatCuont(count) }}</p>
</template>
"#;
    assert_eq!(
        names(&lint_sfc(sfc)),
        [
            "Variable 'formatCount' is not defined",
            "Variable 'formatCount' is not defined",
            "Variable 'formatCuont' is not defined",
            "Variable 'shout' is not defined",
        ]
    );
    let configured = Linter::new()
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_undefined_ref_globals(vec!["formatCount".into(), "shout".into()])
        .lint_sfc(sfc, "Probe.vue");
    let expected = undefined_at(sfc, "formatCuont");
    assert_eq!(
        findings(&configured)
            .into_iter()
            .map(|(rule, severity, start, end, message)| {
                (rule, severity, start, end, message.to_string())
            })
            .collect::<Vec<_>>(),
        vec![expected]
    );
}

#[test]
fn config_globals_reach_the_rule_and_an_empty_list_resets_them() {
    let sfc = "<script setup>\nconst known = 1\n</script>\n<template>{{ known }} {{ formatCount(1) }} {{ missing }}</template>\n";
    let lint = |options: serde_json::Value| {
        let options: vize_l0::config::ConfigLintRuleOptions =
            serde_json::from_value(options).unwrap();
        Linter::new()
            .with_enabled_rules(Some(vec![RULE.into()]))
            .with_config_rule_options(&options)
            .lint_sfc(sfc, "Probe.vue")
    };
    assert_eq!(
        names(&lint(serde_json::json!({
            "vue/no-undefined-refs": { "globals": ["formatCount"] }
        }))),
        ["Variable 'missing' is not defined"]
    );
    for unconfigured in [
        serde_json::json!({}),
        serde_json::json!({ "vue/no-undefined-refs": { "globals": [] } }),
    ] {
        assert_eq!(
            names(&lint(unconfigured)),
            [
                "Variable 'formatCount' is not defined",
                "Variable 'missing' is not defined",
            ]
        );
    }
}

#[test]
fn globals_keep_the_plugin_property_handling_and_do_not_enable_the_rule() {
    let sfc = "<script setup>\nconst known = 1\n</script>\n<template>{{ known }} {{ $t('title') }} {{ $missing }} {{ missing }}</template>\n";
    let configured = Linter::new()
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_undefined_ref_globals(vec!["formatCount".into()])
        .lint_sfc(sfc, "Probe.vue");
    assert_eq!(names(&configured), ["Variable 'missing' is not defined"]);

    let essential = Linter::with_preset(crate::preset::LintPreset::Essential)
        .with_undefined_ref_globals(vec!["formatCount".into()]);
    assert!(!essential.registry().has_rule(RULE));
}

#[test]
fn options_api_members_are_defined_with_or_without_a_define_component_import() {
    let options = r#"{
  props: ['label', 'initial-value'],
  inject: { theme: { from: 'theme' } },
  data() {
    return { count: 1 }
  },
  setup() {
    const route = useRoute()
    return { route }
  },
  computed: {
    upper(): string {
      return this.label.toUpperCase()
    },
  },
  methods: {
    increment() {
      this.count += 1
    },
  },
}"#;
    for script in [
        format!("export default defineComponent({options})"),
        format!(
            "import {{ defineComponent }} from 'vue'\nexport default defineComponent({options})"
        ),
        format!("export default {options}"),
    ] {
        let sfc = format!(
            "<script lang=\"ts\">\n{script}\n</script>\n<template>\n  <button :title=\"theme\" @click=\"increment\">{{{{ route.path }}}} {{{{ upper }}}} {{{{ label }}}} {{{{ initialValue }}}} {{{{ count }}}} {{{{ missing }}}}</button>\n</template>\n"
        );
        let expected = undefined_at(&sfc, "missing");
        assert_eq!(
            findings(&lint_sfc(&sfc))
                .into_iter()
                .map(|(rule, severity, start, end, message)| {
                    (rule, severity, start, end, message.to_string())
                })
                .collect::<Vec<_>>(),
            vec![expected],
            "{script}"
        );
    }
}

#[test]
fn the_reported_auto_import_pages_keep_only_the_near_miss() {
    let page = include_str!(
        "../../../../../../tests/_fixtures/differential/linter/no-undefined-refs-project-globals/AutoImports.vue.txt"
    );
    let options_page = include_str!(
        "../../../../../../tests/_fixtures/differential/linter/no-undefined-refs-project-globals/OptionsPage.vue.txt"
    );
    let options: vize_l0::config::ConfigLintRuleOptions = serde_json::from_str(include_str!(
        "../../../../../../tests/_fixtures/differential/linter/no-undefined-refs-project-globals/options.json"
    ))
    .unwrap();
    let lint = |source: &str, project_globals: bool| {
        let linter = Linter::new().with_enabled_rules(Some(vec![RULE.into()]));
        if project_globals {
            linter.with_config_rule_options(&options)
        } else {
            linter
        }
        .lint_sfc(source, "Page.vue")
    };

    let expected = undefined_at(page, "formatCuont");
    assert_eq!(
        findings(&lint(page, true))
            .into_iter()
            .map(|(rule, severity, start, end, message)| {
                (rule, severity, start, end, message.to_string())
            })
            .collect::<Vec<_>>(),
        vec![expected]
    );
    assert_eq!(findings(&lint(options_page, true)), none());

    // Without the project's names each auto-import reads as undefined, while
    // the members the component declares itself stay resolved.
    let unconfigured = lint(page, false);
    let mut undefined_names = names(&unconfigured);
    undefined_names.dedup();
    assert_eq!(
        undefined_names,
        [
            "Variable 'formatCount' is not defined",
            "Variable 'formatCuont' is not defined",
            "Variable 'isDefined' is not defined",
            "Variable 'navigateTo' is not defined",
            "Variable 'resolveComponent' is not defined",
            "Variable 'sharedSum' is not defined",
            "Variable 'shout' is not defined",
            "Variable 'useColor' is not defined",
        ]
    );
    assert_eq!(
        names(&lint(options_page, false)),
        ["Variable 'formatCount' is not defined"]
    );
}
