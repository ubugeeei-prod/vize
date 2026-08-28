//! Synthetic template diagnostics must never use virtual offsets as SFC positions.

use super::super::{
    BatchTypeChecker, create_project_case, relative_path, resolve_test_tsgo_binary,
};
use crate::batch::{SfcBlockType, TypeChecker};
use vize_s0::{String, cstr};

#[test]
fn unmapped_v_if_guard_does_not_duplicate_into_a_long_style_block() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let mut source = String::from(
        r#"<template>
  <section v-for="foods in menuList">
    <ul v-if="foods.attributes.length">
      <li v-if="attribute" v-for="(attribute, index) in foods.attributes" :key="index">
        {{ attribute }}
      </li>
    </ul>
  </section>
</template>

<script setup lang="ts">
const menuList = [{ attributes: ["size"] }];
</script>

<style>
"#,
    );
    source.push_str(&".row { color: red; }\n".repeat(2_000));
    source.push_str("</style>\n");
    let project_root = create_project_case(
        "unmapped-template-fallback",
        &[("src/App.vue", source.as_str())],
    );
    let mut checker =
        BatchTypeChecker::new(&project_root).expect("batch type checker construction");
    checker.scan_project().expect("project scan");
    let result = checker.check_project().expect("project check");

    let diagnostics: Vec<_> = result
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            (
                relative_path(&project_root, &diagnostic.file),
                diagnostic.code,
                diagnostic.message,
                diagnostic.line + 1,
                diagnostic.column + 1,
                diagnostic.block_type,
            )
        })
        .collect();
    let _ = std::fs::remove_dir_all(&project_root);

    assert_eq!(
        diagnostics,
        vec![(
            cstr!("src/App.vue"),
            Some(2304),
            cstr!("Cannot find name 'attribute'."),
            4,
            17,
            Some(SfcBlockType::Template),
        )]
    );
}
