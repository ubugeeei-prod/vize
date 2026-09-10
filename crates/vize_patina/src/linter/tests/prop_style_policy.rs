use super::{LintPreset, Linter};

#[test]
fn opinionated_allows_prop_driven_inline_style() {
    let linter = Linter::with_preset(LintPreset::Opinionated).with_enabled_rules(Some(vec![
        "vue/no-inline-style".into(),
        "css/no-v-bind-performance".into(),
    ]));
    let inline_style = r#"<template>
  <span :class="['icon', `icon-${size}`]" :style="color ? { color } : undefined" />
</template>
"#;

    let inline_result = linter.lint_sfc(inline_style, "Icon.vue");
    assert_eq!(
        inline_result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.rule_name)
            .collect::<Vec<_>>(),
        Vec::<&str>::new()
    );

    let css_v_bind = r#"<template><span class="icon" /></template>
<style scoped>
.icon {
  color: v-bind("color ?? 'inherit'");
}
</style>
"#;
    let css_result = linter.lint_sfc(css_v_bind, "Icon.vue");
    assert_eq!(
        css_result
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.rule_name, diagnostic.message.as_str()))
            .collect::<Vec<_>>(),
        vec![(
            "css/no-v-bind-performance",
            "v-bind() installs runtime CSS variable updates"
        )]
    );
}
