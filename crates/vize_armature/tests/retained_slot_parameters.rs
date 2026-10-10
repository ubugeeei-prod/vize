//! The slot producer retains one original parameter goal, including failures.
//! One test owns the process-global profiler and allocation windows; ordinary
//! expression counter laws remain in their existing separate test binary.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "tests assert by panicking"
)]

#[path = "retained_slot_parameters/costs.rs"]
mod costs;

use davinci_harness::alloc::{CountingAllocator, mark_installed};
use oxc_ast::ast::{BindingPattern, Expression, FormalParameters};
use oxc_span::{GetSpan, SourceType};
use vize_armature::parse;
use vize_l0::expression_guard::MAX_EXPRESSION_NESTING_DEPTH;
use vize_l0::profiler::global_profiler;
use vize_l0::{Allocator, String};
use vize_relief::{ExpressionNode, PropNode, RootNode, SimpleExpressionNode, TemplateChildNode};

#[global_allocator]
static ALLOCATOR: CountingAllocator<std::alloc::System> = CountingAllocator::system();

fn value<'n, 'a>(root: &'n RootNode<'a>, index: usize) -> &'n SimpleExpressionNode<'a> {
    let TemplateChildNode::Element(element) = &root.children[0] else {
        panic!("original component");
    };
    let PropNode::Directive(directive) = &element.props[index] else {
        panic!("original directive");
    };
    let Some(ExpressionNode::Simple(expression)) = directive.exp.as_ref() else {
        panic!("original directive value");
    };
    expression
}

fn template(pattern: &str) -> String {
    let mut source = String::with_capacity(pattern.len() + 20);
    source.push_str("<Panel v-slot=\"");
    source.push_str(pattern);
    source.push_str("\" />");
    source
}

fn parse_counted<'a>(arena: &'a Allocator, source: &'a str, attempts: u64) -> RootNode<'a> {
    let profiler = global_profiler();
    profiler.clear();
    profiler.enable();
    let (root, errors) = parse(arena, source);
    profiler.disable();
    assert!(errors.is_empty(), "template diagnostics: {errors:?}");
    let summary = profiler.counter_summary();
    let actual = summary
        .entries
        .iter()
        .find(|entry| entry.name == "davinci.expr.parses");
    if attempts == 0 {
        assert!(
            actual.is_none(),
            "the parameter parser must never be entered"
        );
    } else {
        let actual = actual.expect("the original goal is attempted");
        assert_eq!(actual.samples, attempts);
        assert_eq!(actual.total, attempts);
    }
    root
}

fn parameters<'a>(node: &SimpleExpressionNode<'a>) -> &'a FormalParameters<'a> {
    let retained = node.js_ast.expect("even a refused slot keeps its role");
    assert_eq!(retained.raw, node.content);
    assert!(retained.as_expression().is_none());
    retained
        .as_slot_parameters()
        .expect("original slot role")
        .expect("complete parameter goal")
}

fn typed_defaults_and_rest() {
    let pattern = "  { [key]: value = fallback(outer), nested: [first, ...tail], ...rest }: Props, index = outer, ...残り: Item[] /* tail */";
    let source = template(pattern);
    let arena = Allocator::new();
    let root = parse_counted(&arena, &source, 1);
    let node = value(&root, 0);
    let retained = node.js_ast.expect("slot payload");
    assert_eq!(retained.raw, pattern);
    assert_eq!(
        retained.raw.as_ptr(),
        source[node.loc.span.start as usize..].as_ptr()
    );
    let parameters = parameters(node);
    assert_eq!(parameters.items.len(), 2);
    assert_eq!(parameters.span.start, 2);
    assert_eq!(&pattern[parameters.span.end as usize..], " /* tail */");
    let first = &parameters.items[0];
    assert_eq!(
        first
            .type_annotation
            .as_ref()
            .expect("type")
            .span()
            .source_text(pattern),
        ": Props"
    );
    let BindingPattern::ObjectPattern(object) = &first.pattern else {
        panic!("object binding");
    };
    assert!(object.properties[0].computed);
    assert_eq!(object.properties[0].key.span().source_text(pattern), "key");
    let BindingPattern::AssignmentPattern(default) = &object.properties[0].value else {
        panic!("binding default");
    };
    assert_eq!(default.left.span().source_text(pattern), "value");
    assert_eq!(default.right.span().source_text(pattern), "fallback(outer)");
    let BindingPattern::ArrayPattern(array) = &object.properties[1].value else {
        panic!("nested array");
    };
    assert_eq!(
        array.elements[0]
            .as_ref()
            .expect("declaration")
            .span()
            .source_text(pattern),
        "first"
    );
    assert_eq!(
        array
            .rest
            .as_ref()
            .expect("array rest")
            .argument
            .span()
            .source_text(pattern),
        "tail"
    );
    assert_eq!(
        object
            .rest
            .as_ref()
            .expect("object rest")
            .argument
            .span()
            .source_text(pattern),
        "rest"
    );
    assert_eq!(
        parameters.items[1]
            .initializer
            .as_ref()
            .expect("default")
            .span()
            .source_text(pattern),
        "outer"
    );
    let rest = parameters.rest.as_ref().expect("top-level rest");
    assert_eq!(rest.rest.argument.span().source_text(pattern), "残り");
    assert_eq!(
        rest.type_annotation
            .as_ref()
            .expect("rest type")
            .span()
            .source_text(pattern),
        ": Item[]"
    );
}

fn entity_decoded_coordinates_are_display_relative() {
    let authored = "{ &#x30e9;ベル: value = fallback(&quot;x&quot;), ...rest }: Props";
    let decoded = "{ ラベル: value = fallback(\"x\"), ...rest }: Props";
    let source = template(authored);
    let arena = Allocator::new();
    let root = parse_counted(&arena, &source, 1);
    let node = value(&root, 0);
    let retained = node.js_ast.expect("decoded slot payload");
    assert_eq!(retained.raw, decoded);
    assert_eq!(node.content, decoded);
    assert_ne!(
        retained.raw.as_ptr(),
        source[node.loc.span.start as usize..].as_ptr()
    );
    assert_eq!(node.loc.span.slice(&source), authored);
    let parameters = parameters(node);
    let BindingPattern::ObjectPattern(object) = &parameters.items[0].pattern else {
        panic!("decoded object binding");
    };
    let BindingPattern::AssignmentPattern(default) = &object.properties[0].value else {
        panic!("decoded default");
    };
    let declaration = default.left.span();
    assert_eq!(
        declaration.start as usize,
        decoded.find("value").expect("decoded declaration")
    );
    assert_eq!(declaration.source_text(decoded), "value");
    assert_eq!(default.right.span().source_text(decoded), "fallback(\"x\")");
    // The carrier owns decoded spans, not an entity-to-authored source map.
    assert_ne!(
        node.loc.span.start + declaration.start,
        source.find("value").expect("authored declaration") as u32
    );
}

fn malformed_goals_retain_complete_diagnostics() {
    for pattern in [
        "{ value: }",
        "value + other",
        "value => next",
        "...rest, next",
        "props?: Props, required: Props",
    ] {
        let source = template(pattern);
        let arena = Allocator::new();
        let root = parse_counted(&arena, &source, 1);
        let retained = value(&root, 0)
            .js_ast
            .expect("parser failure retains its slot goal");
        assert_eq!(retained.raw, pattern);
        assert!(retained.as_expression().is_none());
        let errors = retained
            .as_slot_parameters()
            .expect("slot role")
            .expect_err(pattern);
        assert!(!errors.is_empty());
        // The independent oracle runs outside the template's profiler window.
        let oracle_arena = Allocator::new();
        let expected = oxc_parser::Parser::new(oracle_arena.as_oxc(), pattern, SourceType::ts())
            .parse_slot_parameters()
            .expect_err(pattern);
        assert_eq!(
            errors, &expected,
            "all messages, labels, codes and severities survive"
        );
    }
}

fn unsupported_tail_retains_a_refusal_after_one_attempt() {
    let source = template("props // tail");
    let arena = Allocator::new();
    let root = parse_counted(&arena, &source, 1);
    let retained = value(&root, 0)
        .js_ast
        .expect("whole-source refusal retains its role");
    assert_eq!(retained.raw, "props // tail");
    assert!(retained.as_expression().is_none());
    let errors = retained
        .as_slot_parameters()
        .expect("slot role")
        .expect_err("a line-comment tail is not admitted");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].message, "Unexpected token");
}

fn guards_preserve_refused_slot_role_without_a_parser_attempt() {
    let mut deep = String::with_capacity((MAX_EXPRESSION_NESTING_DEPTH + 1) * 2 + 5);
    for _ in 0..=MAX_EXPRESSION_NESTING_DEPTH {
        deep.push('[');
    }
    deep.push_str("value");
    for _ in 0..=MAX_EXPRESSION_NESTING_DEPTH {
        deep.push(']');
    }
    for (pattern, silent) in [("{ value", false), (deep.as_str(), true)] {
        let source = template(pattern);
        let arena = Allocator::new();
        let root = parse_counted(&arena, &source, 0);
        let retained = value(&root, 0)
            .js_ast
            .expect("refusal must prevent a consumer retry");
        assert_eq!(retained.raw, pattern);
        assert!(retained.as_expression().is_none());
        let errors = retained
            .as_slot_parameters()
            .expect("refused slot role")
            .expect_err("guard refusal");
        assert_eq!(errors.is_empty(), silent);
        if !silent {
            assert_eq!(errors[0].message, "mismatched expression delimiters");
        }
    }
}

fn roles_and_byte_identical_clones_remain_explicit() {
    let arena = Allocator::new();
    let root = parse_counted(&arena, "<Panel v-slot=\"props\" :value=\"props\" />", 2);
    let slot = value(&root, 0);
    let ordinary = value(&root, 1);
    assert!(parameters(slot).rest.is_none());
    let expression = ordinary.js_ast.expect("ordinary payload");
    assert!(expression.as_slot_parameters().is_none());
    assert!(matches!(
        expression.as_expression().expect("ordinary role").ast,
        Expression::Identifier(_)
    ));
    let mut stale = SimpleExpressionNode::from_node(slot);
    assert!(core::ptr::eq(
        stale.js_ast.expect("copied carrier").ast,
        slot.js_ast.expect("original carrier").ast
    ));
    stale.content = "other";
    assert_ne!(
        stale.js_ast.expect("original parse is not repaired").raw,
        stale.content
    );
    stale.content = "props";
    stale.js_ast = ordinary.js_ast;
    assert!(
        stale
            .js_ast
            .expect("foreign role")
            .as_slot_parameters()
            .is_none()
    );
}

#[test]
fn original_slot_producer_retains_roles_diagnostics_and_exact_attempt_counts() {
    mark_installed();
    typed_defaults_and_rest();
    entity_decoded_coordinates_are_display_relative();
    malformed_goals_retain_complete_diagnostics();
    unsupported_tail_retains_a_refusal_after_one_attempt();
    guards_preserve_refused_slot_role_without_a_parser_attempt();
    roles_and_byte_identical_clones_remain_explicit();
    costs::check();
    global_profiler().clear();
}
