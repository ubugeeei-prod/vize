//! Typed `mixins:` and `extends:` members belong to legacy `$This` (#3609).

use super::super::{
    BatchTypeChecker, create_project_case, relative_path, resolve_test_tsgo_binary,
};
use crate::batch::TypeChecker;

#[test]
fn typed_inherited_members_match_the_clean_vue_tsc_surface() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "options-api-inherited-members",
        &[
            (
                "src/greeter.ts",
                "const greeter = undefined as unknown as new () => { greet(): string }; export default greeter;\n",
            ),
            (
                "src/base.ts",
                "const base = undefined as unknown as new () => { describe(): string }; export default base;\n",
            ),
            (
                "src/untyped.d.ts",
                "declare const untyped: any; export default untyped;\n",
            ),
            ("src/Imported.vue", IMPORTED),
            ("src/Extended.vue", EXTENDED),
            ("src/Inline.vue", INLINE),
            ("src/Untyped.vue", UNTYPED),
        ],
    );
    let mut checker = BatchTypeChecker::new(&project_root).expect("batch checker construction");
    checker.enable_legacy_vue2();
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
                diagnostic.severity,
                diagnostic.block_type,
            )
        })
        .collect();
    let _ = std::fs::remove_dir_all(&project_root);

    assert_eq!(diagnostics, [], "vue-tsc oracle output is empty");
}

const IMPORTED: &str = r#"<script lang="ts">
import greeter from "./greeter";
export default { mixins: [greeter], computed: { consumed() { return this.greet(); } } };
</script>
<template><span /></template>
"#;

const EXTENDED: &str = r#"<script lang="ts">
import base from "./base";
export default { extends: base, computed: { consumed() { return this.describe(); } } };
</script>
<template><span /></template>
"#;

const INLINE: &str = r#"<script lang="ts">
const inlineMixin = undefined as unknown as new () => { ping(): number };
export default { mixins: [inlineMixin], computed: { consumed() { return this.ping(); } } };
</script>
<template><span /></template>
"#;

const UNTYPED: &str = r#"<script lang="ts">
import untyped from "./untyped";
export default { mixins: [untyped], methods: { own() {} }, computed: { consumed() { return 1; } } };
</script>
<template><span /></template>
"#;

/// Native mapped diagnostics, not a reconstruction of the reporter's app.
/// Known signatures must improve without giving opaque dictionaries an index
/// signature on the component's receiver.
#[test]
fn typed_options_object_mixin_methods_keep_arguments_and_unknown_members_checked() {
    let Some(provider) = resolve_test_tsgo_binary() else {
        return;
    };
    let files = [
        (
            "src/Any.vue",
            include_str!(
                "../../../../../../../tests/_fixtures/differential/typechecker/legacy-typed-mixin-methods/src/Any.vue.txt"
            ),
        ),
        (
            "src/Broad.vue",
            include_str!(
                "../../../../../../../tests/_fixtures/differential/typechecker/legacy-typed-mixin-methods/src/Broad.vue.txt"
            ),
        ),
        (
            "src/Extended.vue",
            include_str!(
                "../../../../../../../tests/_fixtures/differential/typechecker/legacy-typed-mixin-methods/src/Extended.vue.txt"
            ),
        ),
        (
            "src/Never.vue",
            include_str!(
                "../../../../../../../tests/_fixtures/differential/typechecker/legacy-typed-mixin-methods/src/Never.vue.txt"
            ),
        ),
        (
            "src/Typed.vue",
            include_str!(
                "../../../../../../../tests/_fixtures/differential/typechecker/legacy-typed-mixin-methods/src/Typed.vue.txt"
            ),
        ),
        (
            "src/Unknown.vue",
            include_str!(
                "../../../../../../../tests/_fixtures/differential/typechecker/legacy-typed-mixin-methods/src/Unknown.vue.txt"
            ),
        ),
        (
            "src/auth.ts",
            include_str!(
                "../../../../../../../tests/_fixtures/differential/typechecker/legacy-typed-mixin-methods/src/auth.ts.txt"
            ),
        ),
        (
            "src/broad.ts",
            include_str!(
                "../../../../../../../tests/_fixtures/differential/typechecker/legacy-typed-mixin-methods/src/broad.ts.txt"
            ),
        ),
        (
            "src/never.d.ts",
            include_str!(
                "../../../../../../../tests/_fixtures/differential/typechecker/legacy-typed-mixin-methods/src/never.d.ts.txt"
            ),
        ),
        (
            "src/unknown.ts",
            include_str!(
                "../../../../../../../tests/_fixtures/differential/typechecker/legacy-typed-mixin-methods/src/unknown.ts.txt"
            ),
        ),
        (
            "src/untyped.d.ts",
            include_str!(
                "../../../../../../../tests/_fixtures/differential/typechecker/legacy-typed-mixin-methods/src/untyped.d.ts.txt"
            ),
        ),
        (
            "tsconfig.json",
            include_str!(
                "../../../../../../../tests/_fixtures/differential/typechecker/legacy-typed-mixin-methods/tsconfig.json.txt"
            ),
        ),
    ];
    let root = create_project_case("legacy-typed-options-mixin-methods", &files);
    let mut checker = BatchTypeChecker::new(&root).expect("batch checker construction");
    checker.enable_legacy_vue2();
    checker.scan_project().expect("project scan");
    let result = checker.check_project().expect("native project check");
    let mut whole: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                relative_path(&root, &diagnostic.file),
                diagnostic.code,
                diagnostic.message.clone(),
                diagnostic.line + 1,
                diagnostic.column + 1,
                diagnostic.severity,
                diagnostic.block_type.map(|block| block.block_name()),
            )
        })
        .collect();
    whole.sort_by(|left, right| (&left.0, left.3, left.4).cmp(&(&right.0, right.3, right.4)));
    if let Some(capture) = std::env::var_os("VIZE_SNAPSHOT_SOURCE_CAPTURE_DIR") {
        let capture = std::path::PathBuf::from(capture).join("legacy-typed-mixin-methods");
        std::fs::create_dir_all(&capture).unwrap();
        let packet = serde_json::json!({
            "scope": "synthetic Vue2 compatibility fixture with actual configured native checker",
            "provider": provider,
            "inputs": files,
            "tsconfig": std::fs::read_to_string(root.join("tsconfig.json")).unwrap(),
            "mappedDiagnostics": whole,
        });
        std::fs::write(
            capture.join("mapped-diagnostics.json"),
            serde_json::to_vec_pretty(&packet).unwrap(),
        )
        .unwrap();
    }
    let _ = std::fs::remove_dir_all(&root);
    let missing = "Property '$missing' does not exist on type '$This'. Did you mean 'missing'?";
    let wrong = "Argument of type 'number' is not assignable to parameter of type 'string'.";
    assert_eq!(
        whole,
        vec![
            (
                "src/Any.vue".into(),
                Some(2551),
                missing.into(),
                3,
                71,
                1,
                Some("script")
            ),
            (
                "src/Broad.vue".into(),
                Some(2551),
                missing.into(),
                3,
                71,
                1,
                Some("script")
            ),
            (
                "src/Never.vue".into(),
                Some(2339),
                "Property '$unbound' does not exist on type '$This'.".into(),
                3,
                104,
                1,
                Some("script"),
            ),
            (
                "src/Typed.vue".into(),
                Some(2345),
                wrong.into(),
                7,
                33,
                1,
                Some("script")
            ),
            (
                "src/Typed.vue".into(),
                Some(2551),
                missing.into(),
                8,
                29,
                1,
                Some("script")
            ),
            (
                "src/Unknown.vue".into(),
                Some(2551),
                missing.into(),
                3,
                71,
                1,
                Some("script")
            ),
        ]
    );
}
