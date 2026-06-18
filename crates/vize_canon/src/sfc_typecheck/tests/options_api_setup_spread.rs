use crate::sfc_typecheck::{SfcTypeCheckOptions, type_check_sfc_with_options_api};

#[test]
fn options_api_setup_return_spread_exposes_template_bindings() {
    let source = r#"<script lang="ts">
function defineComponent(options: any): any { return options }
function toRefs<T extends Record<string, any>>(value: T): { [K in keyof T]: { value: T[K] } } {
  return value as any
}

function useAiSupportForm() {
  return {
    formInput: {
      aiSupportTitle: '',
      aiSupportType: '',
      aiSupportTagName: '',
    },
  }
}

export default defineComponent({
  setup() {
    const { formInput } = useAiSupportForm()
    return {
      ...toRefs(formInput),
    }
  },
})
</script>
<template>
  <div>{{ aiSupportTitle }} {{ aiSupportType }} {{ aiSupportTagName }}</div>
</template>"#;

    let result = type_check_sfc_with_options_api(
        source,
        &SfcTypeCheckOptions::new("OptionsSetupSpread.vue"),
    );
    let unexpected: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic.code.as_deref(),
                Some("2304" | "undefined-binding")
            )
        })
        .collect();

    assert!(
        unexpected.is_empty(),
        "Options API setup return spread bindings should be available in template scope: {unexpected:#?}"
    );
}
