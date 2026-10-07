//! Separate authored setup binding identities from public component keys.

use std::ops::Range;

use oxc_ast::AstKind;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use tower_lsp::lsp_types::{Location, Url};
use vize_l0::FxHashMap;

use super::CanonicalVirtualDocument;
use crate::ide::IdeContext;

pub(super) fn is_local_query(
    ctx: &IdeContext<'_>,
    document: &CanonicalVirtualDocument,
    definitions: &[Location],
) -> bool {
    // An argument's explicit producer endpoint always selects its public key.
    if super::super::component_attribute_position(ctx, document).is_some() {
        return false;
    }
    let mut cache = FxHashMap::<Url, Vec<Range<usize>>>::default();
    let mut contains = |uri: &Url, source: &str, offset: usize| {
        cache
            .entry(uri.clone())
            .or_insert_with(|| setup_binding_ranges(ctx, uri, source))
            .iter()
            .any(|range| range.contains(&offset))
    };
    if contains(ctx.uri, &ctx.content, ctx.offset) {
        return true;
    }
    definitions.iter().any(|definition| {
        let source = if definition.uri == *ctx.uri {
            Some(ctx.content.clone())
        } else {
            document
                .authored_source(&definition.uri)
                .map(str::to_owned)
                .or_else(|| ctx.state.documents.text(&definition.uri))
        };
        source.is_some_and(|source| {
            crate::ide::position_to_offset(
                &source,
                definition.range.start.line,
                definition.range.start.character,
            )
            .is_some_and(|offset| contains(&definition.uri, &source, offset))
        })
    })
}

fn setup_binding_ranges(ctx: &IdeContext<'_>, uri: &Url, source: &str) -> Vec<Range<usize>> {
    if !uri.path().ends_with(".vue") {
        return Vec::new();
    }
    let local = IdeContext::for_unopened(ctx.state, uri, 0, source.to_owned());
    let Some(descriptor) = local.descriptor() else {
        return Vec::new();
    };
    let Some(script) = descriptor.script_setup.as_ref() else {
        return Vec::new();
    };
    let allocator = oxc_allocator::Allocator::default();
    let source_type = match script.lang.as_deref() {
        Some("tsx") => SourceType::tsx(),
        Some("jsx") => SourceType::jsx(),
        _ => SourceType::ts(),
    };
    let parsed = Parser::new(&allocator, &script.content, source_type).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return Vec::new();
    }
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program)
        .semantic;
    semantic
        .nodes()
        .iter()
        .filter_map(|node| match node.kind() {
            AstKind::BindingIdentifier(binding) => Some(
                script.loc.start + binding.span.start as usize
                    ..script.loc.start + binding.span.end as usize,
            ),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::setup_binding_ranges;
    use crate::{ide::IdeContext, server::ServerState};
    use tower_lsp::lsp_types::Url;

    #[test]
    fn reactive_destructure_bindings_and_type_property_keys_have_distinct_ranges() {
        let source = "<script setup lang=\"ts\">const { label = 'Name', checked: active } = defineProps<{ label: string; checked: boolean }>();</script>";
        let state = ServerState::new();
        let uri = Url::parse("file:///workspace/Child.vue").expect("URI");
        let ctx = IdeContext::testing(&state, &uri, 0, source.to_owned());
        let ranges = setup_binding_ranges(&ctx, &uri, source);
        for (needle, expected) in [
            ("label =", true),
            ("active }", true),
            ("checked:", false),
            ("label: string", false),
        ] {
            let offset = source.find(needle).expect("role marker");
            assert_eq!(
                ranges.iter().any(|range| range.contains(&offset)),
                expected,
                "{needle}"
            );
        }
    }
}
