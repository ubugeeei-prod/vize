use super::NoUnusedComponents;
use crate::linter::Linter;
use crate::rule::{Rule, RuleCategory};

#[test]
fn test_meta() {
    let rule = NoUnusedComponents::default();
    assert_eq!(rule.meta().name, "vue/no-unused-components");
    assert_eq!(rule.meta().category, RuleCategory::Essential);
}

#[test]
fn test_should_ignore() {
    let rule = NoUnusedComponents::default();
    assert!(rule.should_ignore("_Internal"));
    assert!(!rule.should_ignore("MyComponent"));
}

fn lint_messages(sfc: &str) -> Vec<String> {
    Linter::new()
        .with_enabled_rules(Some(vec!["vue/no-unused-components".into()]))
        .lint_sfc(sfc, "test.vue")
        .diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.message.to_string())
        .collect()
}

#[test]
fn test_plain_script_component_value_is_not_registration() {
    let sfc = r#"<template>
  <DocSections :docs="docs" />
</template>

<script>
import DocApiTable from './DocApiTable.vue'

export default {
  data() {
    return {
      docs: [{ component: DocApiTable }]
    }
  }
}
</script>
"#;

    assert_eq!(lint_messages(sfc), Vec::<String>::new());
}

#[test]
fn test_options_api_registration_reports_public_alias() {
    let sfc = r#"<script>
import Style from './style.vue'

export default {
  components: {
    FourStyle: Style,
  },
}
</script>

<template>
  <div />
</template>
"#;

    assert_eq!(
        lint_messages(sfc),
        vec!["Component 'FourStyle' is registered but never used in template"]
    );
}

#[test]
fn test_registered_library_component_reports_unused() {
    let sfc = r#"<script>
import { SfButton } from '@storefront-ui/vue'

export default {
  components: {
    SfButton,
  },
}
</script>

<template>
  <div />
</template>
"#;

    assert_eq!(
        lint_messages(sfc),
        vec!["Component 'SfButton' is registered but never used in template"]
    );
}
