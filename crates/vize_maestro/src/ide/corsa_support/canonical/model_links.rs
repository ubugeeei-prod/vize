//! Follow native identities owned by one retained authored model declaration.

use std::ops::Range;

use vize_croquis::{Drawer, DrawerOptions};

use super::{CanonicalSemanticPosition, CanonicalVirtualDocument};
use crate::ide::IdeContext;
use crate::ide::diagnostics::VirtualTsResult;

pub(crate) fn model_linked_semantic_positions(
    ctx: &IdeContext<'_>,
    document: &CanonicalVirtualDocument,
    uri: &str,
    range: &vize_canon::LspRange,
) -> Vec<CanonicalSemanticPosition> {
    let Some((request_uri, source_uri, source, result)) = virtual_source(ctx, document, uri) else {
        return Vec::new();
    };
    let Some(authored) =
        super::mapping::map_virtual_result_lsp_range_to_source(source, result, range)
    else {
        return Vec::new();
    };
    let Some(start) =
        crate::ide::position_to_offset(source, authored.start.line, authored.start.character)
    else {
        return Vec::new();
    };
    let Some(end) =
        crate::ide::position_to_offset(source, authored.end.line, authored.end.character)
    else {
        return Vec::new();
    };
    let authored = start..end;
    // Ordinary references keep their existing scan-only fast path. A retained
    // model event row must own this native location before consulting the
    // existing authored macro declaration scanner.
    if !has_model_event(result, &authored) {
        return Vec::new();
    }
    let Some(declaration) = model_declaration_ranges(source, source_uri.path())
        .into_iter()
        .find(|declaration| owns_declaration(source, declaration, &authored))
    else {
        return Vec::new();
    };
    super::mapping::source_range_to_virtual_positions(result, source, &declaration)
        .into_iter()
        .filter(|&(line, character)| {
            direct_model_position(result, source, &declaration, line, character)
        })
        .map(|(line, character)| CanonicalSemanticPosition {
            request_uri: request_uri.clone(),
            line,
            character,
        })
        .collect()
}

fn direct_model_position(
    result: &VirtualTsResult,
    source: &str,
    declaration: &Range<usize>,
    line: u32,
    character: u32,
) -> bool {
    let Some(post) = crate::ide::position_to_offset(&result.code, line, character) else {
        return false;
    };
    let pre = result.import_source_map.get_original_offset(post as u32) as usize;
    // A broad setup mapping can round-trip the defineModel string argument,
    // but that coordinate is not the projected prop or update identity. The
    // synthesized update prefix likewise has no authored name counterpart.
    result.source_mappings.iter().any(|mapping| {
        mapping.sub_spans.is_empty()
            && mapping.gen_range.len() == mapping.src_range.len()
            && owns_declaration(source, declaration, &mapping.src_range)
            && mapping.gen_range.contains(&pre)
    })
}

fn owns_declaration(source: &str, declaration: &Range<usize>, authored: &Range<usize>) -> bool {
    if declaration == authored {
        return true;
    }
    let Some(start) = declaration.start.checked_sub(1) else {
        return false;
    };
    let Some(end) = declaration.end.checked_add(1) else {
        return false;
    };
    let outer = start..end;
    outer == *authored
        && source.get(outer).is_some_and(
            |text| matches!(text.as_bytes(), [open @ (b'\'' | b'"'), .., close] if open == close),
        )
}

fn has_model_event(result: &VirtualTsResult, authored: &Range<usize>) -> bool {
    result.source_mappings.iter().any(|mapping| {
        let owns_range = mapping.src_range == *authored
            || (mapping.src_range.start.checked_add(1) == Some(authored.start)
                && mapping.src_range.end.checked_sub(1) == Some(authored.end))
            || (mapping.src_range.start.checked_sub(1) == Some(authored.start)
                && mapping.src_range.end.checked_add(1) == Some(authored.end));
        if !owns_range {
            return false;
        }
        let generated = result
            .import_source_map
            .get_virtual_offset(mapping.gen_range.start as u32) as usize;
        result
            .code
            .get(..generated)
            .is_some_and(|prefix| prefix.ends_with("  /* __vize_model_event */ "))
    })
}

fn virtual_source<'a>(
    ctx: &'a IdeContext<'_>,
    document: &'a CanonicalVirtualDocument,
    uri: &str,
) -> Option<(
    &'a vize_l0::String,
    &'a tower_lsp::lsp_types::Url,
    &'a str,
    &'a VirtualTsResult,
)> {
    if uri == document.request_uri {
        return Some((
            &document.request_uri,
            ctx.uri,
            &ctx.content,
            &document.virtual_result,
        ));
    }
    document
        .dependencies
        .iter()
        .find(|dependency| uri == dependency.request_uri)
        .map(|dependency| {
            (
                &dependency.request_uri,
                &dependency.source_uri,
                dependency.source.as_str(),
                &dependency.virtual_result,
            )
        })
        .or_else(|| {
            document
                .materialized_sources
                .iter()
                .find(|source| {
                    uri == source.request_uri
                        && source.mapping_kind
                            == vize_canon::CorsaMaterializedMappingKind::Generated
                })
                .map(|source| {
                    (
                        &source.request_uri,
                        &source.source_uri,
                        source.source.as_str(),
                        &source.virtual_result,
                    )
                })
        })
}

pub(crate) fn model_declaration_ranges(source: &str, filename: &str) -> Vec<Range<usize>> {
    let Some(descriptor) = vize_incremental::parse_descriptor(filename, source) else {
        return Vec::new();
    };
    let Some(script) = descriptor.script_setup.as_ref() else {
        return Vec::new();
    };
    let mut drawer = Drawer::with_options(DrawerOptions {
        analyze_script: true,
        ..Default::default()
    });
    drawer.analyze_script_setup(&script.content);
    let croquis = drawer.finish();
    croquis
        .macros
        .models()
        .iter()
        .filter_map(|model| {
            let (start, end) = croquis.macros.model_declaration(model.name.as_str())?;
            let mut range = script.loc.start + start as usize..script.loc.start + end as usize;
            let authored = source.get(range.clone())?;
            let unquoted = authored
                .strip_prefix(['\'', '"'])
                .and_then(|name| name.strip_suffix(['\'', '"']))
                .unwrap_or(authored);
            (unquoted == model.name).then(|| {
                if unquoted.len() != authored.len() {
                    range.start += 1;
                    range.end -= 1;
                }
                range
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
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
            let result =
                DiagnosticService::generate_virtual_ts(&uri, &source, false, false).unwrap();
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
            let prefix_position =
                crate::ide::offset_to_position(&result.code, event - "update:".len());
            let generic = super::super::mapping::source_range_to_virtual_positions(
                &result,
                &source,
                &declaration,
            );
            // Short names round-trip through the synthesized `update:` prefix.
            let mut expected_generic = vec![(103, 3), (129, 44)];
            if name.len() < "update:".len() {
                expected_generic.push(prefix_position);
            }
            expected_generic.push((169, 35));
            assert_eq!(generic, expected_generic, "{name}");
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
            // The broad row round-trips after the native rows.
            let mut expected_generic = vec![(101, 3), (127, 44)];
            if name.len() < "update:".len() {
                expected_generic.push((167, 28));
            }
            expected_generic.extend([(167, 35), script_position]);
            assert_eq!(generic, expected_generic, "{name}");
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
            assert!(
                model_linked_semantic_positions(&ctx, &unowned, request_uri, &range).is_empty()
            );
            unowned.virtual_result.source_mappings.clear();
            assert!(
                model_linked_semantic_positions(&ctx, &unowned, request_uri, &range).is_empty()
            );
        }
    }
}
