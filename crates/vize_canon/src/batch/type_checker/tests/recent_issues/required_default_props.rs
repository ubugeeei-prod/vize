//! A runtime prop that carries a `default` is optional for the parent even when
//! it is `required: true`, as on the `$props` `vue-tsc` checks the parent
//! against; a required prop without a default and a required plain `Boolean`
//! prop stay required. The setup side keeps the defaulted value.

use std::path::Path;

use super::super::{
    BatchTypeChecker, create_project_case, relative_path, resolve_test_tsgo_binary,
};
use crate::batch::{BatchTypeCheckerOptions, TypeChecker};
use vize_l0::{String, cstr};

const OPTIONS_CHILD: &str = r#"<template>
  <span :title="title">{{ label.toUpperCase() }} {{ flag }}</span>
</template>

<script lang="ts">
import { defineComponent } from 'vue'

export default defineComponent({
  props: {
    label: { type: String, default: '', required: true },
    flag: { type: Boolean, default: false, required: true },
    title: { type: String, required: true },
  },
})
</script>
"#;

const SETUP_CHILD: &str = r#"<script setup lang="ts">
defineProps({
  label: { type: String, default: '', required: true },
  flag: { type: Boolean, default: false, required: true },
  title: { type: String, required: true },
})
</script>

<template>
  <span :title="title">{{ label.toUpperCase() }} {{ flag }}</span>
</template>
"#;

const SHARED_PROPS: &str = r#"export const shared = {
  label: { type: String, default: '', required: true },
  flag: { type: Boolean, default: false, required: true },
} as const
"#;

const SPREAD_CHILD: &str = r#"<script setup lang="ts">
import { shared } from './shared'

defineProps({
  ...shared,
  title: { type: String, required: true },
})
</script>

<template>
  <span :title="title">{{ label.toUpperCase() }} {{ flag }}</span>
</template>
"#;

const VALID_PARENT: &str = r#"<script setup lang="ts">
import OptionsChild from './OptionsChild.vue'
import SetupChild from './SetupChild.vue'
import SpreadChild from './SpreadChild.vue'

const account: { name?: string } = {}
</script>

<template>
  <OptionsChild :label="account.name" flag title="t" />
  <OptionsChild flag title="t" />
  <SetupChild :label="account.name" flag title="t" />
  <SetupChild flag title="t" />
  <SpreadChild :label="account.name" flag title="t" />
  <SpreadChild flag title="t" />
</template>
"#;

const INVALID_PARENT: &str = r#"<script setup lang="ts">
import OptionsChild from './OptionsChild.vue'
import SetupChild from './SetupChild.vue'
import SpreadChild from './SpreadChild.vue'

const account: { name?: string } = {}
</script>

<template>
  <OptionsChild :title="account.name" flag />
  <OptionsChild title="t" />
  <SetupChild :title="account.name" flag />
  <SetupChild title="t" />
  <SpreadChild :title="account.name" flag />
  <SpreadChild title="t" />
</template>
"#;

fn assert_published_props_line(checker: &BatchTypeChecker, file: &str, expected: &str) {
    let projected = checker.virtual_files();
    let projected = projected
        .iter()
        .find(|projected| projected.original_path.ends_with(file))
        .expect("actual child projection");
    let content = projected.content.as_str();
    assert!(
        content.lines().any(|line| line == expected),
        "{file} must publish `{expected}`:\n{content}"
    );
}

#[test]
fn defaulted_required_props_follow_the_published_instance_props() {
    let Some(native) = resolve_test_tsgo_binary() else {
        assert!(
            std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none(),
            "defaulted required props need the native checker"
        );
        return;
    };
    let project_root = create_project_case(
        "required-default-props",
        &[
            ("src/OptionsChild.vue", OPTIONS_CHILD),
            ("src/SetupChild.vue", SETUP_CHILD),
            ("src/shared.ts", SHARED_PROPS),
            ("src/SpreadChild.vue", SPREAD_CHILD),
            ("src/Valid.vue", VALID_PARENT),
            ("src/Invalid.vue", INVALID_PARENT),
        ],
    );
    let diagnostics = project_diagnostics(&project_root, &native);
    let _ = std::fs::remove_dir_all(&project_root);

    let undefined_member = "Type 'string | undefined' is not assignable to type 'string'.\nType 'undefined' is not assignable to type 'string'.";
    assert_eq!(
        diagnostics,
        vec![
            (
                "src/Invalid.vue".into(),
                Some(2322),
                cstr!("10:18: {undefined_member}")
            ),
            (
                "src/Invalid.vue".into(),
                Some(2322),
                cstr!("12:16: {undefined_member}")
            ),
            (
                "src/Invalid.vue".into(),
                Some(2322),
                cstr!("14:17: {undefined_member}")
            ),
            (
                "src/Invalid.vue".into(),
                Some(2345),
                "11:4: Argument of type '{ title: string; }' is not assignable to parameter of type '$ComponentCheckProps<__OptionsChild_CheckProps_0, $PublicComponentAttrs & { [x: string]: unknown; } & { [x: `data${Capitalize<string>}`]: unknown; [x: `data-${string}`]: unknown; } & Record<...>>'.\nProperty 'flag' is missing in type '{ title: string; }' but required in type '{ readonly label?: string | undefined; readonly flag: boolean; readonly title: string; }'.".into()
            ),
            (
                "src/Invalid.vue".into(),
                Some(2345),
                "13:4: Argument of type '{ title: string; }' is not assignable to parameter of type '$ComponentCheckProps<Props, $PublicComponentAttrs & { [x: string]: unknown; } & { [x: `data${Capitalize<string>}`]: unknown; [x: `data-${string}`]: unknown; } & Record<string, unknown>>'.\nProperty '\"flag\"' is missing in type '{ title: string; }' but required in type '{ readonly label?: string | undefined; readonly flag: boolean; readonly title: string; }'.".into()
            ),
            (
                "src/Invalid.vue".into(),
                Some(2345),
                "15:4: Argument of type '{ title: string; }' is not assignable to parameter of type '$ComponentCheckProps<Props, $PublicComponentAttrs & { [x: string]: unknown; } & { [x: `data${Capitalize<string>}`]: unknown; [x: `data-${string}`]: unknown; } & Record<string, unknown>>'.\nProperty '\"flag\"' is missing in type '{ title: string; }' but required in type '{ readonly title: string; readonly label?: string | undefined; readonly flag: boolean; }'.".into()
            ),
        ]
    );
}

fn project_diagnostics(project_root: &Path, native: &Path) -> Vec<(String, Option<u32>, String)> {
    let mut checker = BatchTypeChecker::with_options_and_corsa_path(
        project_root,
        BatchTypeCheckerOptions::default(),
        Some(native),
    )
    .expect("batch checker should initialize");
    checker.enable_options_api();
    checker.scan_project().expect("project scan should succeed");
    let result = checker.check_project().expect("project check should run");

    for (file, expected) in [
        ("SetupChild.vue", "  \"label\"?: string;"),
        ("SetupChild.vue", "  \"flag\": boolean;"),
        ("SetupChild.vue", "  \"title\": string;"),
        (
            "SpreadChild.vue",
            "  \"label\"?: __RuntimePropShape<typeof shared>[\"label\"];",
        ),
        (
            "SpreadChild.vue",
            "  \"flag\": __RuntimePropShape<typeof shared>[\"flag\"];",
        ),
        ("SpreadChild.vue", "  \"title\": string;"),
    ] {
        assert_published_props_line(&checker, file, expected);
    }

    let mut snapshot: Vec<_> = result
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            (
                relative_path(project_root, &diagnostic.file),
                diagnostic.code,
                cstr!(
                    "{}:{}: {}",
                    diagnostic.line + 1,
                    diagnostic.column + 1,
                    diagnostic.message
                ),
            )
        })
        .collect();
    snapshot.sort();
    snapshot
}
