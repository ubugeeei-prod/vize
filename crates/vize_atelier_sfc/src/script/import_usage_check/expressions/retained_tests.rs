//! Retained reads must preserve the existing import-retention visitor's semantics.

use super::{extract_identifiers_from_js_expression, extract_identifiers_from_simple_expression};
use vize_armature::parse;
use vize_atelier_core::retained::js_module_compatible;
use vize_atelier_core::{ExpressionNode, PropNode, SimpleExpressionNode, TemplateChildNode};
use vize_l0::{Allocator, FxHashSet, String, ToCompactString, Vec as ArenaVec};

fn with_template_expression(
    source: &str,
    inspect: impl for<'a> FnOnce(&mut SimpleExpressionNode<'a>, &'a Allocator),
) {
    let allocator = Allocator::new();
    let (mut root, errors) = parse(&allocator, source);
    assert!(errors.is_empty(), "template errors: {errors:?}");
    let expression = match &mut root.children[0] {
        TemplateChildNode::Interpolation(interpolation) => &mut interpolation.content,
        TemplateChildNode::Element(element) => {
            let PropNode::Directive(directive) = &mut element.props[0] else {
                panic!("expected the fixture's directive");
            };
            directive.exp.as_mut().expect("fixture expression")
        }
        _ => panic!("expected the fixture's expression"),
    };
    let ExpressionNode::Simple(node) = expression else {
        panic!("expected the fixture's simple expression");
    };
    inspect(node, &allocator);
}

fn with_expression(
    content: &str,
    inspect: impl for<'a> FnOnce(&mut SimpleExpressionNode<'a>, &'a Allocator),
) {
    let mut source = String::with_capacity(content.len() + 6);
    source.push_str("{{ ");
    source.push_str(content);
    source.push_str(" }}");
    with_template_expression(&source, inspect);
}

fn read_ids(node: &SimpleExpressionNode<'_>) -> FxHashSet<String> {
    let mut ids = FxHashSet::default();
    extract_identifiers_from_simple_expression(node, &mut ids);
    ids
}

fn legacy_ids(content: &str) -> FxHashSet<String> {
    let mut ids = FxHashSet::default();
    extract_identifiers_from_js_expression(content.trim(), &mut ids);
    ids
}

fn expected_ids(names: &[&str]) -> FxHashSet<String> {
    names.iter().map(|name| name.to_compact_string()).collect()
}

fn assert_retained_parity(content: &str, expected: &[&str], compatible: bool) {
    with_expression(content, |node, _| {
        let retained = node.js_ast.as_ref().expect("complete expression retained");
        assert_eq!(retained.raw, node.content);
        assert_eq!(js_module_compatible(retained), compatible, "{content}");
        let actual = read_ids(node);
        assert_eq!(actual, legacy_ids(node.content), "{content}");
        assert_eq!(actual, expected_ids(expected), "{content}");
        node.js_ast = None;
        assert_eq!(read_ids(node), actual, "without retained AST: {content}");
    });
}

#[test]
fn retained_arrows_preserve_scopes_defaults_computed_keys_and_rest_reads() {
    for (content, expected) in [
        (
            "items.map(({ value = fallback, [key]: named = compute(seed), ...rest }, index = start) => value + named + rest[index] + outer)",
            &[
                "items", "fallback", "key", "compute", "seed", "start", "outer",
            ][..],
        ),
        (
            "([first = fallback, ...rest], ...args) => first + rest[0] + args[0] + outside",
            &["fallback", "outside"][..],
        ),
        (
            "((outer) => [outer, (inner) => inner + outer + external])(seed)",
            &["external", "seed"][..],
        ),
        (
            "items.map(item => item.value) + item",
            &["items", "item"][..],
        ),
        // Preserve the current visitor: all parameter bindings enter scope
        // before defaults are walked, including later parameters.
        ("(later = outer, outer = seed) => later", &["seed"][..]),
    ] {
        assert_retained_parity(content, expected, true);
    }
}

#[test]
fn retained_reads_keep_builtin_filtering_and_current_function_semantics() {
    assert_retained_parity(
        "Math.max(value, Number(other)) + console.log($event)",
        &["value", "other"],
        true,
    );
    assert_retained_parity(
        "$slots.foo + $emit('x') + _ctx.value",
        &["$slots", "$emit"],
        true,
    );
    // The existing visitor scopes arrow parameters only. This change must
    // not quietly repair regular-function parameter reads.
    assert_retained_parity(
        "function named(local) { return local + external }",
        &["local", "external"],
        false,
    );
}

#[test]
fn retained_reads_match_unicode_escaped_identifiers_and_closed_comments() {
    assert_retained_parity("測定(値) + café", &["測定", "値", "café"], true);
    assert_retained_parity(r"\u0061 + other", &["a", "other"], true);
    assert_retained_parity(
        "items /* inside */ .map(item => item + external) /* tail */",
        &["items", "external"],
        true,
    );
    assert_retained_parity("((value)) /* tail */   ", &["value"], true);
}

#[test]
fn decoded_directive_content_has_matching_retained_bytes_and_reads() {
    with_template_expression(
        r#"<div :data-x="greeting &amp;&amp; 測定(&quot;x&quot;)" />"#,
        |node, _| {
            assert_eq!(node.content, "greeting && 測定(\"x\")");
            let retained = node.js_ast.as_ref().expect("decoded expression retained");
            assert_eq!(retained.raw, node.content);
            let actual = read_ids(node);
            assert_eq!(actual, expected_ids(&["greeting", "測定"]));
            assert_eq!(actual, legacy_ids(node.content));
            node.js_ast = None;
            assert_eq!(read_ids(node), actual);
        },
    );
}

#[test]
fn changed_content_rejects_a_retained_ast_even_for_equal_length_edits() {
    with_expression("before + branch", |node, _| {
        let retained = node.js_ast.as_ref().expect("original expression retained");
        assert_eq!(retained.raw, "before + branch");
        node.content = "edited + branch";
        assert_eq!(node.content.len(), retained.raw.len());
        assert_eq!(read_ids(node), expected_ids(&["edited", "branch"]));
        assert_eq!(read_ids(node), legacy_ids(node.content));
        node.content = "";
        assert!(read_ids(node).is_empty());
    });
}

#[test]
fn parsed_identifiers_static_and_simple_identifier_precedence_is_unchanged() {
    with_expression("Math.max(value, other)", |node, allocator| {
        assert!(node.js_ast.is_some());
        let mut identifiers = ArenaVec::new_in(&allocator);
        identifiers.push("explicit");
        identifiers.push("Math");
        node.identifiers = Some(identifiers);
        node.is_static = true;
        assert_eq!(read_ids(node), expected_ids(&["explicit", "Math"]));
        node.identifiers = None;
        assert!(read_ids(node).is_empty());
        node.is_static = false;
        node.content = "Math";
        assert_eq!(read_ids(node), expected_ids(&["Math"]));
    });
    with_expression("Math", |node, _| {
        assert!(node.js_ast.is_some());
        assert_eq!(read_ids(node), expected_ids(&["Math"]));
        node.js_ast = None;
        assert_eq!(read_ids(node), expected_ids(&["Math"]));
    });
}

#[test]
fn missing_retained_ast_preserves_multistatement_and_invalid_fallbacks() {
    for content in [
        "save(value); count++",
        "item of items",
        "value +",
        "value // trailing comment",
        "value /* unfinished",
    ] {
        with_expression(content, |node, _| {
            assert!(
                node.js_ast.is_none(),
                "must not retain partial AST: {content}"
            );
            assert_eq!(read_ids(node), legacy_ids(node.content), "{content}");
            assert!(read_ids(node).is_empty(), "legacy fallback: {content}");
        });
    }
}

#[test]
fn module_goal_and_typed_expressions_preserve_the_legacy_parse_result() {
    for content in [
        "((eval) => eval + outer)(seed)",
        "((let) => let + outer)(seed)",
        "(function () { return outer + 010 })()",
        "(function (local) { with (scope) { return local + outer } })(seed)",
        "value as Model",
        "value satisfies Model",
        "value! + other",
        "'<!--' + outer",
        "'-->' + outer",
    ] {
        with_expression(content, |node, _| {
            if let Some(retained) = &node.js_ast {
                assert!(
                    !js_module_compatible(retained),
                    "must keep fallback: {content}"
                );
            }
            let actual = read_ids(node);
            assert_eq!(
                actual,
                legacy_ids(node.content),
                "module fallback: {content}"
            );
            node.js_ast = None;
            assert_eq!(read_ids(node), actual, "without AST: {content}");
        });
    }
}
