use std::fs;

use crate::virtual_ts::VirtualTsOptions;

use super::unique_case_dir;

#[test]
fn document_generator_exposes_with_defaults_props_to_template_scope() {
    let case_dir = unique_case_dir("with-defaults-template-props");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(&case_dir).unwrap();
    let vue_path = case_dir.join("AfsSnackbar.vue");
    let vue_content = r#"<script setup lang="ts">
interface Props {
  isOpened: boolean
  title: string
  timeout?: number
  interaction?:
    | { text: string; to: string; event?: never }
    | { text: string; event: () => void; to?: never }
}

const props = withDefaults(defineProps<Props>(), {
  timeout: 4000
})

void props
</script>

<template>
  <AfsButton v-if="interaction?.to" :to="interaction.to">
    {{ title }} {{ interaction.text }}
  </AfsButton>
  <span v-if="isOpened">{{ title }}</span>
</template>
"#;

    let rewriter = super::super::super::import_rewriter::ImportRewriter::new();
    let virtual_ts = super::super::generate_vue_document_virtual_ts(
        &vue_path,
        vue_content,
        &VirtualTsOptions::default(),
        &rewriter,
        true,
    )
    .unwrap()
    .pre_rewrite_code;

    for prop in ["isOpened", "title", "interaction"] {
        assert!(
            virtual_ts.contains(&format!(r#"const {prop} = props["{prop}"]"#)),
            "top-level prop `{prop}` must be emitted for template scope:\n{virtual_ts}"
        );
    }
    for member in ["to", "event", "text"] {
        assert!(
            !virtual_ts.contains(&format!(r#"const {member} = props["{member}"]"#)),
            "union member `{member}` must not be emitted as a top-level prop:\n{virtual_ts}"
        );
    }

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn legacy_document_generator_exposes_inline_define_props_to_template_scope() {
    let case_dir = unique_case_dir("legacy-inline-template-props");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(&case_dir).unwrap();
    let vue_path = case_dir.join("DialogHost.vue");
    let vue_content = r#"<script setup lang="ts">
type TargetStudent = { id: number; username: string; name: string };

const props = withDefaults(defineProps<{
  targetStudent?: TargetStudent | null
}>(), {
  targetStudent: null,
})

void props
</script>

<template>
  <PlainDialog :is-opened="targetStudent !== null">
    {{ targetStudent?.username ?? '--' }}
  </PlainDialog>
</template>
"#;

    let rewriter = super::super::super::import_rewriter::ImportRewriter::new();
    let virtual_ts = super::super::generate_vue_document_virtual_ts_with_options(
        &vue_path,
        vue_content,
        &VirtualTsOptions::default(),
        &rewriter,
        true,
        super::super::VueDocumentVirtualTsOptions {
            legacy_vue2: true,
            options_api: false,
        },
    )
    .unwrap()
    .pre_rewrite_code;

    assert!(
        virtual_ts.contains(
            r#"const targetStudent = props["targetStudent"] as Exclude<__WithDefaultsResult<Props, Pick<Props, "targetStudent">>["targetStudent"], undefined>;"#
        ),
        "legacy Vue2 template scope must expose inline defineProps keys:\n{virtual_ts}"
    );

    let _ = fs::remove_dir_all(&case_dir);
}
