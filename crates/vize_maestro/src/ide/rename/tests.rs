#![expect(clippy::string_slice, reason = "tests assert by panicking")]
use super::RenameService;
use tower_lsp::lsp_types::{Position, Range};

/// Offset helpers the rename tests probe; the service itself works on
/// references-provider spans.
impl RenameService {
    fn get_word_at_offset(content: &str, offset: usize) -> Option<String> {
        crate::ide::token_at_offset(content, offset, |c| Self::is_ident_char(c as char))
    }

    fn offset_range_to_lsp(content: &str, start: usize, end: usize) -> Range {
        Range {
            start: Self::offset_to_position(content, start),
            end: Self::offset_to_position(content, end),
        }
    }

    fn offset_to_position(content: &str, offset: usize) -> Position {
        crate::utils::offset_to_position_str(content, offset)
    }

    fn is_ident_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || c == '$'
    }
}

/// Renaming a local binding must not rewrite a same-named prop
/// *attribute name* on a component usage — that token belongs to the
/// child's own prop symbol, and rewriting it silently breaks the call
/// site (#3892). Building rename from the references provider makes the
/// invariant structural: every edit is a reference (or the declaration).
#[test]
fn rename_leaves_same_named_prop_attribute_names_alone() {
    let content = r#"<script setup lang="ts">
import Child from './Child.vue'
const count = 1
</script>

<template>
  <Child :count="count" />
</template>
"#;
    let uri = tower_lsp::lsp_types::Url::parse("file:///ws/Parent.vue").unwrap();
    let state = crate::server::ServerState::new();
    state
        .documents
        .open(uri.clone(), content.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, content);
    let offset = content.find("const count").unwrap() + "const ".len();
    let ctx = crate::ide::IdeContext::new(&state, &uri, offset).unwrap();

    let edit = RenameService::rename(&ctx, "tally").unwrap();
    let edits = &edit.changes.unwrap()[&uri];

    let attr_name_line = content[..content.find(":count=").unwrap()]
        .matches('\n')
        .count() as u32;
    for edit in edits {
        let touches_attr_name = edit.range.start.line == attr_name_line
            && edit.range.start.character
                == content
                    .lines()
                    .nth(attr_name_line as usize)
                    .unwrap()
                    .find(":count")
                    .unwrap() as u32
                    + 1;
        assert!(
            !touches_attr_name,
            "the :count attribute name must not be renamed: {edits:?}"
        );
    }

    // The invariant that pins the providers together: every rename edit is
    // one of the reference locations.
    let references = crate::ide::references::ReferencesService::references(&ctx, true).unwrap();
    for edit in edits {
        assert!(
            references
                .iter()
                .any(|reference| reference.range == edit.range),
            "rename edit {edit:?} is not a reference: {references:?}"
        );
    }
    assert!(
        edits.len() >= 2,
        "the declaration and the binding value must still rename: {edits:?}"
    );
}

#[test]
fn test_get_word_at_offset() {
    let content = "const count = ref(0)";
    assert_eq!(
        RenameService::get_word_at_offset(content, 6),
        Some("count".to_string())
    );
    assert_eq!(
        RenameService::get_word_at_offset(content, 14),
        Some("ref".to_string())
    );
    assert_eq!(
        RenameService::get_word_at_offset(content, 11),
        Some("count".to_string())
    );
    assert_eq!(RenameService::get_word_at_offset(content, 12), None);
}

#[test]
fn test_is_valid_identifier() {
    assert!(RenameService::is_valid_identifier("count"));
    assert!(RenameService::is_valid_identifier("_private"));
    assert!(RenameService::is_valid_identifier("$refs"));
    assert!(!RenameService::is_valid_identifier("123abc"));
    assert!(!RenameService::is_valid_identifier(""));
}

#[test]
fn test_is_keyword() {
    assert!(RenameService::is_keyword("const"));
    assert!(RenameService::is_keyword("function"));
    assert!(!RenameService::is_keyword("count"));
}

#[test]
fn test_offset_range_to_lsp_counts_utf16_code_units() {
    let content = "const emoji = \"😀\"; const message = ref(emoji)";
    let start = content.find("message").unwrap();
    let end = start + "message".len();
    let range = RenameService::offset_range_to_lsp(content, start, end);

    assert_eq!(range.start.line, 0);
    assert_eq!(range.start.character, 26);
    assert_eq!(range.end.character, 33);
}

#[cfg(feature = "native")]
mod corsa_options_api_tests {
    #![expect(clippy::string_slice, reason = "tests assert by panicking")]
    use std::{fs, sync::Arc};

    use tower_lsp::lsp_types::{PrepareRenameResponse, Range, Url};
    use vize_canon::{CorsaBridge, CorsaBridgeConfig};

    use super::RenameService;
    use crate::{ide::IdeContext, server::ServerState};

    #[test]
    fn canonical_options_api_template_rename_includes_authored_declaration() {
        crate::runtime::block_on(async {
            let Some(tsgo_path) = resolve_tsgo_binary() else {
                return;
            };
            let project = tempfile::TempDir::new().expect("temp project");
            let project_root = project
                .path()
                .canonicalize()
                .expect("canonical project root");
            let src = project_root.join("src");
            fs::create_dir_all(&src).expect("src");
            fs::write(
                project_root.join("tsconfig.json"),
                r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
            )
            .expect("tsconfig");

            let source = r#"<template>
  <div :style="sliderLeft + '; width: 50%;'"></div>
</template>
<script lang="ts">
export default {
  computed: {
    sliderLeft () {
      return 'left: 0%'
    }
  }
}
</script>
"#;
            let path = src.join("Tab.vue");
            fs::write(&path, source).expect("source");
            let uri = Url::from_file_path(&path).expect("uri");
            let state = ServerState::new();
            state.set_workspace_root(project_root.clone());
            state
                .documents
                .open(uri.clone(), source.to_string(), 1, "vue".to_string());
            state.update_virtual_docs(&uri, source);
            let bridge = Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
                corsa_path: Some(tsgo_path),
                working_dir: Some(project_root),
                timeout_ms: 30_000,
                ..Default::default()
            }));
            bridge.spawn().await.expect("tsgo session");

            let query_offset = source.find("sliderLeft +").expect("template binding") + 1;
            let ctx = IdeContext::new(&state, &uri, query_offset).expect("context");
            let prepare = RenameService::prepare_rename_with_corsa(&ctx, Some(Arc::clone(&bridge)))
                .await
                .expect("prepare rename");
            assert_eq!(authored_text(source, prepare_range(prepare)), "sliderLeft");

            let edit = RenameService::rename_with_corsa(
                &ctx,
                "renamedSliderLeft",
                Some(Arc::clone(&bridge)),
            )
            .await
            .expect("rename");
            bridge.shutdown().await.expect("shutdown");

            let changes = edit.changes.expect("plain workspace changes");
            assert_eq!(changes.keys().collect::<Vec<_>>(), [&uri]);
            let edits = changes.get(&uri).expect("SFC edits");
            assert_eq!(edits.len(), 2, "template and declaration edits: {edits:#?}");
            assert!(
                edits
                    .iter()
                    .all(|edit| edit.new_text == "renamedSliderLeft")
            );
            assert!(
                edits
                    .iter()
                    .any(|edit| authored_text(source, edit.range) == "sliderLeft"
                        && edit.range.start.line == 1),
                "missing template edit: {edits:#?}"
            );
            assert!(
                edits
                    .iter()
                    .any(|edit| authored_text(source, edit.range) == "sliderLeft"
                        && edit.range.start.line == 6),
                "missing Options API declaration edit: {edits:#?}"
            );
        });
    }

    fn prepare_range(response: PrepareRenameResponse) -> Range {
        match response {
            PrepareRenameResponse::Range(range)
            | PrepareRenameResponse::RangeWithPlaceholder { range, .. } => range,
            PrepareRenameResponse::DefaultBehavior { .. } => panic!("expected an authored range"),
        }
    }

    fn authored_text(source: &str, range: Range) -> &str {
        let start = crate::ide::position_to_offset(source, range.start.line, range.start.character)
            .expect("source start");
        let end = crate::ide::position_to_offset(source, range.end.line, range.end.character)
            .expect("source end");
        &source[start..end]
    }

    fn resolve_tsgo_binary() -> Option<std::path::PathBuf> {
        if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
            return None;
        }
        let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)?;
        vize_carton::corsa_resolver::resolve_corsa_executable(
            vize_carton::corsa_resolver::CorsaResolveRequest {
                project_root: Some(workspace_root),
                ..Default::default()
            },
        )
        .ok()
    }
}
