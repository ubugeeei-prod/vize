//! Nuxt lazy component intersections retain props from both members (#13689).

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};

macro_rules! reported {
    ($file:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/typechecker/lazy-component-intersection-props-13689/",
            $file
        ))
    };
}

#[test]
fn lazy_component_intersections_accept_the_reported_hydration_prop() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project = super::real_vue::create_project_case(
        "lazy-component-intersection-reported-13689",
        &[
            ("tsconfig.json", reported!("tsconfig.json.txt")),
            ("Foo.vue", reported!("Foo.vue.txt")),
            ("lazy.ts", reported!("lazy.ts.txt")),
            ("App.vue", reported!("App.vue.txt")),
        ],
    );
    let actual = snapshot_project_diagnostics(&project);
    let _ = std::fs::remove_dir_all(project);
    assert_eq!(actual, Some(vec![]));
}

const OWN_PROPS: &str = r#"<script setup lang="ts">
defineProps<{ requiredCount: number }>();
</script>
<template><div /></template>"#;

const WRAPPERS: &str = r#"import type { DefineComponent } from "vue";
import Foo from "./Foo.vue";
type Hydration = DefineComponent<{ hydrateOnIdle?: number | true }>;
export declare const Forward: Hydration & typeof Foo;
export declare const Reverse: typeof Foo & Hydration;
export declare const Union: (Hydration & typeof Foo) | typeof Foo;
export declare const Narrow: (new (...args: any[]) => {
  $props: { requiredCount: 1 };
}) & typeof Foo;
export declare const Indexed: (new (...args: any[]) => {
  $props: { [key: string]: any; extra?: number };
}) & typeof Foo;
"#;

#[test]
fn public_extensions_keep_raw_requiredness_values_and_closed_unknown_checks() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project = super::real_vue::create_project_case(
        "lazy-component-intersection-contracts-13689",
        &[
            ("tsconfig.json", reported!("tsconfig.json.txt")),
            ("Foo.vue", OWN_PROPS),
            ("lazy.ts", WRAPPERS),
            (
                "Good.vue",
                r#"<script setup lang="ts">
import { Forward, Reverse, Union, Narrow, Indexed } from "./lazy";
</script>
<template>
  <Forward :required-count="1" hydrate-on-idle />
  <Reverse :required-count="1" :hydrate-on-idle="12" />
  <Union :required-count="1" hydrate-on-idle />
  <Narrow :required-count="1" />
  <Indexed :required-count="1" :extra="1" />
</template>"#,
            ),
            (
                "Missing.vue",
                r#"<script setup lang="ts">
import { Forward } from "./lazy";
</script>
<template><Forward hydrate-on-idle /></template>"#,
            ),
            (
                "Unknown.vue",
                r#"<script setup lang="ts">
import { Forward } from "./lazy";
</script>
<template><Forward :required-count="1" unknown-attribute /></template>"#,
            ),
            (
                "WrongHydration.vue",
                r#"<script setup lang="ts">
import { Reverse } from "./lazy";
</script>
<template><Reverse :required-count="1" :hydrate-on-idle="'wrong'" /></template>"#,
            ),
            (
                "WrongNarrow.vue",
                r#"<script setup lang="ts">
import { Narrow } from "./lazy";
</script>
<template><Narrow :required-count="2" /></template>"#,
            ),
            (
                "WrongIndexed.vue",
                r#"<script setup lang="ts">
import { Indexed } from "./lazy";
</script>
<template><Indexed :required-count="1" :extra="'wrong'" /></template>"#,
            ),
            (
                "UnknownIndexed.vue",
                r#"<script setup lang="ts">
import { Indexed } from "./lazy";
</script>
<template><Indexed :required-count="1" unknown-attribute /></template>"#,
            ),
        ],
    );
    let actual = snapshot_project_diagnostics(&project);
    let _ = std::fs::remove_dir_all(project);
    let mut codes = actual
        .expect("the project typechecks")
        .into_iter()
        .map(|(file, code, _)| (file, code))
        .collect::<Vec<_>>();
    codes.sort();
    assert_eq!(
        codes,
        vec![
            ("Missing.vue".into(), Some(2345)),
            ("Unknown.vue".into(), Some(2353)),
            ("UnknownIndexed.vue".into(), Some(2353)),
            ("WrongHydration.vue".into(), Some(2322)),
            ("WrongIndexed.vue".into(), Some(2322)),
            ("WrongNarrow.vue".into(), Some(2322)),
        ],
    );
}
