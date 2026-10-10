use oxc_allocator::Allocator;
use oxc_ast::ast::{BindingPattern, Expression};
use oxc_span::{GetSpan, SourceType};

use crate::{
    Parser,
    config::{NoTokensLexerConfig, ParserConfig, RuntimeParserConfig, TokensParserConfig},
};

#[derive(Default)]
struct CustomConfig;
impl ParserConfig for CustomConfig {
    type LexerConfig = NoTokensLexerConfig;
    fn lexer_config(&self) -> Self::LexerConfig {
        NoTokensLexerConfig
    }
}

fn declaration<'a>(pattern: &'a BindingPattern<'a>) -> &'a str {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => identifier.name.as_str(),
        _ => panic!("authored control is a single declaration"),
    }
}

#[test]
fn direct_goal_preserves_every_original_parameter_part_and_span() {
    let allocator = Allocator::default();
    let source = "  { [key]: value = fallback(outer), nested: [first, ...tail], ...rest }: Props, index = outer, ...残り: Item[] /* tail */";
    let parameters =
        Parser::new(&allocator, source, SourceType::ts()).parse_slot_parameters().unwrap();
    assert_eq!(parameters.items.len(), 2);
    assert_eq!(parameters.span.start, 2);
    assert_eq!(&source[parameters.span.end as usize..], " /* tail */");
    let first = &parameters.items[0];
    assert_eq!(first.type_annotation.as_ref().unwrap().span().source_text(source), ": Props");
    let BindingPattern::ObjectPattern(object) = &first.pattern else {
        panic!("original object pattern");
    };
    let property = &object.properties[0];
    assert!(property.computed);
    assert_eq!(property.key.span().source_text(source), "key");
    let BindingPattern::AssignmentPattern(default) = &property.value else {
        panic!("original default");
    };
    assert_eq!(declaration(&default.left), "value");
    assert_eq!(default.right.span().source_text(source), "fallback(outer)");
    let BindingPattern::ArrayPattern(array) = &object.properties[1].value else {
        panic!("original nested array");
    };
    assert_eq!(declaration(array.elements[0].as_ref().unwrap()), "first");
    assert_eq!(declaration(&array.rest.as_ref().unwrap().argument), "tail");
    assert_eq!(declaration(&object.rest.as_ref().unwrap().argument), "rest");
    let second = &parameters.items[1];
    assert_eq!(declaration(&second.pattern), "index");
    assert_eq!(second.initializer.as_ref().unwrap().span().source_text(source), "outer");
    let rest = parameters.rest.as_ref().unwrap();
    assert_eq!(declaration(&rest.rest.argument), "残り");
    assert_eq!(rest.type_annotation.as_ref().unwrap().span().source_text(source), ": Item[]");
    assert_eq!(rest.rest.argument.span().source_text(source), "残り");

    let source = "{ slot = fallback }, props?: Props";
    let parameters =
        Parser::new(&allocator, source, SourceType::ts()).parse_slot_parameters().unwrap();
    let BindingPattern::ObjectPattern(object) = &parameters.items[0].pattern else {
        panic!("shorthand pattern");
    };
    assert!(object.properties[0].shorthand);
    let BindingPattern::AssignmentPattern(default) = &object.properties[0].value else {
        panic!("shorthand initializer must survive");
    };
    assert_eq!(declaration(&default.left), "slot");
    assert_eq!(default.right.span().source_text(source), "fallback");
    assert!(parameters.items[1].optional);

    // The new grammar does not reinterpret the ordinary expression cover tree.
    assert!(
        Parser::new(&allocator, "{ slot = fallback }", SourceType::ts())
            .parse_expression()
            .is_err()
    );
    assert!(matches!(
        Parser::new(&allocator, "slot", SourceType::ts()).parse_expression().unwrap(),
        Expression::Identifier(_)
    ));
}

#[test]
fn all_stock_and_custom_configurations_use_the_same_complete_original_goal() {
    let allocator = Allocator::default();
    let source = "\r\n{ value = fallback }: Props, ...rest: Item[]";
    let expected = format!(
        "{:?}",
        Parser::new(&allocator, source, SourceType::ts()).parse_slot_parameters().unwrap()
    );
    let tokens = Parser::new(&allocator, source, SourceType::ts())
        .with_config(TokensParserConfig)
        .parse_slot_parameters()
        .unwrap();
    assert_eq!(format!("{tokens:?}"), expected);
    for tokens in [false, true] {
        let runtime = Parser::new(&allocator, source, SourceType::ts())
            .with_config(RuntimeParserConfig::new(tokens))
            .parse_slot_parameters()
            .unwrap();
        assert_eq!(format!("{runtime:?}"), expected);
    }
    let custom = Parser::new(&allocator, source, SourceType::ts())
        .with_config(CustomConfig)
        .parse_slot_parameters()
        .unwrap();
    assert_eq!(format!("{custom:?}"), expected);
}

#[test]
fn direct_goal_refuses_invalid_bindings_rest_arrow_this_and_incomplete_raw() {
    for source in [
        "{ value: }",
        "{ method() {} }",
        "{ get value() {} }",
        "value + other",
        "value; next",
        "value => next",
        "(...rest)",
        "...rest,",
        "...rest, next",
        "[...rest, next]",
        "{ ...rest, next }",
        "...rest = fallback",
        "this: Props",
        "@decorator value",
        "props?: Props, required: Props",
        "value /* unfinished",
    ] {
        let allocator = Allocator::default();
        let errors = Parser::new(&allocator, source, SourceType::ts())
            .parse_slot_parameters()
            .expect_err(source);
        assert!(!errors.is_empty(), "{source}");
        assert!(errors.iter().all(|error| !error.to_string().is_empty()), "{source}");
    }
}
