use super::*;
use crate::server::ServerState;
use tower_lsp::lsp_types::Url;

const SOURCE: &str = r#"<script setup lang="ts">
const ordinary = 1
const rows = 'outer'
const result = { rows: [1, 2] }
</script>
<template>
  <section v-match="result">
    <template v-when="{ const rows } if (rows.length > 0)">
      <p v-for="row in rows">{{ row.toFixed() }}{{ rows.length }}</p>
    </template>
    <p v-when="_"/>
  </section>
  <p>{{ ordinary }}</p>
</template>"#;

fn enabled_state() -> (tempfile::TempDir, ServerState) {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vize.config.json"),
        r#"{"experimentals":{"patternedTemplate":true}}"#,
    )
    .unwrap();
    let state = ServerState::new();
    state.load_workspace_config(project.path());
    (project, state)
}

#[test]
fn classification_separates_local_shared_and_unaffected_symbols() {
    let (project, state) = enabled_state();
    let uri = Url::from_file_path(project.path().join("App.vue")).unwrap();
    for (needle, expected) in [
        ("ordinary =", Navigation::Ordinary),
        ("ordinary }}", Navigation::Ordinary),
        ("rows =", Navigation::Shared),
        ("result =", Navigation::Shared),
        ("result\">", Navigation::Shared),
        ("rows } if", Navigation::Local),
        ("rows.length >", Navigation::Local),
        ("rows.length }}", Navigation::Local),
        ("rows\">", Navigation::Local),
        ("row.toFixed", Navigation::Local),
        ("row in", Navigation::Local),
    ] {
        let ctx =
            IdeContext::testing(&state, &uri, SOURCE.find(needle).unwrap(), SOURCE.into());
        assert_eq!(classify(&ctx), expected, "{needle}");
    }
    let unicode = SOURCE.replace("rows", "\u{6570}\u{5024}");
    for (needle, expected) in [
        ("\u{6570}\u{5024} =", Navigation::Shared),
        ("\u{6570}\u{5024}.length }}", Navigation::Local),
    ] {
        let ctx =
            IdeContext::testing(&state, &uri, unicode.find(needle).unwrap(), unicode.clone());
        assert_eq!(classify(&ctx), expected);
    }
    let malformed = SOURCE.replace("{ const rows }", "{ const rows, const rows }");
    let property = SOURCE.replace("{{ rows.length }}", "{{ rows.row }}");
    let ctx = IdeContext::testing(&state, &uri, property.find("row }}").unwrap(), property);
    assert_eq!(classify(&ctx), Navigation::Shared);
    let ctx = IdeContext::testing(
        &state,
        &uri,
        malformed.find("rows.length }}").unwrap(),
        malformed,
    );
    assert_eq!(classify(&ctx), Navigation::Shared);
}

#[test]
fn opt_out_and_pattern_like_text_do_not_change_ordinary_routing() {
    let (project, enabled) = enabled_state();
    let disabled = ServerState::new();
    let uri = Url::from_file_path(project.path().join("App.vue")).unwrap();
    let ctx = IdeContext::testing(
        &disabled,
        &uri,
        SOURCE.find("rows.length }}").unwrap(),
        SOURCE.into(),
    );
    assert_eq!(classify(&ctx), Navigation::Ordinary);
    let source = "<script setup>const value = '<template v-match>'; </script><template><!-- v-match=\"value\" -->{{ value }}</template>";
    let ctx = IdeContext::testing(
        &enabled,
        &uri,
        source.find("value }}").unwrap(),
        source.into(),
    );
    assert_eq!(classify(&ctx), Navigation::Ordinary);
    let uri = Url::from_file_path(project.path().join("plain.ts")).unwrap();
    let ctx = IdeContext::testing(&enabled, &uri, 6, "const value = 1".into());
    assert_eq!(classify(&ctx), Navigation::Ordinary);
}

#[cfg(feature = "native")]
#[test]
fn experimental_opt_in_keeps_ordinary_reference_and_rename_fallbacks() {
    use crate::ide::{ReferencesService, RenameService};
    crate::runtime::block_on(async {
        let (project, enabled) = enabled_state();
        let disabled = ServerState::new();
        let uri = Url::from_file_path(project.path().join("App.vue")).unwrap();
        for source in [
            SOURCE,
            "<script setup>const ordinary = 1</script><template>{{ ordinary }}</template>",
        ] {
            let offset = source.find("ordinary }}").unwrap();
            for state in [&disabled, &enabled] {
                state
                    .documents
                    .open(uri.clone(), source.into(), 1, "vue".into());
                state.update_virtual_docs(&uri, source);
            }
            let baseline = IdeContext::new(&disabled, &uri, offset).unwrap();
            let ctx = IdeContext::new(&enabled, &uri, offset).unwrap();
            let references = ReferencesService::references(&baseline, true).unwrap();
            assert_eq!(references.len(), 2);
            let rename = RenameService::rename(&baseline, "changed").unwrap();
            let prepare = RenameService::prepare_rename(&baseline).unwrap();
            assert_eq!(
                ReferencesService::references(&ctx, true),
                Some(references.clone())
            );
            assert_eq!(RenameService::rename(&ctx, "changed"), Some(rename.clone()));
            for bridge in [
                None,
                Some(std::sync::Arc::new(vize_canon::CorsaBridge::new())),
            ] {
                assert_eq!(
                    ReferencesService::references_with_corsa(&ctx, true, bridge.clone()).await,
                    Some(references.clone())
                );
                assert_eq!(
                    RenameService::prepare_rename_with_corsa(&ctx, bridge.clone()).await,
                    Some(prepare.clone())
                );
                assert_eq!(
                    RenameService::rename_with_corsa(&ctx, "changed", bridge).await,
                    Some(rename.clone())
                );
            }
        }
    });
}
