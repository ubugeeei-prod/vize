//! Ask native references on the value identity of a reactive shorthand binding.

use oxc_ast::{
    AstKind,
    ast::{BindingPattern, Expression},
};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

use super::CanonicalVirtualDocument;
use crate::ide::IdeContext;

/// TypeScript references at a shorthand binding include its source property
/// group, although rename at that cursor selects the local variable. A real
/// resolved value reference avoids mixing the two identities before querying.
pub(crate) fn local_binding_reference_position(
    ctx: &IdeContext<'_>,
    document: &CanonicalVirtualDocument,
    request_uri: &str,
    position: (u32, u32),
) -> Option<(u32, u32)> {
    if !is_reactive_binding_cursor(ctx) {
        return Some(position);
    }
    let (_, result) = super::semantic_links::virtual_result(document, request_uri)?;
    let code = &result.code;
    let offset = crate::ide::position_to_offset(code, position.0, position.1)?;
    let allocator = oxc_allocator::Allocator::default();
    let mut parsed = Parser::new(&allocator, code, SourceType::ts()).parse();
    if !parsed.diagnostics.is_empty() {
        parsed = Parser::new(&allocator, code, SourceType::tsx()).parse();
    }
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program)
        .semantic;
    let binding = semantic.nodes().iter().find_map(|node| match node.kind() {
        AstKind::BindingIdentifier(binding)
            if (binding.span.start as usize..binding.span.end as usize).contains(&offset) =>
        {
            binding.symbol_id.get()
        }
        _ => None,
    })?;
    let reference = semantic
        .scoping()
        .get_resolved_references(binding)
        .filter_map(
            |reference| match semantic.nodes().get_node(reference.node_id()).kind() {
                AstKind::IdentifierReference(identifier) => Some(identifier.span.start as usize),
                _ => None,
            },
        )
        .min()?;
    Some(crate::ide::offset_to_position(code, reference))
}

fn is_reactive_binding_cursor(ctx: &IdeContext<'_>) -> bool {
    let Some(descriptor) = ctx.descriptor() else {
        return false;
    };
    let Some(script) = descriptor.script_setup.as_ref() else {
        return false;
    };
    let Some(offset) = ctx.offset.checked_sub(script.loc.start) else {
        return false;
    };
    if offset >= script.content.len() {
        return false;
    }
    let allocator = oxc_allocator::Allocator::default();
    let source_type = match script.lang.as_deref() {
        Some("tsx") => SourceType::tsx(),
        Some("jsx") => SourceType::jsx(),
        _ => SourceType::ts(),
    };
    let parsed = Parser::new(&allocator, &script.content, source_type).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return false;
    }
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program)
        .semantic;
    semantic.nodes().iter().any(|node| {
        let AstKind::BindingIdentifier(binding) = node.kind() else {
            return false;
        };
        if !(binding.span.start as usize..binding.span.end as usize).contains(&offset) {
            return false;
        }
        let mut id = node.id();
        loop {
            let parent = semantic.nodes().parent_id(id);
            if parent == id {
                return false;
            }
            id = parent;
            if let AstKind::VariableDeclarator(declarator) = semantic.nodes().get_node(id).kind() {
                return matches!(&declarator.id, BindingPattern::ObjectPattern(_))
                    && declarator.init.as_ref().is_some_and(is_define_props);
            }
        }
    })
}

fn is_define_props(expression: &Expression<'_>) -> bool {
    let Expression::CallExpression(call) = expression.get_inner_expression() else {
        return false;
    };
    let Expression::Identifier(callee) = call.callee.get_inner_expression() else {
        return false;
    };
    callee.name == "defineProps"
        || (callee.name == "withDefaults"
            && call
                .arguments
                .first()
                .and_then(|argument| argument.as_expression())
                .is_some_and(is_define_props))
}

#[cfg(test)]
mod tests {
    use super::local_binding_reference_position;
    use crate::{
        ide::{DiagnosticService, IdeContext, corsa_support},
        server::ServerState,
    };
    use tower_lsp::lsp_types::Url;

    #[test]
    fn native_reference_query_uses_a_resolved_value_for_reactive_shorthand_only() {
        for (source, needle, rerouted) in [
            (
                "<script setup lang=\"ts\">const { label } = defineProps<{ label: string }>();</script><template>{{ label }}</template>",
                "label }",
                true,
            ),
            (
                "<script setup lang=\"tsx\">const view = <span />; const { label = 'Name' } = defineProps<{ label: string }>();</script><template>{{ label }}</template>",
                "label =",
                true,
            ),
            (
                "<script setup lang=\"ts\">const value = { label: 'Name' }; const { label } = value;</script><template>{{ label }}</template>",
                "label }",
                false,
            ),
        ] {
            let state = ServerState::new();
            let uri = Url::parse("file:///workspace/Child.vue").expect("URI");
            let offset = source.find(needle).expect("binding marker");
            let ctx = IdeContext::testing(&state, &uri, offset, source.to_owned());
            let document = corsa_support::CanonicalVirtualDocument {
                source_uri: uri.clone(),
                request_uri: "file:///workspace/Child.vue.ts".into(),
                virtual_result: DiagnosticService::generate_virtual_ts(
                    ctx.uri, source, false, false,
                )
                .expect("virtual document"),
                dependencies: Vec::new(),
                materialized_sources: Vec::new(),
                session_project_roots: Vec::new(),
                source_catalogs: Vec::new(),
            };
            let original = corsa_support::canonical_source_offset_to_position(&document, offset)
                .expect("native declaration position");
            let selected =
                local_binding_reference_position(&ctx, &document, &document.request_uri, original)
                    .expect("native value position");
            assert_eq!(selected != original, rerouted, "{source}");
            let generated = crate::ide::position_to_offset(
                &document.virtual_result.code,
                selected.0,
                selected.1,
            )
            .expect("value offset");
            assert_eq!(
                document.virtual_result.code.get(generated..generated + 5),
                Some("label")
            );
        }
    }
}
