use super::{SfcTypeCheckOptions, type_check_sfc};

#[test]
fn optional_chain_union_prop_members_are_not_template_props() {
    let source = r#"<script setup lang="ts">
interface Props {
  isOpened: boolean
  title: string
  timeout?: number
  interaction?:
    | {
        text: string
        to: string
        event?: never
      }
    | {
        text: string
        event: () => void
        to?: never
      }
}

const props = withDefaults(defineProps<Props>(), {
  timeout: 4000
})

void props
</script>

<template>
  <AfsButton v-if="interaction?.to" :to="interaction.to">
    {{ interaction.text }}
  </AfsButton>
  <AfsButton v-else-if="interaction?.event" @click="interaction.event">
    {{ interaction.text }}
  </AfsButton>
</template>"#;
    let options = SfcTypeCheckOptions::new("AfsSnackbar.vue").with_virtual_ts();
    let result = type_check_sfc(source, &options);
    let virtual_ts = result.virtual_ts.expect("virtual ts should be generated");

    assert!(
        virtual_ts.contains(r#"const interaction = props["interaction"];"#),
        "top-level interaction prop must stay available:\n{virtual_ts}"
    );
    for member in ["to", "event", "text"] {
        assert!(
            !virtual_ts.contains(&format!(r#"const {member} = props["{member}"]"#)),
            "member `{member}` must not be emitted as a top-level prop:\n{virtual_ts}"
        );
    }
    assert!(
        !result.diagnostics.iter().any(|diagnostic| {
            diagnostic.code.as_deref() == Some("TS7053")
                && diagnostic.message.contains("\"to\"")
                && diagnostic
                    .message
                    .contains("__WithDefaultsResult<Props, Pick<Props, \"timeout\">>")
        }),
        "optional-chain member must not report a top-level prop TS7053: {:#?}",
        result.diagnostics
    );
}

#[test]
fn optional_chain_inside_slot_vbind_object_stays_guarded() {
    let source = r#"<script setup lang="ts">
const external = false
const scope: { isActive: boolean } | undefined = undefined
</script>

<template>
  <slot v-bind="external ? { isActive: undefined } : { isActive: scope?.isActive }" />
</template>"#;
    let options = SfcTypeCheckOptions::new("SlotForwarder.vue").with_virtual_ts();
    let result = type_check_sfc(source, &options);
    let virtual_ts = result.virtual_ts.expect("virtual ts should be generated");

    assert!(
        virtual_ts.contains("external ? { isActive: undefined } : { isActive: scope?.isActive }"),
        "slot v-bind ternary object must preserve optional chaining:\n{virtual_ts}"
    );
    assert!(
        !virtual_ts.contains("{ isActive: scope.isActive }"),
        "slot v-bind ternary object must not emit an unguarded member access:\n{virtual_ts}"
    );
}
