//! P1-5 retained-expression counter law: parse-at-template-parse.
//!
//! One template parse must attempt exactly one retained oxc parse per
//! distinct non-static expression node it builds — `davinci.expr.parses ==
//! distinct expressions`, no re-parses — and must retain the AST exactly
//! where the content is one complete expression.
//!
//! The counter lives on the process-global profiler, so this file holds a
//! single `#[test]` (its own test binary, no concurrent recorder): parallel
//! tests in the same binary would interleave increments.

use oxc_ast::ast::Expression;
use vize_armature::parse;
use vize_relief::{
    ExpressionNode, JsExpression, PropNode, SimpleExpressionNode, TemplateChildNode,
};
use vize_s0::profiler::global_profiler;
use vize_s0::{Allocator, String};

/// Nine distinct non-static expression nodes, exercising every retained-parse
/// shape: an entity-decoded value (arena-copy `raw`), a dynamic argument, a
/// multi-statement v-on body (`None`), a v-for value (`None`), a same-name
/// shorthand, and two interpolations.
const TEMPLATE: &str = concat!(
    "<div :id=\"a &amp;&amp; b\" :[key]=\"v\" @click=\"count++; other++\" v-if=\"ok\">",
    "{{ msg }}<span v-for=\"item of items\" :title>{{ item.name }}</span></div>"
);

/// Distinct expressions in `TEMPLATE`, by construction: `:id` value, `[key]`
/// argument, `:[key]` value, `@click` body, `v-if` value, `{{ msg }}`,
/// `v-for` value, `:title` shorthand value, `{{ item.name }}`.
const DISTINCT_EXPRESSIONS: u64 = 9;

fn simple<'n, 'a>(expr: &'n ExpressionNode<'a>) -> &'n SimpleExpressionNode<'a> {
    match expr {
        ExpressionNode::Simple(node) => node,
        ExpressionNode::Compound(_) => panic!("parser only builds simple expression nodes"),
    }
}

fn retained<'n, 'a>(expr: &'n ExpressionNode<'a>) -> &'n JsExpression<'a> {
    simple(expr)
        .js_ast
        .as_ref()
        .expect("expression content is one complete expression, so its AST is retained")
}

#[test]
fn template_parse_retains_each_expression_ast_exactly_once() {
    let profiler = global_profiler();
    profiler.clear();
    profiler.enable();

    let allocator = Allocator::new();
    let (root, errors) = parse(&allocator, TEMPLATE);

    profiler.disable();
    let counters = profiler.counter_summary();
    let entry = counters
        .entries
        .iter()
        .find(|entry| entry.name == "davinci.expr.parses")
        .expect("the retained parse site records davinci.expr.parses");
    // The counter law under parse-at-template-parse: one increment per
    // distinct expression — an attempt is a parse, so the two `None` shapes
    // count too, and nothing is ever parsed twice.
    assert_eq!(errors.len(), 0);
    assert_eq!(entry.samples, DISTINCT_EXPRESSIONS);
    assert_eq!(entry.total, DISTINCT_EXPRESSIONS);

    let TemplateChildNode::Element(div) = &root.children[0] else {
        panic!("root child is the div element");
    };
    assert_eq!(div.props.len(), 4);

    // `:id="a &amp;&amp; b"`: entities decode before parsing, so `raw` is the
    // arena copy of the decoded content, not a source slice.
    let PropNode::Directive(bind_id) = &div.props[0] else {
        panic!("props[0] is the :id directive");
    };
    let id_value = retained(bind_id.exp.as_ref().expect(":id has a value"));
    assert_eq!(id_value.raw, "a && b");
    assert!(matches!(id_value.ast, Expression::LogicalExpression(_)));
    let id_arg = simple(bind_id.arg.as_ref().expect(":id has an argument"));
    assert!(id_arg.js_ast.is_none());

    // `:[key]="v"`: a dynamic argument is an expression; a static one is not.
    let PropNode::Directive(bind_dynamic) = &div.props[1] else {
        panic!("props[1] is the :[key] directive");
    };
    let dynamic_arg = retained(bind_dynamic.arg.as_ref().expect(":[key] has an argument"));
    assert_eq!(dynamic_arg.raw, "key");
    assert!(matches!(dynamic_arg.ast, Expression::Identifier(_)));
    let dynamic_value = retained(bind_dynamic.exp.as_ref().expect(":[key] has a value"));
    assert_eq!(dynamic_value.raw, "v");

    // `@click="count++; other++"`: two statements are not one expression, so
    // no AST is retained (the attempt still counted above) — a partial parse
    // covering only `count++` must never be stored.
    let PropNode::Directive(on_click) = &div.props[2] else {
        panic!("props[2] is the @click directive");
    };
    let click = simple(on_click.exp.as_ref().expect("@click has a value"));
    assert!(click.js_ast.is_none());
    assert_eq!(click.content, "count++; other++");

    // `v-if="ok"`.
    let PropNode::Directive(v_if) = &div.props[3] else {
        panic!("props[3] is the v-if directive");
    };
    assert_eq!(
        retained(v_if.exp.as_ref().expect("v-if has a value")).raw,
        "ok"
    );

    // `{{ msg }}`.
    let TemplateChildNode::Interpolation(msg) = &div.children[0] else {
        panic!("children[0] is the msg interpolation");
    };
    let msg_expr = retained(&msg.content);
    assert_eq!(msg_expr.raw, "msg");
    assert!(matches!(msg_expr.ast, Expression::Identifier(_)));

    let TemplateChildNode::Element(span) = &div.children[1] else {
        panic!("children[1] is the span element");
    };
    assert_eq!(span.props.len(), 2);

    // `v-for="item of items"`: not one expression (`of` ends it), so no AST.
    let PropNode::Directive(v_for) = &span.props[0] else {
        panic!("span props[0] is the v-for directive");
    };
    let for_value = simple(v_for.exp.as_ref().expect("v-for has a value"));
    assert!(for_value.js_ast.is_none());
    assert_eq!(for_value.content, "item of items");

    // `:title` same-name shorthand synthesizes the value expression.
    let PropNode::Directive(bind_title) = &span.props[1] else {
        panic!("span props[1] is the :title directive");
    };
    let title = retained(
        bind_title
            .exp
            .as_ref()
            .expect("shorthand synthesizes a value"),
    );
    assert_eq!(title.raw, "title");

    // `{{ item.name }}`.
    let TemplateChildNode::Interpolation(member) = &span.children[0] else {
        panic!("span children[0] is the item.name interpolation");
    };
    let member_expr = retained(&member.content);
    assert_eq!(member_expr.raw, "item.name");
    assert!(matches!(
        member_expr.ast,
        Expression::StaticMemberExpression(_)
    ));

    // A pathologically nested expression (the #956 stack-overflow shape) is
    // refused by the shared nesting guard before oxc ever runs: no crash, no
    // retained AST, and no counter increment — the same refusal every legacy
    // parse site applies before creating its parse arena, so "parsed at most
    // once" holds as "never" and the counter still equals the nine
    // guard-accepted expressions above.
    profiler.clear();
    profiler.enable();
    let mut deep_source = String::with_capacity(2_750);
    deep_source.push_str("<li :key=\"S");
    for _ in 0..2_704 {
        deep_source.push('[');
    }
    deep_source.push_str("\">item</li>");
    let deep_allocator = Allocator::new();
    let (deep_root, _deep_errors) = parse(&deep_allocator, &deep_source);
    profiler.disable();
    let TemplateChildNode::Element(li) = &deep_root.children[0] else {
        panic!("root child is the li element");
    };
    let PropNode::Directive(bind_key) = &li.props[0] else {
        panic!("li props[0] is the :key directive");
    };
    let key = simple(bind_key.exp.as_ref().expect(":key has a value"));
    assert!(key.js_ast.is_none());
    let refused = profiler
        .counter_summary()
        .entries
        .iter()
        .find(|entry| entry.name == "davinci.expr.parses")
        .map(|entry| entry.total);
    assert_eq!(refused, None);
    profiler.clear();
}
