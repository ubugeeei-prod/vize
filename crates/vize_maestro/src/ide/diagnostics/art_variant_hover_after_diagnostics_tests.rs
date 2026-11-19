//! Art variant hovers keep imported signatures after the diagnostics pass
//! moves the native session onto the canonical project (#13383).
#![expect(
    clippy::expect_used,
    reason = "native fixture failures require context"
)]

use tower_lsp::lsp_types::{HoverContents, Url};

use super::DiagnosticService;
use super::editor_typecheck_fixture::{
    resolve_test_tsgo_binary, state_for_fixture, write_corsa_config,
};
use crate::ide::{HoverService, IdeContext};

const SOURCE: &str = r#"<script setup lang="ts">
import { imported, imported as importedAlias } from './format'
const simpleLabel: string = 'typed'
function format(value: string, precision: number): string {
  return value.repeat(precision)
}
</script>
<art title="Button" component="./Button.vue">
  <variant name="default">
    <p>{{ simpleLabel }} {{ format('art', 2) }} {{ imported(10, 16) }} {{ importedAlias(10, 16) }}</p>
  </variant>
</art>
"#;

#[test]
fn art_variant_hover_resolves_imports_after_diagnostics() {
    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        assert!(
            std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none(),
            "required native Art hover law has no configured runtime"
        );
        return;
    };
    crate::runtime::block_on(async {
        let project = crate::test_support::tempdir().expect("project dir");
        let root = project.path();
        std::fs::create_dir_all(root.join("src")).expect("src dir");
        std::fs::write(
            root.join("tsconfig.json"),
            r#"{"compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","noEmit":true},"include":["src/**/*"]}"#,
        )
        .expect("tsconfig");
        std::fs::write(
            root.join("src/format.ts"),
            "export function imported(value: number, radix: number): string { return value.toString(radix) }\n",
        )
        .expect("format module");
        std::fs::write(
            root.join("src/Button.vue"),
            "<template><button /></template>\n",
        )
        .expect("target component");
        let art = root.join("src/Button.art.vue");
        std::fs::write(&art, SOURCE).expect("art file");
        write_corsa_config(root, &corsa_path);
        let uri = Url::from_file_path(&art).expect("art uri");
        let state = state_for_fixture(root, &uri, SOURCE);
        state.load_workspace_config(root);
        state.update_virtual_docs(&uri, SOURCE);

        // The editor publishes diagnostics on open, before any hover.
        DiagnosticService::collect_async(&state, &uri).await;
        let bridge = state.get_corsa_bridge().await;
        assert!(bridge.is_some(), "the native bridge must start");

        for (marker, word, expected) in [
            ("simpleLabel }}", "simpleLabel", "const simpleLabel: string"),
            (
                "format('art'",
                "format",
                "function format(value: string, precision: number): string",
            ),
            (
                "imported(10",
                "imported",
                "var imported: (value: number, radix: number) => string",
            ),
            (
                "importedAlias(10",
                "importedAlias",
                "var importedAlias: (value: number, radix: number) => string",
            ),
        ] {
            let offset = SOURCE.rfind(marker).expect("marker") + 2;
            let ctx = IdeContext::new(&state, &uri, offset).expect("art context");
            let hover = HoverService::hover_with_corsa(&ctx, bridge.clone())
                .await
                .expect("art variant hover");
            let HoverContents::Markup(markup) = &hover.contents else {
                panic!("markdown hover expected for {word}");
            };
            assert_eq!(
                markup.value,
                vize_l0::cstr!("```typescript\n{expected}\n```").as_str(),
                "{word}"
            );
            let range = hover.range.expect("art hover range");
            let start =
                crate::ide::position_to_offset(SOURCE, range.start.line, range.start.character)
                    .expect("range start");
            let end = crate::ide::position_to_offset(SOURCE, range.end.line, range.end.character)
                .expect("range end");
            assert_eq!(
                &SOURCE[start..end],
                word,
                "hover range must cover the authored token"
            );
        }
    });
}
