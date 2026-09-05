use super::super::super::{
    create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics,
};
use vize_s0::String;

#[test]
fn any_v_for_key_matches_vue_tsc_object_fallback() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "external-slot-payload-vfor-any",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
const anyList: any = null;
const arrList: string[] = [];
const objList: Record<string, number> = {};
const unknownList: unknown = null;

function takesNumber(value: number) {
  return value;
}
</script>

<template>
  <div v-for="(item, index) in anyList" :key="index">
    {{ item }}{{ index < arrList.length - 1 ? ',' : '' }}
  </div>
  <div v-for="(item2, index2) in arrList" :key="index2">
    {{ item2 }}{{ index2 < arrList.length - 1 ? ',' : '' }}
  </div>
  <div v-for="(value3, key3, index3) in objList" :key="key3">
    {{ value3 }}{{ takesNumber(index3) }}
  </div>
  <div v-for="(item4, key4, index4) in anyList" :key="key4">
    {{ item4 }}{{ takesNumber(index4) }}
  </div>
  <div v-for="(item5, index5) in unknownList" :key="index5">
    {{ item5 }}{{ takesNumber(index5) }}
  </div>
</template>
"#,
        )],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };
    let _ = std::fs::remove_dir_all(&project_root);

    // vue-tsc treats an `any` source as the object fallback: the second binding
    // is `string | number`, while the third binding remains `number`. This is
    // the exact shape that reports Vuestic Admin's
    // `EditProjectForm.vue:106:35` template-comparison diagnostic.
    assert_eq!(
        snapshot,
        vec![(
            String::from("src/App.vue"),
            Some(2365),
            String::from(
                "14:18:error Operator '<' cannot be applied to types 'string | number' and 'number'."
            ),
        )]
    );
}
