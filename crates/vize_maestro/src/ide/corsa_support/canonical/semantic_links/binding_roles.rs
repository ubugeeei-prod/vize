//! Separate authored setup binding identities from public component keys.

use std::ops::Range;

use oxc_ast::ast::BindingIdentifier;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
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
            .or_insert_with(|| {
                let phase = vize_canon::corsa_bridge::EditorPreparationPhase::navigation(
                    "setup_binding_ranges",
                    1,
                );
                let ranges = setup_binding_ranges(ctx, uri, source);
                phase.finish();
                ranges
            })
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
                .map(|source| std::sync::Arc::new(source.to_owned()))
                .or_else(|| ctx.state.documents.text_snapshot(&definition.uri))
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
    let mut bindings = BindingRanges {
        offset: script.loc.start,
        ranges: Vec::new(),
    };
    bindings.visit_program(&parsed.program);
    bindings.ranges
}

/// Binding roles are syntax. Preserve the full AST visitation order without
/// constructing symbol tables, references, scopes or semantic node storage.
struct BindingRanges {
    offset: usize,
    ranges: Vec<Range<usize>>,
}

impl<'a> Visit<'a> for BindingRanges {
    fn visit_binding_identifier(&mut self, binding: &BindingIdentifier<'a>) {
        self.ranges.push(
            self.offset + binding.span.start as usize..self.offset + binding.span.end as usize,
        );
    }
}

#[cfg(test)]
mod oracle_tests;

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
        let label = source.find("label =").expect("default binding");
        let active = source.find("active }").expect("aliased binding");
        assert_eq!(ranges, vec![label..label + 5, active..active + 6]);
    }
}
