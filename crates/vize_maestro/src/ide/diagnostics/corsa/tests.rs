use super::mapping::{line_character_to_byte_offset, source_offset_to_position};

#[test]
fn line_character_to_byte_offset_counts_utf16_code_units() {
    let source = "const icon = \"😀\";\nconst message = icon";

    assert_eq!(
        line_character_to_byte_offset(source, 0, 16),
        Some("const icon = \"😀".len())
    );
    assert_eq!(
        line_character_to_byte_offset(source, 1, 6),
        Some(source.find("message").unwrap())
    );
}

#[test]
fn line_character_to_byte_offset_rejects_surrogate_pair_interior() {
    let source = "a😀b";

    assert_eq!(line_character_to_byte_offset(source, 0, 2), None);
}

#[test]
fn source_offset_to_position_counts_utf16_code_units() {
    let source = "const icon = \"😀\"; missing";
    let offset = source.find("missing").unwrap();

    assert_eq!(source_offset_to_position(source, offset), (0, 19));
}

/// Issue #752: editor-side virtual TS generation must rewrite `.vue`
/// import specifiers to `.vue.ts` so the Corsa session can resolve
/// siblings via the virtual mirror — alias *and* relative specifiers
/// both get rewritten, mirroring the batch pipeline.
#[test]
fn editor_virtual_ts_rewrites_dot_vue_imports() {
    use crate::DiagnosticService;
    use tower_lsp::lsp_types::Url;

    let uri = Url::parse("file:///tmp/Host.vue").expect("parse uri");
    let content = "<script setup lang=\"ts\">\n\
                   import App from './app.vue'\n\
                   import Sibling from '../shared/Sib.vue'\n\
                   import Aliased from '@/Alias.vue'\n\
                   import { ref } from 'vue'\n\
                   const _u = App\n\
                   const _v = Sibling\n\
                   const _w = Aliased\n\
                   const _r = ref(0)\n\
                   </script>\n\
                   <template><div></div></template>";

    let result = DiagnosticService::generate_virtual_ts(&uri, content, false, false)
        .expect("virtual ts generated");

    assert!(
        !result.code.contains("'./app.vue'"),
        "expected relative .vue import to be rewritten, got:\n{}",
        result.code,
    );
    assert!(
        result.code.contains("'./app.vue.ts'"),
        "expected rewritten relative specifier, got:\n{}",
        result.code,
    );
    assert!(
        result.code.contains("'../shared/Sib.vue.ts'"),
        "expected rewritten parent-path specifier, got:\n{}",
        result.code,
    );
    assert!(
        result.code.contains("'@/Alias.vue.ts'"),
        "expected rewritten alias specifier, got:\n{}",
        result.code,
    );
    // Only relative specifiers feed the sibling overlay; alias and bare
    // imports are excluded since they resolve via tsconfig paths and the
    // ambient stub respectively.
    assert!(
        result.relative_vue_imports.iter().any(|s| s == "./app.vue"),
        "expected ./app.vue in relative_vue_imports, got {:?}",
        result.relative_vue_imports,
    );
    assert!(
        result
            .relative_vue_imports
            .iter()
            .any(|s| s == "../shared/Sib.vue"),
        "expected ../shared/Sib.vue in relative_vue_imports, got {:?}",
        result.relative_vue_imports,
    );
    assert!(
        !result
            .relative_vue_imports
            .iter()
            .any(|s| s == "@/Alias.vue"),
        "alias specifier must not appear in relative_vue_imports",
    );
}

#[test]
fn editor_virtual_ts_for_inline_art_binds_self_to_host_props() {
    use crate::DiagnosticService;
    use tower_lsp::lsp_types::Url;

    let uri = Url::parse("file:///tmp/Button.vue").expect("parse uri");
    let content = r#"<script setup lang="ts">
defineProps<{ variant?: "primary" | "secondary" }>();
</script>

<template><button /></template>

<art>
  <variant name="Primary" default>
    <Self :variant="123" />
  </variant>
</art>"#;

    let results =
        DiagnosticService::generate_virtual_ts_for_inline_art_variants(&uri, content, false, false);

    assert_eq!(results.len(), 1);
    let (_, result) = &results[0];
    assert!(
        result
            .code
            .contains("declare const Self: { new (): { $props: Props } };"),
        "expected Self component binding, got:\n{}",
        result.code,
    );
    assert!(
        result.code.contains("type __Self_Props_0 = typeof Self"),
        "expected Self prop checks, got:\n{}",
        result.code,
    );
}

#[test]
fn editor_virtual_ts_for_art_imports_define_art_target_component() {
    use crate::DiagnosticService;
    use tower_lsp::lsp_types::Url;

    let uri = Url::parse("file:///tmp/Button.art.vue").expect("parse uri");
    let content = r#"<script setup lang="ts">
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="Primary" default>
    <Button :variant="123" />
  </variant>
</art>"#;

    let result = DiagnosticService::generate_virtual_ts_for_art(&uri, content)
        .expect("virtual TS generated");

    assert!(
        result
            .code
            .contains("import __VizeArtTarget_Button from \"./Button.vue.ts\";"),
        "expected defineArt component import, got:\n{}",
        result.code,
    );
    assert!(
        result
            .relative_vue_imports
            .iter()
            .any(|s| s == "./Button.vue"),
        "expected ./Button.vue to be overlaid, got {:?}",
        result.relative_vue_imports,
    );
    assert!(
        result
            .code
            .contains("type __Button_Props_0 = typeof Button"),
        "expected Button prop checks, got:\n{}",
        result.code,
    );
}
