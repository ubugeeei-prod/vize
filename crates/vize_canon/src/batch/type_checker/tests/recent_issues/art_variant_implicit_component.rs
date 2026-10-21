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

    let wrong_type = "error Type 'number' is not assignable to type 'string'.";
    let [
        implicit_wrong,
        implicit_missing,
        explicit_wrong,
        explicit_missing,
    ] = snapshot.as_slice()
    else {
        panic!("expected the exact implicit and explicit prop diagnostic pairs: {snapshot:?}");
    };
    assert_eq!(
        (
            &*implicit_wrong.0,
            implicit_wrong.1,
            diagnostic_body(&implicit_wrong.2, "5:34:"),
        ),
        ("src/MyCard.art.vue", Some(2322), wrong_type)
    );
    assert_eq!(
        (
            &*explicit_wrong.0,
            explicit_wrong.1,
            diagnostic_body(&explicit_wrong.2, "6:34:"),
        ),
        ("src/MyCardExplicit.art.vue", Some(2322), wrong_type)
    );
    assert_eq!(
        (&*implicit_missing.0, implicit_missing.1),
        ("src/MyCard.art.vue", Some(2345))
    );
    assert_eq!(
        (&*explicit_missing.0, explicit_missing.1),
        ("src/MyCardExplicit.art.vue", Some(2345))
    );

    let implicit_missing =
        normalize_native_attribute_pretty_print(diagnostic_body(&implicit_missing.2, "6:28:"));
    let explicit_missing =
        normalize_native_attribute_pretty_print(diagnostic_body(&explicit_missing.2, "7:28:"));
    let missing_title = concat!(
        "error Argument of type '{ elevated: true; }' is not assignable to parameter of type ",
        "'__VizeComponentCheckProps<Props, __VizePublicComponentAttrs & <native attributes>>'.\n",
        "Property 'title' is missing in type '{ elevated: true; }' but required in type ",
        "'{ readonly title: string; readonly elevated?: boolean | undefined; }'.",
    );
    assert_eq!(implicit_missing, missing_title);
    assert_eq!(explicit_missing, missing_title);
}

fn diagnostic_body<'a>(message: &'a str, site: &str) -> &'a str {
    message
        .strip_prefix(site)
        .expect("diagnostic snapshot keeps the exact line and column prefix")
}

fn normalize_native_attribute_pretty_print(message: &str) -> String {
    const PREFIX: &str = concat!(
        "error Argument of type '{ elevated: true; }' is not assignable to parameter of type ",
        "'__VizeComponentCheckProps<Props, __VizePublicComponentAttrs",
    );
    const SUFFIX: &str = ">'.\n";
    let body = message
        .strip_prefix(PREFIX)
        .expect("missing-prop diagnostic keeps the component-check prefix");
    let (_, property) = body
        .rsplit_once(SUFFIX)
        .expect("missing-prop diagnostic keeps its complete native-attribute type");
    format!("{PREFIX} & <native attributes>{SUFFIX}{property}")
}

#[test]
fn native_attribute_pretty_print_normalization_keeps_the_exact_prop_error() {
    const PREFIX: &str = concat!(
        "error Argument of type '{ elevated: true; }' is not assignable to parameter of type ",
        "'__VizeComponentCheckProps<Props, __VizePublicComponentAttrs",
    );
    const PROPERTY: &str = concat!(
        "Property 'title' is missing in type '{ elevated: true; }' but required in type ",
        "'{ readonly title: string; readonly elevated?: boolean | undefined; }'.",
    );
    let expected = format!("{PREFIX} & <native attributes>>'.\n{PROPERTY}");

    for native_attributes in [
        " & { [x: string]: unknown; } & { [x: `data${string}`]: unknown; } & Record<string, unknown>",
        " & { abbr?: unknown; ... 673 more ...; zoomAndPan?: unknown; } & { ...; } & Record<...>",
    ] {
        let message = format!("{PREFIX}{native_attributes}>'.\n{PROPERTY}");
        assert_eq!(normalize_native_attribute_pretty_print(&message), expected);
    }
}
