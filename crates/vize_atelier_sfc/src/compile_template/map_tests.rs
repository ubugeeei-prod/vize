use super::{TemplateBlockCompileContext, compile_template_block};
use crate::types::{BlockLocation, SfcTemplateBlock, TemplateCompileOptions};
use std::borrow::Cow;

fn template_block(source: &str) -> SfcTemplateBlock<'_> {
    SfcTemplateBlock {
        content: Cow::Borrowed(source),
        loc: BlockLocation {
            start: 0,
            end: source.len(),
            tag_start: 0,
            tag_end: source.len(),
            start_line: 1,
            start_column: 1,
            end_line: 1,
            end_column: source.len() + 1,
        },
        lang: None,
        src: None,
        attrs: Default::default(),
    }
}

fn compile_with_source_map(source_map: bool) -> super::TemplateBlockCompileResult {
    let template = template_block("<div>{{ msg }}</div>");
    let options = TemplateCompileOptions {
        compiler_options: Some(vize_atelier_dom::DomCompilerOptions {
            source_map,
            ..Default::default()
        }),
        ..Default::default()
    };

    compile_template_block(
        &template,
        &options,
        TemplateBlockCompileContext {
            scope_id: "abc123",
            apply_scope_id: false,
            has_scoped: false,
            is_ts: false,
            inline: false,
            component_name: Some("MapProbe"),
            bindings: None,
            croquis: None,
        },
        vize_atelier_core::TemplateSyntaxMode::Standard,
    )
    .expect("template should compile")
}

#[test]
fn dom_template_carries_source_map_fragment_without_changing_code() {
    let with_map = compile_with_source_map(true);
    let without_map = compile_with_source_map(false);

    assert_eq!(
        with_map.code, without_map.code,
        "source-map registration must not change generated code"
    );
    assert!(
        without_map.maps.source_map().is_none(),
        "source maps should remain absent by default"
    );

    let map = with_map
        .maps
        .source_map()
        .expect("DOM Atelier map fragment should be retained");
    let parsed: serde_json::Value =
        serde_json::from_str(map).expect("source map fragment should be valid JSON");

    assert_eq!(parsed["version"], 3);
    assert_eq!(parsed["sources"][0], "template.vue");
}
