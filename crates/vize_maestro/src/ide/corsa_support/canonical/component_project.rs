//! On-demand configured-project coverage for public component member navigation.

use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use vize_canon::CorsaBridge;
use vize_croquis::{Drawer, DrawerOptions};
use vize_relief::BindingType;

use super::semantic_links::{
    CanonicalSemanticPosition, ComponentPropNavigationMatches, materialized_semantic_positions,
};
use super::{CanonicalProjectOpenError, CanonicalVirtualDocument};
use crate::ide::IdeContext;

mod definition_positions;

/// Ordinary local symbols keep the existing open-importer surface. Public
/// component props and events also need unopened consumers, independent of `crossFile`.
/// This route is called only by references and rename, never by document edits,
/// diagnostics, hover, completion or prepareRename. The existing TypeScript
/// definition checks still decide which same-spelling public endpoints match.
pub(crate) async fn open_canonical_virtual_navigation_project_document_strict(
    ctx: &IdeContext<'_>,
    bridge: &CorsaBridge,
) -> Result<Option<CanonicalVirtualDocument>, CanonicalProjectOpenError> {
    if may_query_component_prop(ctx) {
        super::project::open_canonical_virtual_workspace_document_strict(ctx, bridge).await
    } else {
        super::project::open_canonical_virtual_project_document_strict(ctx, bridge).await
    }
}

fn may_query_component_prop(ctx: &IdeContext<'_>) -> bool {
    if let Some((_, component)) =
        crate::ide::definition::helpers::get_attribute_and_component_at_offset(ctx)
    {
        return crate::ide::component_name_candidates(&component)
            .iter()
            .any(|name| crate::ide::definition::helpers::find_import_path(ctx, name).is_some());
    }
    let Some(name) = crate::ide::script_identifier::at_offset(&ctx.content, ctx.offset) else {
        return false;
    };
    let Some(descriptor) = ctx.descriptor() else {
        return false;
    };
    let Some(script) = descriptor.script_setup.as_ref() else {
        return false;
    };
    let allocator = oxc_allocator::Allocator::default();
    let source_type = SourceType::ts().with_jsx(matches!(
        script.lang.as_deref().map(str::trim),
        Some("tsx" | "jsx")
    ));
    let parsed = vize_croquis::script_parser::parse_program_for_analysis(
        &allocator,
        &script.content,
        source_type,
    );
    if parsed.panicked {
        return false;
    }
    let mut analyzer = Drawer::with_options(DrawerOptions::full());
    analyzer.analyze_script_setup_program(
        &parsed.program,
        &script.content,
        script.attrs.get("generic").map(|generic| generic.as_ref()),
    );
    let croquis = analyzer.finish();
    let event_candidate = croquis
        .macros
        .emits()
        .iter()
        .any(|event| event.name.as_str() == name);
    if event_candidate && parsed.diagnostics.is_empty() {
        let built = SemanticBuilder::new().build(&parsed.program);
        if built.diagnostics.is_empty()
            && vize_canon::virtual_ts::owned_emit_navigation_source_ranges(
                &parsed.program,
                built.semantic.scoping(),
                &script.content,
                &croquis,
            )
            .iter()
            .any(|range| range.contains(&ctx.offset.saturating_sub(script.loc.start)))
        {
            return true;
        }
    }
    if ctx.block_type == Some(crate::virtual_code::BlockType::ScriptSetup)
        && croquis
            .scopes
            .bindings_visible_at(ctx.offset.saturating_sub(script.loc.start) as u32)
            .iter()
            .any(|(binding_name, binding, _)| {
                *binding_name == name && binding.binding_type != BindingType::Props
            })
    {
        return false;
    }
    croquis.get_props().any(|(prop, _)| prop == name) || event_candidate
}

impl ComponentPropNavigationMatches {
    /// Attribute queries resolve the public prop declaration, while the local
    /// template binding can have a separate generated TypeScript identity.
    pub(crate) fn authored_definition_positions(
        &self,
        ctx: &crate::ide::IdeContext<'_>,
        document: &CanonicalVirtualDocument,
        include_property_projections: bool,
    ) -> Vec<CanonicalSemanticPosition> {
        let mut positions = Vec::new();
        for definition in &self.authored_definitions {
            let source = if definition.uri == *ctx.uri {
                Some(ctx.content.clone())
            } else {
                document
                    .authored_source(&definition.uri)
                    .map(str::to_owned)
                    .or_else(|| ctx.state.documents.text(&definition.uri))
                    .or_else(|| {
                        definition
                            .uri
                            .to_file_path()
                            .ok()
                            .and_then(|path| std::fs::read_to_string(path).ok())
                    })
            };
            let Some(offset) = source.as_deref().and_then(|source| {
                crate::ide::position_to_offset(
                    source,
                    definition.range.start.line,
                    definition.range.start.character,
                )
            }) else {
                continue;
            };
            positions.extend(materialized_semantic_positions(
                document,
                &definition.uri,
                offset,
            ));
            if include_property_projections && let Some(source) = source.as_deref() {
                positions.extend(definition_positions::positions(
                    document, definition, source,
                ));
            }
        }
        positions
    }
}

#[cfg(test)]
mod tests {
    use super::may_query_component_prop;
    use crate::{ide::IdeContext, server::ServerState};
    use tower_lsp::lsp_types::Url;

    #[test]
    fn prop_queries_expand_but_local_bindings_and_parameter_shadows_do_not() {
        let source = "<script setup lang=\"ts\">\ndefineProps<{ text?: string }>();\nimport Child from './Child.vue';\nconst local = 1;\nfunction inner(text: string) { return text; }\n</script>\n<template>{{ text }} {{ local }} <Child text=\"Save\" /><div text=\"Native\" /></template>";
        let state = ServerState::new();
        let uri = Url::parse("file:///workspace/Child.vue").unwrap();
        state
            .documents
            .open(uri.clone(), source.to_owned(), 1, "vue".to_owned());
        for (needle, expected) in [
            ("text?:", true),
            ("text }}", true),
            ("text: string", false),
            ("text;", false),
            ("local =", false),
            ("local }}", false),
            ("text=\"Save\"", true),
            ("text=\"Native\"", false),
        ] {
            let ctx = IdeContext::new(&state, &uri, source.find(needle).unwrap()).unwrap();
            assert_eq!(may_query_component_prop(&ctx), expected, "{needle}");
        }
    }

    #[test]
    fn declared_event_keys_and_literal_calls_expand_the_configured_project() {
        let source = "<script setup lang=\"ts\">\nconst emit = defineEmits<{ change: [value: boolean] }>();\nfunction flip() { emit(\"change\", true); }\n</script>";
        let state = ServerState::new();
        let uri = Url::parse("file:///workspace/Toggle.vue").unwrap();
        state
            .documents
            .open(uri.clone(), source.to_owned(), 1, "vue".to_owned());
        for (needle, expected) in [("change:", true), ("change\"", true), ("flip()", false)] {
            let ctx = IdeContext::new(&state, &uri, source.find(needle).unwrap()).unwrap();
            assert_eq!(may_query_component_prop(&ctx), expected, "{needle}");
        }
    }

    #[test]
    fn public_event_roles_expand_without_admitting_same_name_value_roles() {
        let source = "<script setup lang=\"ts\">\nconst emit = defineEmits<{ change: [value: boolean] }>();\nconst change = true;\nfunction flip() { emit(\"change\", change); }\nfunction shadow(emit: (name: string, value: boolean) => void) { emit(\"change\", change); }\n</script>";
        let state = ServerState::new();
        let uri = Url::parse("file:///workspace/Toggle.vue").unwrap();
        state
            .documents
            .open(uri.clone(), source.to_owned(), 1, "vue".to_owned());
        let actual = [
            "change:",
            "change =",
            "change\", change",
            "change);",
            "change\", change); }\n</script>",
        ]
        .map(|needle| {
            let ctx = IdeContext::new(&state, &uri, source.find(needle).unwrap()).unwrap();
            may_query_component_prop(&ctx)
        });
        assert_eq!(actual, [true, false, true, false, false]);
    }
}
