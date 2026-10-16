//! The producer owns syntax, source maps and explicit refusals together.

use super::*;
use oxc_ast::ast::{Expression, Statement};
use oxc_span::GetSpan;
use vize_l0::Allocator;
use vize_relief::{ExpressionNode, PropNode, TemplateChildNode};

fn value<'n, 'a>(root: &'n RootNode<'a>, index: usize) -> &'n SimpleExpressionNode<'a> {
    let TemplateChildNode::Element(element) = &root.children[0] else {
        panic!("original element");
    };
    let PropNode::Directive(directive) = &element.props[index] else {
        panic!("original directive");
    };
    let Some(ExpressionNode::Simple(node)) = &directive.exp else {
        panic!("original expression");
    };
    node
}

fn find<'f, 'a>(
    facts: &'f TemplateSyntaxFacts<'a>,
    source: &str,
    cooked: &str,
) -> &'f TemplateSyntaxRecord<'a> {
    facts
        .records_for(source)
        .expect("original source")
        .iter()
        .find(|record| record.cooked() == cooked)
        .expect("original observation")
}

#[test]
fn ordinary_capture_is_disabled_and_demand_reuses_exact_expression_and_slot_roots() {
    let arena = Allocator::new();
    let source = "<Panel :value=\"$listeners\" #default=\"{ value = fallback }\"/>";
    assert!(Parser::new(&arena, source).template_syntax_facts.is_none());
    let (_, _, _, ordinary_facts) = Parser::new(&arena, source).parse_inner();
    assert!(
        ordinary_facts.is_none(),
        "ordinary parsing constructs no sidecar packet"
    );
    let (ordinary, errors) = Parser::new(&arena, source).parse();
    let (root, demanded_errors, facts) =
        Parser::new(&arena, source).parse_with_template_syntax_facts();
    assert_eq!(format!("{ordinary:?}"), format!("{root:?}"));
    assert_eq!(format!("{errors:?}"), format!("{demanded_errors:?}"));
    assert!(errors.is_empty());
    let expression = value(&root, 0);
    let retained = expression
        .js_ast
        .expect("existing expression")
        .as_expression()
        .expect("expression role");
    let record = facts
        .syntax_for(
            root.source,
            JsSpan::new(expression.loc.span.start, expression.loc.span.end),
            expression.content,
        )
        .expect("exact expression");
    let TemplateSyntax::Expression(Some(captured)) = record.syntax() else {
        panic!("expression role")
    };
    assert!(
        core::ptr::eq(retained.ast, captured.ast),
        "no clone or second parse"
    );
    let slot = value(&root, 1);
    let actual = slot.js_ast.expect("existing slot");
    let TemplateSyntax::Slot(Some(captured)) = find(&facts, source, slot.content).syntax() else {
        panic!("slot role")
    };
    assert!(
        core::ptr::eq(actual.ast, captured.ast),
        "same original parameter goal"
    );
}

#[test]
fn full_escaped_tokens_and_attribute_and_text_entities_have_authored_maps() {
    let arena = Allocator::new();
    let source = r#"<p :a="&#36;listeners" :b="\u0024listeners">{{ &#36;listeners }}</p>"#;
    let (root, errors, facts) = Parser::new(&arena, source).parse_with_template_syntax_facts();
    assert!(errors.is_empty(), "{errors:?}");
    let records = facts.records_for(root.source).expect("source");
    assert_eq!(records.len(), 3);
    for record in records {
        let TemplateSyntax::Expression(Some(js)) = record.syntax() else {
            panic!("complete expression")
        };
        let Expression::Identifier(id) = js.ast else {
            panic!("identifier")
        };
        assert_eq!(id.name.as_str(), "$listeners");
        let authored = record
            .authored_span(id.span)
            .expect("complete original token");
        assert!(matches!(
            authored.slice(source),
            "&#36;listeners" | "\\u0024listeners"
        ));
        assert_eq!(record.source().expect("map").text(), js.raw);
    }
    let copied = source.to_owned();
    assert!(facts.records_for(&copied).is_none());
    let first = &records[0];
    assert!(
        facts
            .syntax_for(&copied, first.span(), first.cooked())
            .is_none()
    );
    assert!(
        facts
            .syntax_for(
                source,
                JsSpan::new(first.span().start + 1, first.span().end),
                first.cooked()
            )
            .is_none()
    );
    assert!(facts.syntax_for(source, first.span(), "other").is_none());
}

#[test]
fn for_and_statement_handlers_keep_distinct_original_owners() {
    let arena = Allocator::new();
    let source = "<p v-for=\"($listeners, key) in &#36;listeners\" @click=\"const x = 1; use(&#36;listeners)\"/>";
    let (root, errors, facts) = Parser::new(&arena, source).parse_with_template_syntax_facts();
    assert!(errors.is_empty(), "{errors:?}");
    assert!(value(&root, 0).js_ast.is_none(), "For did not attempt Expr");
    assert!(
        value(&root, 1).js_ast.is_none(),
        "a failed statement expression does not retain an Expr root"
    );
    let TemplateSyntax::For(owner) =
        find(&facts, source, "($listeners, key) in $listeners").syntax()
    else {
        panic!("original For owner")
    };
    assert!(owner.hole().is_none(), "{owner:?}");
    let aliases = owner.aliases().expect("aliases").expect("parameter owner");
    assert_eq!(aliases.parameters().expect("actual parameters").len(), 2);
    let collection = owner
        .collection()
        .expect("collection")
        .expect("expression owner");
    let ast = collection.expression().expect("actual collection");
    assert_eq!(
        collection
            .authored_span(ast.span())
            .expect("original collection token")
            .slice(source),
        "&#36;listeners"
    );
    let TemplateSyntax::Handler(owner) =
        find(&facts, source, "const x = 1; use($listeners)").syntax()
    else {
        panic!("original handler owner")
    };
    assert!(owner.hole().is_none(), "{owner:?}");
    assert_eq!(
        owner.body().expect("actual original body").statements.len(),
        2
    );
    assert_eq!(owner.source().text(), "const x = 1; use($listeners)");
    let Statement::ExpressionStatement(statement) = &owner.body().expect("body").statements[1]
    else {
        panic!("original expression statement")
    };
    let Expression::CallExpression(call) = &statement.expression else {
        panic!("original call")
    };
    let Some(Expression::Identifier(identifier)) = call.arguments[0].as_expression() else {
        panic!("original identifier argument")
    };
    assert_eq!(
        owner
            .authored_span(identifier.span)
            .expect("wrapper and entity projection")
            .slice(source),
        "&#36;listeners"
    );
}

#[test]
fn handler_expressions_preserve_functions_callbacks_objects_and_original_offsets() {
    let arena = Allocator::new();
    let source = "<p @a=\"function(event) { use(&#36;listeners) }\" @b=\"{ ...$listeners }\" @c=\"() => use($listeners)\"/>";
    let (ordinary, ordinary_errors) = Parser::new(&arena, source).parse();
    let (root, errors, facts) = Parser::new(&arena, source).parse_with_template_syntax_facts();
    assert_eq!(format!("{ordinary:?}"), format!("{root:?}"));
    assert_eq!(format!("{ordinary_errors:?}"), format!("{errors:?}"));
    assert!(errors.is_empty(), "{errors:?}");
    for index in 0..3 {
        let node = value(&root, index);
        let original = node
            .js_ast
            .expect("original successful expression")
            .as_expression()
            .expect("expression role");
        let record = facts
            .syntax_for(
                root.source,
                JsSpan::new(node.loc.span.start, node.loc.span.end),
                node.content,
            )
            .expect("original selected event");
        let TemplateSyntax::HandlerExpression(captured) = record.syntax() else {
            panic!("a successful handler expression cannot become a statement body")
        };
        assert!(
            core::ptr::eq(original.ast, captured.ast),
            "reuse the exact original arena expression"
        );
        assert_eq!(original.raw, captured.raw);
        assert_eq!(
            record
                .authored_span(captured.ast.span())
                .expect("authored full expression"),
            node.loc.span
        );
    }
    assert!(matches!(
        value(&root, 0).js_ast.unwrap().as_expression().unwrap().ast,
        Expression::FunctionExpression(_)
    ));
    assert!(matches!(
        value(&root, 1).js_ast.unwrap().as_expression().unwrap().ast,
        Expression::ObjectExpression(_)
    ));
    assert!(matches!(
        value(&root, 2).js_ast.unwrap().as_expression().unwrap().ast,
        Expression::ArrowFunctionExpression(_)
    ));
}

#[test]
fn legacy_parameter_roles_are_selected_only_for_semantic_hosts() {
    let arena = Allocator::new();
    let source = "<Panel slot-scope=\"{ value = &#36;listeners }\"/><template scope=\"$listeners\"/><div scope=\"unrelated\" title=\"$listeners\"/>";
    let (root, errors, facts) = Parser::new(&arena, source).parse_with_template_syntax_facts();
    assert!(errors.is_empty(), "{errors:?}");
    let records = facts.records_for(root.source).expect("source");
    assert_eq!(records.len(), 2);
    for record in records {
        let TemplateSyntax::LegacySlot(owner) = record.syntax() else {
            panic!("parameter owner")
        };
        assert!(owner.hole().is_none(), "{owner:?}");
        let parameters = owner.parameters().expect("retained parameters");
        assert_eq!(parameters.len(), 1);
        let pattern = &parameters[0].pattern;
        let span = record
            .authored_span(pattern.span())
            .expect("authored parameter extent");
        assert!(matches!(
            span.slice(source),
            "{ value = &#36;listeners }" | "$listeners"
        ));
    }
}

#[test]
fn own_and_inherited_v_pre_discard_syntax_even_before_the_pre_attribute() {
    let arena = Allocator::new();
    let source = "<Panel slot-scope=\"$listeners\" :value=\"$listeners\" v-pre @click=\"$listeners\"><template scope=\"$listeners\">{{ $listeners }}</template></Panel><p>{{ outside }}</p>";
    let (root, errors, facts) = Parser::new(&arena, source).parse_with_template_syntax_facts();
    assert!(errors.is_empty(), "{errors:?}");
    let records = facts.records_for(root.source).expect("source");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].cooked(), "outside");
}

#[test]
fn failed_and_over_budget_goals_remain_observed_refusals() {
    let arena = Allocator::new();
    let source = "<p v-for=\"item in\" @click=\"const x =\" :value=\"value +\"/><Panel #default=\"{ value: }\"/><template scope=\"{ value: }\"/>";
    let (root, errors, facts) = Parser::new(&arena, source).parse_with_template_syntax_facts();
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(facts.records_for(root.source).expect("source").len(), 5);
    assert!(
        matches!(find(&facts, source, "item in").syntax(), TemplateSyntax::For(owner) if owner.hole().is_some())
    );
    assert!(
        matches!(find(&facts, source, "const x =").syntax(), TemplateSyntax::Handler(owner) if owner.hole().is_some())
    );
    assert!(matches!(
        find(&facts, source, "value +").syntax(),
        TemplateSyntax::Expression(None)
    ));
    for record in facts
        .records_for(source)
        .expect("source")
        .iter()
        .filter(|record| record.cooked() == "{ value: }")
    {
        match record.syntax() {
            TemplateSyntax::Slot(Some(owner)) => {
                assert!(owner.as_slot_parameters().expect("slot role").is_err())
            }
            TemplateSyntax::LegacySlot(owner) => assert!(owner.hole().is_some()),
            _ => panic!("failed original parameter role"),
        }
    }
    assert!(
        facts
            .records_for(source)
            .expect("complete refusals")
            .iter()
            .all(|record| record.authored_span(JsSpan::new(0, 0)).is_none())
    );
    let source = format!("<p @click=\"{}\"/>", "consume(value);".repeat(20));
    let (_, _, facts) = Parser::new(&arena, &source).parse_with_template_syntax_facts();
    let TemplateSyntax::Handler(owner) = facts.records_for(&source).expect("source")[0].syntax()
    else {
        panic!("handler refusal")
    };
    assert_eq!(
        owner.hole(),
        Some(vize_l1::embed::syntax::EmbedHole::TokenBudget)
    );
    assert!(owner.body().is_none());
}

#[test]
fn stale_cooked_shorthand_cannot_borrow_an_unrelated_authored_map() {
    let arena = Allocator::new();
    let source = "<p :foo-bar/>";
    let (_, errors, facts) = Parser::new(&arena, source).parse_with_template_syntax_facts();
    assert!(errors.is_empty());
    let record = find(&facts, source, "fooBar");
    assert!(record.source().is_none());
    assert!(matches!(
        record.syntax(),
        TemplateSyntax::Refused(TemplateSyntaxRole::Expression)
    ));
    assert!(record.authored_span(JsSpan::new(0, 6)).is_none());
}

#[test]
fn record_budget_refuses_the_complete_packet_instead_of_returning_a_prefix() {
    let arena = Allocator::new();
    let source = "<p :value=\"outside\"/>".repeat(TEMPLATE_SYNTAX_RECORD_LIMIT + 1);
    let (root, errors, facts) = Parser::new(&arena, &source).parse_with_template_syntax_facts();
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(root.children.len(), TEMPLATE_SYNTAX_RECORD_LIMIT + 1);
    assert!(!facts.is_complete_for(root.source));
    assert!(facts.records_for(root.source).is_none());
    assert!(facts.records.is_empty(), "discard all partial observations");
}
