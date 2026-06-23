use super::*;
use crate::batch::BatchTypeCheckerOptions;
use crate::virtual_ts::VirtualTsOptions;

#[test]
fn batch_type_checker_reports_camel_case_child_component_prop_error() {
    // Repro 15 from ushironoko/vize-config-repro: `__VizeComponentProps<T>`
    // is a camel/kebab union, so prop value extraction must distribute over
    // the union instead of using `keyof` on the union as a whole.
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "camel-case-child-component-props",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{
  countTotal: number
}>()
</script>

<template>
  <span>{{ countTotal }}</span>
</template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Child from "./Child.vue";

const wrong: string = "not a number";
</script>

<template>
  <Child :countTotal="wrong" />
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
        snapshot
            .iter()
            .any(|(file, code, message)| file == "src/Parent.vue"
                && *code == Some(2322)
                && message.contains("Type 'string' is not assignable to type 'number'")),
        "expected camelCase child prop mismatch to report TS2322: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_accepts_forwarded_optional_component_props() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "optional-component-props",
        &[
            (
                "src/Provider.vue",
                r#"<script lang="ts">
export type LinkBehavior = "window" | "browser" | null;
</script>

<script setup lang="ts">
defineProps<{
  behavior?: LinkBehavior;
}>();
</script>

<template>
  <a><slot /></a>
</template>
"#,
            ),
            (
                "src/Consumer.vue",
                r#"<script setup lang="ts">
import Provider from "./Provider.vue";
import type { LinkBehavior } from "./Provider.vue";

defineProps<{
  behavior?: LinkBehavior;
}>();
</script>

<template>
  <Provider :behavior="behavior" />
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
        "forwarded optional component prop should type-check, got: {snapshot:?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_keeps_v_for_items_separate_from_kebab_component_props() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "v-for-item-component-props",
        &[
            (
                "src/AfKeyboardButton.vue",
                r#"<script setup lang="ts">
export type KeyboardButtonType = "function" | "input" | "submit" | "system";

defineProps<{
  keyType?: KeyboardButtonType;
  innerHtml?: string;
  width?: string;
  longPress?: boolean;
}>();
</script>

<template>
  <button><slot /></button>
</template>
"#,
            ),
            (
                "src/AfHiraganaKeyboard.vue",
                r#"<script setup lang="ts">
import AfKeyboardButton from "./AfKeyboardButton.vue";

type KeyboardKey = InstanceType<typeof AfKeyboardButton>["$props"] & {
  onClick?: () => void;
};
type KeyboardLayout = KeyboardKey[][];

const keyboardLayout: KeyboardLayout = [
  [
    {
      keyType: "input",
      innerHtml: "あ",
      width: "1fr",
      longPress: true,
    },
  ],
];
</script>

<template>
  <div v-for="(row, rowIndex) in keyboardLayout" :key="rowIndex">
    <AfKeyboardButton
      v-for="(key, i) in row"
      :key="`hiragana-key-${i}`"
      :key-type="key.keyType"
      :inner-html="key.innerHtml"
      :width="key.width"
      :long-press="key.longPress"
    />
  </div>
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
        "v-for item model should stay readable through camelCase props, got: {snapshot:?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_legacy_vue2_accepts_vuetify_global_events_and_props() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "legacy-vue2-vuetify-global-events-props",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
const width = 320;
const hideDetails = true;
function updateDate(newDate: string) {
  void newDate;
}
</script>

<template>
  <v-date-picker
    :width="width"
    :hide-details="hideDetails"
    chips
    @input="updateDate"
  />
</template>
"#,
        )],
    );

    let options = BatchTypeCheckerOptions {
        virtual_ts_options: VirtualTsOptions {
            auto_import_stubs: vec![
                "declare const VDatePicker: { new (): { $props: { mini?: boolean } } };".into(),
            ],
            external_template_bindings: vec!["VDatePicker".into()],
            ..Default::default()
        },
        ..Default::default()
    };
    let mut checker = match BatchTypeChecker::with_options(&project_root, options) {
        Ok(checker) => checker,
        Err(_) => {
            let _ = std::fs::remove_dir_all(&project_root);
            return;
        }
    };
    checker.enable_legacy_vue2();
    checker.scan_project().unwrap();
    let result = checker.check_project().unwrap();
    let relevant = result
        .diagnostics
        .iter()
        .filter(|diagnostic| relative_path(&project_root, &diagnostic.file) == "src/App.vue")
        .filter(|diagnostic| {
            diagnostic.code == Some(2345)
                || diagnostic.message.contains("InputEvent")
                || diagnostic.message.contains("keyof Props")
                || diagnostic.message.contains("hideDetails")
                || diagnostic.message.contains("width")
                || diagnostic.message.contains("chips")
        })
        .collect::<Vec<_>>();

    assert!(
        relevant.is_empty(),
        "legacy Vue 2 Vuetify globals should not report event/prop false positives: {relevant:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
