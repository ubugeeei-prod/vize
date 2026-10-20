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
    assert!(snapshot.is_empty(), "capture exact differential diagnostics: {snapshot:#?}");

    for file in ["src/MyCard.art.vue", "src/MyCardExplicit.art.vue"] {
        assert!(
            snapshot
                .iter()
                .any(|(candidate, code, message)| candidate == file
                    && *code == Some(2322)
                    && message.contains("number")
                    && message.contains("string")),
            "{file} must report the wrong title type: {snapshot:#?}"
        );
        assert!(
            snapshot
                .iter()
                .any(|(candidate, code, message)| candidate == file
                    && *code == Some(2345)
                    && message.contains("title")
                    && message.contains("missing")),
            "{file} must report the missing required title: {snapshot:#?}"
        );
    }
}
