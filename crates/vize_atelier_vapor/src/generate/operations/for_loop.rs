use crate::ir::ForIRNode;
use vize_atelier_core::codegen::document::EmitDocument;
use vize_carton::{FxHashMap, String, ToCompactString, cstr};

use super::{
    super::{context::GenerateContext, destructure::needs_parsed_reads, generate_block},
    block_shape::{fragment_rows, single_node_rows},
    insertion::emit_insertion_state,
};

/// Generate For
pub(super) fn generate_for(
    ctx: &mut GenerateContext,
    for_node: &ForIRNode<'_>,
    element_template_map: &FxHashMap<usize, usize>,
) {
    if for_node.match_scope {
        super::match_scope::generate_match_scope(ctx, for_node, element_template_map);
        return;
    }
    ctx.use_helper("createFor");
    emit_insertion_state(ctx, for_node.parent, for_node.anchor);

    let depth = ctx.for_scopes.len();
    let mut source = EmitDocument::plain("(");
    if for_node.source.is_static {
        source.push_str(for_node.source.content);
    } else {
        source.push_spanned(&ctx.spanned_expression_node(&for_node.source));
    }
    source.push_str(")");

    let key_alias = for_node.key.as_ref().map(|k| k.content);
    let index_alias = for_node.index.as_ref().map(|i| i.content);

    // Build parameter list using _for_item0, _for_key0 naming
    let for_item_var = cstr!("_for_item{}", depth);
    let for_key_var = cstr!("_for_key{}", depth);
    let for_index_var = cstr!("_for_index{}", depth);

    let mut params = for_item_var.clone();
    if key_alias.is_some() || index_alias.is_some() {
        params.push_str(", ");
        params.push_str(&for_key_var);
    }
    if index_alias.is_some() {
        params.push_str(", ");
        params.push_str(&for_index_var);
    }

    // Push for scope before generating body
    ctx.push_for_scope_retained(
        for_node.value.as_deref(),
        for_node.key.as_deref(),
        for_node.index.as_deref(),
    );

    let slot_root = ctx.transition_slot
        && for_node.parent.is_none()
        && for_node.key_prop.is_some()
        && matches!(for_node.render.returns.as_slice(), [id] if element_template_map.contains_key(id));
    let previous_transition = ctx.transition_slot;
    ctx.transition_slot = false;
    let was_fragment = ctx.is_fragment;
    ctx.is_fragment = true;

    // `_createFor(` opens at the authored element carrying `v-for`.
    let for_id_str = for_node.id.to_compact_string();
    let mut head = EmitDocument::plain(&["const n", &for_id_str, " = "].concat());
    head.push_spanned(&ctx.spanned_at("_createFor(", ctx.unit_of(&for_node.source)));
    head.push_str("() => ");
    head.push_spanned(&source);
    head.push_str(&[", (", &params, ") => {"].concat());
    ctx.push_line_spanned(&head);
    ctx.indent();
    ctx.push_component_scope();
    // A `v-if` that is a loop row renders its branches as fragments.
    let row_ifs = for_node
        .render
        .operation
        .iter()
        .filter_map(|operation| match operation {
            crate::ir::OperationNode::If(node) if for_node.render.returns.contains(&node.id) => {
                Some(node.id)
            }
            _ => None,
        })
        .collect::<std::vec::Vec<_>>();
    let outer_row_ifs = core::mem::replace(&mut ctx.loop_row_ifs, row_ifs);
    generate_block(ctx, &for_node.render, element_template_map);
    ctx.loop_row_ifs = outer_row_ifs;
    ctx.pop_component_scope();
    ctx.deindent();

    // Generate key function if key_prop is provided
    let key_func = generate_for_key_function(ctx, for_node);

    // Check if this is a range-based for (source is a number literal)
    let is_range = for_node.source.content.parse::<f64>().is_ok();

    // Rows that are one component each unmount one by one (upstream's
    // `IS_COMPONENT`: a component carrying `v-for`, or a `<template>` holding
    // only one).
    let component = for_node.component || component_rows(&for_node.render);
    // The runtime's `createFor` flags, as Vue's compiler encodes them.
    let table = [
        (for_node.only_child, 1, "FAST_REMOVE"),
        (component, 2, "IS_COMPONENT"),
        (fragment_rows(&for_node.render), 16, "IS_FRAGMENT"),
        (
            slot_root || !component && single_node_rows(&for_node.render, element_template_map),
            8,
            "IS_SINGLE_NODE",
        ),
        (is_range, 4, "ONCE"),
        (slot_root, 32, "SLOT_ROOT"),
        (
            for_node.wrapped_rows && !previous_transition,
            64,
            "WRAPPED_ROWS",
        ),
    ];
    let flags = table
        .iter()
        .filter(|(set, _, _)| *set)
        .fold(0u32, |flags, (_, bit, _)| flags | bit);
    ctx.push_indent();
    ctx.push("}");
    if let Some(key_fn) = &key_func {
        ctx.push(", ");
        ctx.push(key_fn);
    }
    if flags != 0 {
        use std::fmt::Write as _;
        if key_func.is_none() {
            ctx.push(", undefined");
        }
        let _ = write!(ctx, ", {flags} /* ");
        for (index, (_, _, name)) in table.iter().filter(|(set, _, _)| *set).enumerate() {
            if index > 0 {
                ctx.push(", ");
            }
            ctx.push(name);
        }
        ctx.push(" */");
    }
    ctx.push(")\n");

    ctx.transition_slot = previous_transition;
    ctx.is_fragment = was_fragment;
    ctx.for_scopes.pop();
}

/// Whether every row is a single component, or a single component under
/// a lone `v-if` (upstream reads the row element's tag type either way).
fn component_rows(render: &crate::ir::BlockIRNode<'_>) -> bool {
    use crate::ir::OperationNode;
    let [id] = render.returns.as_slice() else {
        return false;
    };
    render.operation.iter().any(|operation| match operation {
        OperationNode::CreateComponent(component) => component.id == *id,
        OperationNode::If(node) => {
            node.id == *id && node.negative.is_none() && component_rows(&node.positive)
        }
        _ => false,
    })
}

/// Generate key function for v-for
fn generate_for_key_function(
    ctx: &mut GenerateContext<'_>,
    for_node: &ForIRNode<'_>,
) -> Option<String> {
    if let Some(ref key_prop) = for_node.key_prop {
        let key_expr = resolve_key_expression(ctx, key_prop);
        // Build params: (value_alias) or (value_alias, key_alias)
        let value_name = match for_node.value.as_ref().map(|v| v.content) {
            Some(pattern)
                if pattern.trim_start().starts_with(['{', '[']) && needs_parsed_reads(pattern) =>
            {
                for_node
                    .value
                    .as_ref()
                    .and_then(|node| ctx.resolve_retained_pattern_parameter(node))
                    .unwrap_or_else(|| ctx.resolve_pattern_parameter(pattern))
            }
            Some(alias) => alias.to_compact_string(),
            None => "_item".to_compact_string(),
        };
        let key_name = for_node
            .key
            .as_ref()
            .map(|k| vize_atelier_core::steps::untyped_for_alias(k.content));
        let index_name = for_node
            .index
            .as_ref()
            .map(|i| vize_atelier_core::steps::untyped_for_alias(i.content));

        let mut params = value_name;
        let untyped_len = vize_atelier_core::steps::untyped_for_alias(&params).len();
        params.truncate(untyped_len);
        if key_name.is_some() || index_name.is_some() {
            params.push_str(", ");
            params.push_str(key_name.unwrap_or("_key"));
        }
        if let Some(index) = index_name {
            params.push_str(", ");
            params.push_str(index);
        }

        Some(cstr!("({params}) => ({key_expr})"))
    } else {
        None
    }
}

fn resolve_key_expression(
    ctx: &mut GenerateContext<'_>,
    key_prop: &vize_atelier_core::SimpleExpressionNode<'_>,
) -> String {
    let Some(scope) = ctx.for_scopes.last_mut() else {
        return ctx.resolve_expression_node(key_prop);
    };
    let previous = core::mem::replace(&mut scope.callback_locals, true);
    // Resolve the original key AST in the callback's own namespace. Generated
    // reactive paths are never substituted back through authored string bytes.
    let resolved = ctx.resolve_expression_node(key_prop);
    // The immutable resolver cannot change the scope stack, including on its
    // literal, simple-reference and retained/fallback expression fast paths.
    if let Some(scope) = ctx.for_scopes.last_mut() {
        scope.callback_locals = previous;
    }
    resolved
}

#[cfg(test)]
mod typed_alias_tests;

#[cfg(test)]
mod tests {
    struct KeyFunctions<'code> {
        code: &'code str,
        functions: vize_carton::SmallVec<[&'code str; 2]>,
    }

    impl<'ast> oxc_ast_visit::Visit<'ast> for KeyFunctions<'_> {
        fn visit_call_expression(&mut self, call: &oxc_ast::ast::CallExpression<'ast>) {
            if matches!(&call.callee, oxc_ast::ast::Expression::Identifier(name) if name.name == "_createFor")
            {
                let Some(oxc_ast::ast::Argument::ArrowFunctionExpression(key)) =
                    call.arguments.get(2)
                else {
                    panic!("keyed fixture must emit a key callback: {}", self.code);
                };
                self.functions.push(
                    self.code
                        .get(key.span.start as usize..key.span.end as usize)
                        .expect("whole key span"),
                );
            }
            oxc_ast_visit::walk::walk_call_expression(self, call);
        }
    }

    #[test]
    fn key_callback_ast_preserves_authored_literals_and_binding_ownership() {
        let cases: [(&str, &[&str]); 12] = [
            (
                r#"<p v-for="\u005frow of items" :key="capture('_for_item0.value')">ok</p>"#,
                &[r#"(\u005frow) => (_ctx.capture('_for_item0.value'))"#],
            ),
            (
                r#"<p v-for="\u005frow of items" :key="capture(`_for_item0.value`)">ok</p>"#,
                &[r#"(\u005frow) => (_ctx.capture(`_for_item0.value`))"#],
            ),
            (
                r#"<p v-for="\u005frow of items" :key="capture(/_for_item0.value/.source)">ok</p>"#,
                &[r#"(\u005frow) => (_ctx.capture(/_for_item0.value/.source))"#],
            ),
            (
                r#"<p v-for="\u005frow of items" :key="capture(/* _for_item0.value */ '_for_item0.value')">ok</p>"#,
                &[r#"(\u005frow) => (_ctx.capture(/* _for_item0.value */ '_for_item0.value'))"#],
            ),
            (
                r#"<p v-for="\u005frow of items" :key="capture(holder._for_item0.value)">ok</p>"#,
                &[r#"(\u005frow) => (_ctx.capture(_ctx.holder._for_item0.value))"#],
            ),
            (
                r#"<p v-for="\u005frow of items" :key="(_row => capture(_row + '_for_item0.value'))('shadow')">ok</p>"#,
                &[
                    r#"(\u005frow) => ((_row => _ctx.capture(_row + '_for_item0.value'))('shadow'))"#,
                ],
            ),
            (
                r#"<p v-for="row of items" :key="capture('_for_item0.value')">ok</p>"#,
                &[r#"(row) => (_ctx.capture('_for_item0.value'))"#],
            ),
            (
                r#"<p v-for="(row, \u005fkey, \u005findex) of items" :key="_key + ':' + _index">ok</p>"#,
                &[r#"(row, \u005fkey, \u005findex) => (_key + ':' + _index)"#],
            ),
            (
                r#"<div v-for="\u005fouter of groups" :key="_outer.id"><p v-for="\u005frow of _outer.items" :key="_outer.id + _row.id">ok</p></div>"#,
                &[
                    r#"(\u005fouter) => (_outer.id)"#,
                    r#"(\u005frow) => (_for_item0.value.id + _row.id)"#,
                ],
            ),
            (
                r#"<p v-for="\u005frow of items" :key="'_for_item0.value'">ok</p>"#,
                &[r#"(\u005frow) => ('_for_item0.value')"#],
            ),
            (
                r#"<p v-for="\u005frow of items" :key="(_for_item0 => capture(_for_item0.value))({value: 'shadow'})">ok</p>"#,
                &[
                    r#"(\u005frow) => ((_for_item0 => _ctx.capture(_for_item0.value))({value: 'shadow'}))"#,
                ],
            ),
            (
                r#"<div v-for="\u005fouter of items" :key="_outer.id"><Child><template #default="{ row: _row }"><p v-for="\u005finner of children" :key="_row + _inner">ok</p></template></Child></div>"#,
                &[
                    r#"(\u005fouter) => (_outer.id)"#,
                    r#"(\u005finner) => (_slotProps0.row + _inner)"#,
                ],
            ),
        ];
        for (source, expected) in cases {
            assert_key_functions(source, expected);
        }
        for alias in [
            "kind",
            "café",
            "種類",
            "行",
            "番号",
            "index番号",
            "e\u{301}",
            "a\u{200c}",
            "a\u{200d}",
            "𐐀",
        ] {
            let source =
                vize_carton::cstr!("<p v-for=\"{alias} of items\" :key=\"{alias}.id\">ok</p>");
            let expected = vize_carton::cstr!("({alias}) => ({alias}.id)");
            assert_key_functions(&source, &[expected.as_str()]);
        }
    }

    #[test]
    fn unicode_loop_keys_capture_their_own_parameters_and_parse_as_javascript() {
        let cases: [(&str, &[&str]); 11] = [
            (
                r#"<p v-for="kind in ['x', 'y']" :key="kind">{{ kind }}</p>"#,
                &["(kind) => (kind)"],
            ),
            (
                r#"<p v-for="café in ['x']" :key="café">{{ café }}</p>"#,
                &["(café) => (café)"],
            ),
            (
                r#"<p v-for="種類 in ['x', 'y']" :key="種類">{{ 種類 }}</p>"#,
                &["(種類) => (種類)"],
            ),
            (
                r#"<p v-for="行 in items" :key="行.id">{{ 行.id }}</p>"#,
                &["(行) => (行.id)"],
            ),
            (
                r#"<p v-for="(r, 番号) in items" :key="番号">{{ r.id }}</p>"#,
                &["(r, 番号) => (番号)"],
            ),
            (
                r#"<p v-for="(行, i) in items" :key="i">{{ 行.id }}</p>"#,
                &["(行, i) => (i)"],
            ),
            (
                r#"<p v-for="種類 in items" :key="種類.id + 種類別">{{ 種類 }}</p>"#,
                &["(種類) => (種類.id + _ctx.種類別)"],
            ),
            (
                r#"<p v-for="{ id: 行 } in items" :key="行">{{ 行 }}</p>"#,
                &["({ id: 行 }) => (行)"],
            ),
            (
                r#"<p v-for="行 in items" :key="\u884c.id">{{ 行.id }}</p>"#,
                &["(行) => (行.id)"],
            ),
            (
                r#"<p v-for="(行, 番号, index番号) in items" :key="index番号">{{ 行.id }}</p>"#,
                &["(行, 番号, index番号) => (index番号)"],
            ),
            (
                r#"<div v-for="外側 in items" :key="外側.id"><p v-for="内側 in 外側.children" :key="外側.id + 内側.id">{{ 内側.id }}</p></div>"#,
                &[
                    "(外側) => (外側.id)",
                    "(内側) => (_for_item0.value.id + 内側.id)",
                ],
            ),
        ];
        for (source, key) in cases {
            assert_key_functions(source, key);
        }
    }

    fn assert_key_functions(source: &str, key: &[&str]) {
        let allocator = vize_carton::Allocator::new();
        let result = crate::compile_vapor(&allocator, source, Default::default());
        assert!(
            result.error_messages.is_empty(),
            "{:?}",
            result.error_messages
        );
        let parsed = oxc_parser::Parser::new(
            allocator.as_oxc(),
            &result.code,
            oxc_span::SourceType::mjs(),
        )
        .parse();
        assert!(
            parsed.diagnostics.is_empty(),
            "{:?}\n{}",
            parsed.diagnostics,
            result.code
        );
        let mut keys = KeyFunctions {
            code: &result.code,
            functions: Default::default(),
        };
        oxc_ast_visit::Visit::visit_program(&mut keys, &parsed.program);
        assert_eq!(keys.functions.as_slice(), key, "{source}\n{}", result.code);
    }
}
