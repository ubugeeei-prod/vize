//! Typed `mixins:` and `extends:` members belong to legacy `__VizeThis` (#3609).

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
