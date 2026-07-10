use crate::sfc_typecheck::{SfcTypeCheckOptions, type_check_sfc_with_legacy_vue2};

#[test]
fn legacy_vue2_required_options_props_in_setup_are_defined() {
    let source = r#"<script lang="ts">
import { defineComponent, type PropType } from 'vue'

const componentProps = {
  items: {
    type: Array as PropType<Array<{ id: string }>>,
    required: true,
  },
}

export default defineComponent({
  props: componentProps,
  setup(props) {
    props.items.findIndex((item) => item.id)
    props.items[0]
    return {}
  },
})
</script>"#;

    let options = SfcTypeCheckOptions::new("RequiredOptionsProps.vue");
    let result = type_check_sfc_with_legacy_vue2(source, &options);
    let unexpected: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic.code.as_deref(),
                Some("18048" | "2532" | "undefined-binding")
            )
        })
        .collect();

    assert!(
        unexpected.is_empty(),
        "Vue 2 required Options API props should be defined in setup(): {unexpected:#?}"
    );
}
