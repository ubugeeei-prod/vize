//! The component named by a string-literal `defineArt()` source is an implicit
//! binding inside `<variant>` markup (#9854).

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn implicit_art_component_reports_the_same_prop_errors_as_an_explicit_import() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "art-variant-implicit-component",
        &[
            (
                "src/MyCard.vue",
                r#"<script setup lang="ts">
defineProps<{ title: string; elevated?: boolean }>();
</script>
<template><section>{{ title }}</section></template>
"#,
            ),
            (
                "src/MyCard.art.vue",
                r#"<script setup lang="ts">
defineArt("./MyCard.vue", { title: "MyCard" });
</script>
<art>
  <variant name="Wrong"><MyCard :title="123" /></variant>
  <variant name="Missing"><MyCard elevated /></variant>
</art>
"#,
            ),
            (
                "src/MyCardExplicit.art.vue",
                r#"<script setup lang="ts">
import MyCard from "./MyCard.vue";
defineArt("./MyCard.vue", { title: "MyCard" });
</script>
<art>
  <variant name="Wrong"><MyCard :title="123" /></variant>
  <variant name="Missing"><MyCard elevated /></variant>
</art>
"#,
            ),
        ],
    );
    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    let wrong_type = "Type 'number' is not assignable to type 'string'.";
    let missing_title = concat!(
        "Argument of type '{ elevated: true; }' is not assignable to parameter of type ",
        "'__VizeComponentCheckProps<Props, __VizePublicComponentAttrs & { [x: string]: unknown; } & ",
        "{ [x: `data${string}`]: unknown; } & Record<string, unknown>>'.\n",
        "Property 'title' is missing in type '{ elevated: true; }' but required in type ",
        "'{ readonly title: string; readonly elevated?: boolean | undefined; }'.",
    );
    let expected: Vec<(vize_carton::String, Option<u32>, vize_carton::String)> = vec![
        ("src/MyCard.art.vue".into(), Some(2322), wrong_type.into()),
        (
            "src/MyCard.art.vue".into(),
            Some(2345),
            missing_title.into(),
        ),
        (
            "src/MyCardExplicit.art.vue".into(),
            Some(2322),
            wrong_type.into(),
        ),
        (
            "src/MyCardExplicit.art.vue".into(),
            Some(2345),
            missing_title.into(),
        ),
    ];
    assert_eq!(snapshot, expected);
}
