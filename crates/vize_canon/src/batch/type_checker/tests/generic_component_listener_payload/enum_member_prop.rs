use super::*;

#[test]
fn infers_generic_emit_payload_from_enum_member_prop() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "issue-5475-generic-emit-enum-member-prop",
        &[
            (
                "node_modules/@scope/dialogs/package.json",
                r#"{
  "name": "@scope/dialogs",
  "type": "module",
  "exports": {
    ".": "./src/index.ts",
    "./types": "./src/types.ts"
  },
  "main": "./src/index.ts",
  "module": "./src/index.ts"
}
"#,
            ),
            (
                "node_modules/@scope/dialogs/src/types.ts",
                r#"export enum RegistrationType {
  NormalOneto = "NormalOneto",
  TextbookOneto = "TextbookOneto",
  CauseWeakOneto = "CauseWeakOneto",
}

export type ParsedContentsInfoRaw<T extends RegistrationType> =
  T extends RegistrationType.NormalOneto
    ? { normalId: string }
    : T extends RegistrationType.TextbookOneto
      ? { textbookId: number }
      : { weakId: boolean };

export type ParsedQuestionItemRaw<T extends RegistrationType> =
  T extends RegistrationType.NormalOneto
    ? { normalQuestionId: string }
    : T extends RegistrationType.TextbookOneto
      ? { textbookQuestionId: number }
      : { weakQuestionId: boolean };
"#,
            ),
            (
                "node_modules/@scope/dialogs/src/Dialog.vue",
                r#"<script setup lang="ts" generic="T extends RegistrationType.NormalOneto | RegistrationType.TextbookOneto | RegistrationType.CauseWeakOneto">
import { RegistrationType, type ParsedContentsInfoRaw, type ParsedQuestionItemRaw } from "./types";

const { registrationType } = defineProps<{
  registrationType: T;
}>();

void registrationType;

defineEmits<{
  parsed: [
    value: {
      contentsInfo: ParsedContentsInfoRaw<T>;
      questionItems: ParsedQuestionItemRaw<T>[];
    },
  ];
  cancel: [];
}>();

defineModel<boolean>({ required: true });
</script>

<template>
  <button />
</template>
"#,
            ),
            (
                "node_modules/@scope/dialogs/src/index.ts",
                r#"export { default as Dialog } from "./Dialog.vue";
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import { Dialog } from "@scope/dialogs";
import { RegistrationType, type ParsedContentsInfoRaw, type ParsedQuestionItemRaw } from "@scope/dialogs/types";

let isClosed = false;
let opened = true;

function onParsed({
  contentsInfo,
  questionItems,
}: {
  contentsInfo: ParsedContentsInfoRaw<RegistrationType.NormalOneto>;
  questionItems: ParsedQuestionItemRaw<RegistrationType.NormalOneto>[];
}) {
  contentsInfo.normalId.toUpperCase();
  questionItems[0]?.normalQuestionId.toUpperCase();
}

function cancel() {}
</script>

<template>
  <Dialog
    v-if="!isClosed"
    v-model="opened"
    :registration-type="RegistrationType.NormalOneto"
    @parsed="onParsed"
    @cancel="cancel"
  />
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.is_empty(),
        "enum member prop should narrow generic emit payload: {snapshot:#?}"
    );
    let _ = std::fs::remove_dir_all(&project_root);
}
