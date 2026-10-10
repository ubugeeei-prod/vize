use tower_lsp::lsp_types::Url;

use super::{
    CanonicalVirtualDocument, direct_model_position, has_model_event, model_declaration_ranges,
    model_linked_semantic_positions, owns_declaration,
};
use crate::ide::{DiagnosticService, IdeContext};
use crate::server::ServerState;

#[test]
fn only_retained_model_declarations_own_inner_or_matching_quoted_ranges() {
    let source = "<script setup lang=\"ts\">\ndefineModel<string>('saveItem');\nconst unrelated = 'saveItem';\n</script>";
    let start = source.find("'saveItem'").unwrap();
    let declarations = model_declaration_ranges(source, "Child.vue");
    assert_eq!(declarations, vec![start + 1..start + 9]);
    assert!(owns_declaration(
        source,
        &declarations[0],
        &(start + 1..start + 9)
    ));
    assert!(owns_declaration(
        source,
        &declarations[0],
        &(start..start + 10)
    ));
    assert!(!owns_declaration(
        source,
        &declarations[0],
        &(start + 1..start + 10)
    ));
    let unrelated = source.rfind("'saveItem'").unwrap();
    assert!(!owns_declaration(
        source,
        &declarations[0],
        &(unrelated + 1..unrelated + 9)
    ));
    for script in [
        "defineModel<string>();",
        "defineModel<string>('save\\u0049tem');",
    ] {
        let source = vize_l0::cstr!("<script setup lang=\"ts\">{script}</script>");
        assert!(model_declaration_ranges(&source, "Child.vue").is_empty());
    }
}

#[test]
fn model_prop_locations_follow_the_distinct_native_update_identity() {
    for (name, quote) in ["x", "title", "valueId", "saveItem"]
        .into_iter()
        .flat_map(|name| ['\'', '"'].map(|quote| (name, quote)))
    {
        let literal = vize_l0::cstr!("{quote}{name}{quote}");
        let source = vize_l0::cstr!(
            "<script setup lang=\"ts\">\nexport interface Props {{ value?: string }}\ndefineProps<Props>();\nconst prefix = '🌸'; defineModel<string>({literal}, {{ default: \"\" }});\n</script>\n<template><span /></template>"
        );
        let uri = Url::parse("file:///workspace/Child.vue").unwrap();
        let state = ServerState::new();
        let ctx = IdeContext::testing(&state, &uri, 0, source.as_str().to_owned());
        let result = DiagnosticService::generate_virtual_ts(&uri, &source, false, false).unwrap();
        let authored = source.find(literal.as_str()).unwrap();
        let declaration = authored + 1..authored + 1 + name.len();
        assert!(has_model_event(&result, &declaration));
        assert!(has_model_event(
            &result,
            &(authored..authored + literal.len())
        ));
        assert!(!has_model_event(
            &result,
            &(authored + 1..authored + literal.len())
        ));
        let prop = result
            .code
            .find(vize_l0::cstr!("  \"{name}\"").as_str())
            .unwrap()
            + 2;
        let event = result
            .code
            .find(vize_l0::cstr!("/* __vize_model_event */ \"update:{name}\"").as_str())
            .unwrap()
            + "/* __vize_model_event */ \"update:".len();
        let (line, character) = crate::ide::offset_to_position(&result.code, prop);
        let (end_line, end_character) =
            crate::ide::offset_to_position(&result.code, prop + name.len() + 2);
        let range = vize_canon::LspRange {
            start: vize_canon::LspPosition { line, character },
            end: vize_canon::LspPosition {
                line: end_line,
                character: end_character,
            },
        };
        let expected_prop = crate::ide::offset_to_position(&result.code, prop + 1);
        let expected_event = crate::ide::offset_to_position(&result.code, event);
        let prefix_position = crate::ide::offset_to_position(&result.code, event - "update:".len());
        let generic = super::super::mapping::source_range_to_virtual_positions(
            &result,
            &source,
            &declaration,
        );
        assert_eq!(
            generic.contains(&prefix_position),
            name.len() < "update:".len(),
            "short names round-trip through the synthesized prefix: {name}"
        );
        assert!(!direct_model_position(
            &result,
            &source,
            &declaration,
            prefix_position.0,
            prefix_position.1,
        ));
        let request_uri = "file:///workspace/Child.vue.ts";
        let mut document = CanonicalVirtualDocument {
            source_uri: uri.clone(),
            request_uri: request_uri.into(),
            virtual_result: result,
            dependencies: Vec::new(),
            materialized_sources: Vec::new(),
            session_project_roots: Vec::new(),
            source_catalogs: Vec::new(),
        };
        let actual = model_linked_semantic_positions(&ctx, &document, request_uri, &range);
        let mut coordinates: Vec<_> = actual
            .iter()
            .map(|position| {
                assert_eq!(position.request_uri.as_str(), request_uri);
                (position.line, position.character)
            })
            .collect();
        let mut expected = vec![expected_prop, expected_event];
        expected.sort_unstable();
        coordinates.sort_unstable();
        assert_eq!(
            coordinates, expected,
            "only the two native model identities"
        );

        // An exact coordinate round-trip through a broad setup statement is
        // insufficient: the original defineModel argument is a script string,
        // not a third native prop/event declaration.
        let script_start = source.find('\n').unwrap() + 1;
        let script_end = source.find("</script>").unwrap();
        let script = source.get(script_start..script_end).unwrap();
        document.virtual_result.code.push('\n');
        let generated = document.virtual_result.code.len();
        document.virtual_result.code.push_str(script);
        let pre = document
            .virtual_result
            .import_source_map
            .get_original_offset(generated as u32) as usize;
        let broad = vize_canon::virtual_ts::VizeMapping::new(
            pre..pre + script.len(),
            script_start..script_end,
        );
        document.virtual_result.source_mappings.push(broad.clone());
        let script_position = crate::ide::offset_to_position(
            &document.virtual_result.code,
            generated + declaration.start - script_start,
        );
        let generic = super::super::mapping::source_range_to_virtual_positions(
            &document.virtual_result,
            &source,
            &declaration,
        );
        assert!(
            generic.contains(&script_position),
            "the broad row round-trips"
        );
        assert!(!direct_model_position(
            &document.virtual_result,
            &source,
            &declaration,
            script_position.0,
            script_position.1,
        ));
        let mut filtered: Vec<_> =
            model_linked_semantic_positions(&ctx, &document, request_uri, &range)
                .into_iter()
                .map(|position| (position.line, position.character))
                .collect();
        filtered.sort_unstable();
        assert_eq!(filtered, expected, "broad script identity must be refused");
        assert!(
            model_linked_semantic_positions(
                &ctx,
                &document,
                "file:///workspace/Other.vue.ts",
                &range
            )
            .is_empty()
        );
        let mut unowned = document;
        unowned.virtual_result.source_mappings = vec![broad];
        assert!(model_linked_semantic_positions(&ctx, &unowned, request_uri, &range).is_empty());
        unowned.virtual_result.source_mappings.clear();
        assert!(model_linked_semantic_positions(&ctx, &unowned, request_uri, &range).is_empty());
    }
}
