//! Exact private declaration proof and frozen whole native template dispatcher.

use std::{fs, path::Path, sync::Arc};
use tower_lsp::lsp_types::{
    GotoDefinitionResponse, Location, Position, Range, TextDocumentContentChangeEvent, Url,
};
use vize_canon::{CorsaBridge, CorsaBridgeConfig};

use crate::{
    ide::{DefinitionService, IdeContext},
    server::ServerState,
};

mod original;

const LOCAL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/performance/lsp-dual-server/sources/App.v2.vue.txt"
));
const PUBLIC: &str = "<script setup lang=\"ts\">import Child from './Child.vue'; const message = 'local';</script><template><Child :message=\"message\" /></template>";
const CHILD: &str = "<script setup lang=\"ts\">defineProps<{ message: string }>();</script><template>{{ message }}</template>";

struct Case {
    id: &'static str,
    source: std::string::String,
    offset: usize,
    private: bool,
}
fn case(id: &'static str, source: &str, query: &str, delta: usize, private: bool) -> Case {
    Case {
        id,
        source: source.into(),
        offset: source.rfind(query).expect("query") + delta,
        private,
    }
}
fn cases() -> Vec<Case> {
    let crlf = LOCAL.replace('\n', "\r\n");
    let unicode = "<script setup lang=\"ts\">const before = '雪🌸'; const message = '日本語';</script><template>🚀 {{ message }}</template>";
    let mut cases = vec![
        case(
            "local-first",
            LOCAL,
            "🚀 {{ message",
            "🚀 {{ ".len() + 2,
            true,
        ),
        case("local-last", LOCAL, "message }}", 2, true),
        case(
            "crlf-first",
            &crlf,
            "🚀 {{ message",
            "🚀 {{ ".len() + 2,
            true,
        ),
        case("crlf-last", &crlf, "message }}", 2, true),
        case("same-public-prop-local-value", PUBLIC, "message\"", 2, true),
        case("unicode", unicode, "message }}", 2, true),
    ];
    for (id, script, template, suffix, query) in [
        (
            "import",
            "import { message } from './shared';",
            "{{ message }}",
            "",
            "message }}",
        ),
        (
            "props",
            "const { message } = defineProps<{ message: string }>();",
            "{{ message }}",
            "",
            "message }}",
        ),
        (
            "model",
            "const message = defineModel<string>();",
            "{{ message }}",
            "",
            "message }}",
        ),
        (
            "expose",
            "const message = 1; defineExpose({ message });",
            "{{ message }}",
            "",
            "message }}",
        ),
        (
            "emits",
            "const message = 1; defineEmits<{ save: [] }>();",
            "{{ message }}",
            "",
            "message }}",
        ),
        (
            "slots",
            "const message = 1; defineSlots<{ default(): unknown }>();",
            "{{ message }}",
            "",
            "message }}",
        ),
        (
            "options",
            "const message = 1; defineOptions({ name: 'Public' });",
            "{{ message }}",
            "",
            "message }}",
        ),
        (
            "defaults",
            "const message = 1; withDefaults(defineProps<{ x?: number }>(), { x: 1 });",
            "{{ message }}",
            "",
            "message }}",
        ),
        (
            "export",
            "export const message = 1;",
            "{{ message }}",
            "",
            "message }}",
        ),
        (
            "export-list",
            "const message = 1; export { message };",
            "{{ message }}",
            "",
            "message }}",
        ),
        (
            "pattern",
            "const { message } = store;",
            "{{ message }}",
            "",
            "message }}",
        ),
        (
            "object-shorthand",
            "const message = 1; const value = { message };",
            "{{ message }}",
            "",
            "message }}",
        ),
        (
            "computed",
            "const message = 1; const value = { ['x']: message };",
            "{{ message }}",
            "",
            "message }}",
        ),
        (
            "member",
            "const message = 1; const value = store.message;",
            "{{ message }}",
            "",
            "message }}",
        ),
        (
            "capture",
            "const message = 1; const f = () => message;",
            "{{ message }}",
            "",
            "message }}",
        ),
        (
            "shadow",
            "const message = 1; const f = (message: number) => message;",
            "{{ message }}",
            "",
            "message }}",
        ),
        ("let", "let message = 1;", "{{ message }}", "", "message }}"),
        (
            "assignment",
            "const message = 1; message = 2;",
            "{{ message }}",
            "",
            "message }}",
        ),
        (
            "update",
            "const message = 1; message++;",
            "{{ message }}",
            "",
            "message }}",
        ),
        (
            "attribute-shorthand",
            "const message = 1;",
            "<Child :message />",
            "",
            "message />",
        ),
        (
            "public-property-key",
            "const message = 1;",
            "<Child :message=\"message\" />",
            "",
            "message=",
        ),
        (
            "template-shadow",
            "const message = 1;",
            "<i v-for=\"message in [1]\">{{ message }}</i>",
            "",
            "message }}",
        ),
        (
            "invalid",
            "const message = ;",
            "{{ message }}",
            "",
            "message }}",
        ),
        (
            "dual-script",
            "const message = 1;",
            "{{ message }}",
            "<script>export default {}</script>",
            "message }}",
        ),
        (
            "style",
            "const message = 1;",
            "{{ message }}",
            "<style>.x{color:v-bind(message)}</style>",
            "message }}",
        ),
        (
            "art",
            "const message = 1;",
            "{{ message }}",
            "<art title=\"A\" component=\"./Child.vue\"><variant name=\"V\">{{ message }}</variant></art>",
            "message }}</template>",
        ),
    ] {
        let source = vize_l0::cstr!(
            "<script setup lang=\"ts\">{script}</script><template>{template}</template>{suffix}"
        );
        cases.push(case(id, &source, query, 2, false));
    }
    cases.push(case(
        "token-end",
        LOCAL,
        "message }}",
        "message".len(),
        false,
    ));
    cases.push(case("non-token", LOCAL, "🚀", 0, false));
    cases
}
fn project(source: &str) -> (tempfile::TempDir, Url) {
    let directory = crate::test_support::tempdir().expect("physical project");
    for (name, contents) in [
        ("App.vue", source),
        ("Child.vue", CHILD),
        ("shared.ts", "export const message = 1;"),
        (
            "tsconfig.json",
            r#"{"compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","noEmit":true},"include":["./**/*"]}"#,
        ),
    ] {
        fs::write(directory.path().join(name), contents).expect("authored input");
    }
    let uri = Url::from_file_path(directory.path().join("App.vue")).expect("source URI");
    (directory, uri)
}
fn opened(root: &Path, uri: &Url, source: &str) -> ServerState {
    let state = ServerState::new();
    state.set_workspace_root(root.to_path_buf());
    let child = Url::from_file_path(root.join("Child.vue")).expect("child URI");
    for (uri, text) in [(uri, source), (&child, CHILD)] {
        state
            .documents
            .open(uri.clone(), text.into(), 1, "vue".into());
        state.update_virtual_docs(uri, text);
    }
    state
}
fn expected(uri: &Url, source: &str) -> GotoDefinitionResponse {
    let start = source.find("const message").expect("declaration") + "const ".len();
    let (line, character) = crate::ide::offset_to_position(source, start);
    let (end_line, end_character) = crate::ide::offset_to_position(source, start + "message".len());
    GotoDefinitionResponse::Scalar(Location {
        uri: uri.clone(),
        range: Range::new(
            Position::new(line, character),
            Position::new(end_line, end_character),
        ),
    })
}
#[test]
fn private_template_definition_requires_exact_declaration_and_half_open_query() {
    let cases = cases();
    assert_eq!(cases.len(), 34);
    for case in cases {
        let (directory, uri) = project(&case.source);
        let state = opened(directory.path(), &uri, &case.source);
        let ctx = IdeContext::new(&state, &uri, case.offset).expect("query context");
        let actual = DefinitionService::private_template_declaration(&ctx);
        assert_eq!(
            actual.is_some(),
            case.private,
            "{}: {}",
            case.id,
            case.source
        );
        if case.private {
            assert_eq!(actual, Some(expected(&uri, &case.source)));
        }
        assert!(!state.has_corsa_bridge());
    }
}
#[test]
fn private_template_definition_refuses_stale_foreign_and_provider_uncertain_owners() {
    let (directory, uri) = project(LOCAL);
    let state = opened(directory.path(), &uri, LOCAL);
    let offset = LOCAL.rfind("message }}").expect("template value");
    let ctx = IdeContext::new(&state, &uri, offset).expect("old context");
    assert_eq!(
        DefinitionService::private_template_declaration(&ctx),
        Some(expected(&uri, LOCAL))
    );
    assert_eq!(
        DefinitionService::private_template_definition(&ctx, None),
        None
    );
    let uninitialized = CorsaBridge::default();
    assert_eq!(
        DefinitionService::private_template_definition(&ctx, Some(&uninitialized)),
        None
    );
    let foreign = Url::parse("file:///another-owner/App.vue").expect("foreign URI");
    let mut wrong_uri = IdeContext::new(&state, &uri, offset).expect("current context");
    wrong_uri.uri = &foreign;
    assert_eq!(
        DefinitionService::private_template_declaration(&wrong_uri),
        None
    );
    let declaration = IdeContext::new(&state, &uri, LOCAL.find("message =").expect("declaration"))
        .expect("script context");
    assert_eq!(
        DefinitionService::private_template_declaration(&declaration),
        None
    );
    state.documents.apply_changes(
        &uri,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: LOCAL.replace("hello", "dirty"),
        }],
        2,
    );
    assert_eq!(DefinitionService::private_template_declaration(&ctx), None);
    state.documents.close(&uri);
    assert_eq!(DefinitionService::private_template_declaration(&ctx), None);
    state
        .documents
        .open(uri.clone(), LOCAL.into(), 1, "vue".into());
    state.update_virtual_docs(&uri, LOCAL);
    assert_eq!(DefinitionService::private_template_declaration(&ctx), None);
    let current = IdeContext::new(&state, &uri, offset).expect("reopened owner");
    assert_eq!(
        DefinitionService::private_template_declaration(&current),
        Some(expected(&uri, LOCAL))
    );
    let other = crate::test_support::tempdir().expect("foreign project");
    state.set_workspace_root(other.path().to_path_buf());
    assert_eq!(
        DefinitionService::private_template_declaration(&current),
        None
    );
    state.set_workspace_root(directory.path().to_path_buf());
    state.documents.close(&uri);
    state
        .documents
        .open(uri.clone(), LOCAL.into(), 1, "typescript".into());
    let other_language = IdeContext::new(&state, &uri, offset).expect("other language");
    assert_eq!(
        DefinitionService::private_template_declaration(&other_language),
        None
    );
}
async fn native_packet(
    root: &Path,
    uri: &Url,
    case: &Case,
    provider: &Path,
    original: bool,
) -> (Option<GotoDefinitionResponse>, Option<bool>) {
    let state = opened(root, uri, &case.source);
    let bridge = Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(provider.to_path_buf()),
        working_dir: Some(root.to_path_buf()),
        ..Default::default()
    }));
    bridge.spawn().await.expect("actual pinned native owner");
    let ctx = IdeContext::new(&state, uri, case.offset).expect("native template context");
    assert!(matches!(
        ctx.block_type,
        Some(crate::virtual_code::BlockType::Template)
    ));
    let packet = if original {
        DefinitionService::original_template_definition(&ctx, Some(Arc::clone(&bridge))).await
    } else {
        DefinitionService::definition_with_corsa(&ctx, Some(Arc::clone(&bridge))).await
    };
    let admitted = (!original).then(|| {
        DefinitionService::private_template_definition(&ctx, Some(bridge.as_ref())).is_some()
    });
    bridge.shutdown().await.expect("native shutdown");
    (packet, admitted)
}
#[test]
fn private_template_definition_matches_the_complete_frozen_native_dispatcher() {
    let Some(provider) = crate::test_support::tsgo() else {
        assert!(
            std::env::var_os("VIZE_TEST_DISABLE_TSGO")
                .is_some_and(|value| !value.is_empty() && value != "false"),
            "native provider mandatory outside explicit PR-disabled envelope"
        );
        eprintln!(
            "VIZE_PRIVATE_DEFINITION_NATIVE_DISABLED explicit PR-disabled envelope; native qualification=false"
        );
        return;
    };
    crate::runtime::block_on(async {
        for (index, case) in cases().into_iter().enumerate() {
            let (directory, uri) = project(&case.source);
            let (old, old_admitted) =
                native_packet(directory.path(), &uri, &case, &provider, true).await;
            let (actual, admitted) =
                native_packet(directory.path(), &uri, &case, &provider, false).await;
            eprintln!(
                "VIZE_PRIVATE_DEFINITION_NATIVE {}",
                serde_json::json!({
                    "case": index, "id": case.id, "source": case.source, "offset": case.offset,
                    "private": case.private, "admitted": admitted, "old": old, "actual": actual,
                })
            );
            assert_eq!(old_admitted, None);
            assert_eq!(admitted, Some(case.private));
            if case.private {
                assert_eq!(
                    actual,
                    Some(expected(&uri, &case.source)),
                    "positive complete Scalar"
                );
            }
            assert_eq!(actual, old, "whole Option<GotoDefinitionResponse> {index}");
        }
    });
}
