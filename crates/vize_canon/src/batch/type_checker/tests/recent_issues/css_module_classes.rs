//! Authored CSS-module classes are closed types when the module is static (#3741).

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

/// `vue-tsc 3.3.4 --noEmit` reports no diagnostics for this byte-identical SFC:
/// its CSS-module surface remains `Record<string, string>` on both sides. Vize
/// is intentionally stricter when it can prove a closed inline export shape.
/// The shared compat ledger cannot encode this as an expected pair because its
/// comparator deliberately rejects one-sided diagnostics, so the full oracle
/// is pinned here instead: exact count, code, message, authored line and column.
#[test]
fn static_default_and_named_modules_reject_only_misspelled_classes() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "css-module-authored-classes",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
import { useCssModule as cssModule } from "vue";
const styles = cssModule();
const tokenStyles = cssModule("tokens");
void styles.root;
void styles.typoed;
void tokenStyles.active;
void tokenStyles.missing;
</script>

<template>
  <div :class="$style.root" />
  <div :class="$style.typoed" />
  <div :class="tokens.active" />
  <div :class="tokens.missing" />
</template>

<style module>
.root, .row { display: flex; }
</style>

<style module="tokens">
.active { color: green; }
</style>
"#,
        )],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let snapshot = snapshot.expect("type-check CSS-module project");

    assert_eq!(
        snapshot,
        vec![
            (
                "src/App.vue".into(),
                Some(2339),
                "13:23:error Property 'typoed' does not exist on type '{ readonly root: string; readonly row: string; }'.".into(),
            ),
            (
                "src/App.vue".into(),
                Some(2339),
                "15:23:error Property 'missing' does not exist on type '{ readonly active: string; }'.".into(),
            ),
            (
                "src/App.vue".into(),
                Some(2339),
                "6:13:error Property 'typoed' does not exist on type '{ readonly root: string; readonly row: string; }'.".into(),
            ),
            (
                "src/App.vue".into(),
                Some(2339),
                "8:18:error Property 'missing' does not exist on type '{ readonly active: string; }'.".into(),
            ),
        ],
        "correct classes stay clean while script and template typos keep exact authored ranges"
    );
}

#[test]
fn unresolved_modules_keep_the_index_signature_fallback() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "css-module-dynamic-fallback",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
import { useCssModule } from "vue";
const styles = useCssModule();
const tokens = useCssModule("tokens");
const scss = useCssModule("scss");
void styles.fromExternalFile;
void tokens.fromImportedCss;
void scss.generatedByPreprocessor;
</script>

<template>
  <div :class="$style.fromExternalFile" />
  <div :class="tokens.fromImportedCss" />
  <div :class="scss.generatedByPreprocessor" />
</template>

<style module src="./external.css"></style>
<style module="tokens">
@import "./tokens.css";
.local { color: green; }
</style>
<style module="scss" lang="scss">
.root { &__child { color: green; } }
</style>
"#,
        )],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let snapshot = snapshot.expect("type-check unresolved CSS-module project");

    assert_eq!(
        snapshot,
        Vec::new(),
        "external and imported modules must not gain false-positive property errors"
    );
}
