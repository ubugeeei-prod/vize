use oxc_ast::ast::{BindingPattern, Expression};
use vize_l0::{Allocator, Span};

use super::super::{Embed, EmbedHole, EmbedSource, Grammar, Lang, Shape, parse_once};
use crate::embed::prepare_attribute_value;

fn embed(text: &str, shape: Shape, lang: Lang) -> Embed<'_> {
    Embed {
        grammar: Grammar { shape, lang },
        source: EmbedSource::authored(text, Span::new(0, text.len() as u32)).unwrap(),
    }
}

#[test]
fn original_parameter_roots_and_descendants_outlive_the_observation_owner() {
    let allocator = Allocator::default();
    let (parameters, rest) = {
        let tree = parse_once(
            &allocator,
            embed("{item}, index = 0, ...rest", Shape::SlotParams, Lang::Js),
        );
        let view = tree.slot_params().unwrap();
        let original_slice = view.parameters().as_ptr();
        let BindingPattern::ObjectPattern(object) = &view.parameters()[0].pattern else {
            panic!("expected actual binding pattern")
        };
        let object = &**object as *const _;
        let initializer = view.parameters()[1].initializer.as_deref().unwrap() as *const _;
        let BindingPattern::BindingIdentifier(identifier) = &view.rest().unwrap().rest.argument
        else {
            panic!("expected actual rest binding")
        };
        let identifier = &**identifier as *const _;
        let retained = tree.into_slot_params().unwrap();
        assert_eq!(retained.hole(), None);
        let parameters = retained.parameters().unwrap();
        let rest = retained.rest().unwrap();
        assert_eq!(parameters.as_ptr(), original_slice);
        let BindingPattern::ObjectPattern(moved) = &parameters[0].pattern else {
            panic!("expected original object binding")
        };
        assert_eq!(&**moved as *const _, object);
        assert_eq!(
            parameters[1].initializer.as_deref().unwrap() as *const _,
            initializer
        );
        let BindingPattern::BindingIdentifier(moved) = &rest.rest.argument else {
            panic!("expected original rest identifier")
        };
        assert_eq!(&**moved as *const _, identifier);
        assert_eq!(
            retained.decoded_span(parameters[0].span),
            Ok(Span::new(0, 6))
        );
        assert_eq!(retained.authored_span(rest.span), Ok(Span::new(19, 26)));
        drop(retained);
        (parameters, rest)
    };
    assert_eq!(parameters.len(), 2);
    assert!(matches!(
        parameters[1].initializer.as_deref(),
        Some(Expression::NumericLiteral(_))
    ));
    let BindingPattern::BindingIdentifier(identifier) = &rest.rest.argument else {
        panic!("rest root remains valid after owner drop")
    };
    assert_eq!(identifier.name.as_str(), "rest");
}

#[test]
fn retained_bindings_keep_ts_annotations_and_existing_decoded_maps() {
    let allocator = Allocator::default();
    let tree = parse_once(
        &allocator,
        embed("value: Item", Shape::SlotParams, Lang::Ts),
    );
    let annotation = tree.slot_params().unwrap().parameters()[0]
        .type_annotation
        .as_deref()
        .unwrap() as *const _;
    let retained = tree.into_slot_params().unwrap();
    assert_eq!(retained.grammar().lang, Lang::Ts);
    assert_eq!(retained.parser_prefix(), 2);
    assert_eq!(
        retained.parameters().unwrap()[0]
            .type_annotation
            .as_deref()
            .unwrap() as *const _,
        annotation
    );
    assert_eq!(
        retained.decoded_span(retained.parameters().unwrap()[0].span),
        Ok(Span::new(0, 11))
    );
    let js = parse_once(
        &allocator,
        embed("value: Item", Shape::SlotParams, Lang::Js),
    )
    .into_slot_params()
    .unwrap();
    assert_eq!(js.hole(), Some(EmbedHole::Syntax));
    assert!(js.parameters().is_none() && js.rest().is_none());

    let source = prepare_attribute_value(&allocator, "zz&fjlig;zz", Span::new(2, 9)).unwrap();
    let map = source.decode_map().unwrap().segments().as_ptr();
    let retained = parse_once(
        &allocator,
        Embed {
            grammar: Grammar {
                shape: Shape::SlotParams,
                lang: Lang::Js,
            },
            source,
        },
    )
    .into_slot_params()
    .unwrap();
    assert_eq!(
        retained.source().decode_map().unwrap().segments().as_ptr(),
        map
    );
    let BindingPattern::BindingIdentifier(identifier) = &retained.parameters().unwrap()[0].pattern
    else {
        panic!("expected original once-decoded binding")
    };
    assert_eq!(identifier.name.as_str(), "fj");
    assert_eq!(retained.authored_span(identifier.span), Ok(Span::new(2, 9)));
}

#[test]
fn holes_keep_comments_and_complete_owned_diagnostics_without_recovery_bindings() {
    let allocator = Allocator::default();
    let tree = parse_once(
        &allocator,
        embed("value = /*keep*/", Shape::SlotParams, Lang::Js),
    );
    let message = tree.diagnostics().next().unwrap().message().as_ptr();
    let comment = tree.comments().next().unwrap().text().unwrap().as_ptr();
    let comment_slice = tree.embedding.as_ref().unwrap().comments().as_ptr();
    let retained = tree.into_slot_params().unwrap();
    assert_eq!(retained.hole(), Some(EmbedHole::Syntax));
    assert!(retained.parameters().is_none() && retained.rest().is_none());
    assert_eq!(
        retained.diagnostics().next().unwrap().message().as_ptr(),
        message
    );
    assert_eq!(
        retained.comments().next().unwrap().text().unwrap().as_ptr(),
        comment
    );
    assert_eq!(
        retained.observation.as_ref().unwrap().comments().as_ptr(),
        comment_slice
    );
    for diagnostic in retained.diagnostics() {
        for label in diagnostic.labels() {
            let span = label.decoded_span().unwrap();
            assert!(
                retained
                    .source()
                    .text()
                    .get(span.start as usize..span.end as usize)
                    .is_some()
            );
        }
    }
    let large = ",".repeat(32);
    let retained = parse_once(&allocator, embed(&large, Shape::SlotParams, Lang::Js))
        .into_slot_params()
        .unwrap();
    assert_eq!(retained.hole(), Some(EmbedHole::TokenBudget));
    assert_eq!(retained.source().text(), large);
    assert!(retained.parameters().is_none() && retained.rest().is_none());
    let retained = parse_once(
        &allocator,
        embed("x=()=>{import 'x'}", Shape::SlotParams, Lang::Js),
    )
    .into_slot_params()
    .unwrap();
    assert_eq!(retained.hole(), Some(EmbedHole::InvalidModuleContext));
    assert!(retained.parameters().is_none() && retained.rest().is_none());
}

#[test]
fn empty_and_rest_only_parameters_expose_no_generated_container() {
    let allocator = Allocator::default();
    let empty = parse_once(&allocator, embed("", Shape::SlotParams, Lang::Js))
        .into_slot_params()
        .unwrap();
    assert_eq!(empty.hole(), None);
    assert!(empty.parameters().unwrap().is_empty() && empty.rest().is_none());
    assert!(empty.decoded_span(oxc_span::Span::new(0, 4)).is_err());
    let rest = parse_once(
        &allocator,
        embed("...rest //tail", Shape::SlotParams, Lang::Js),
    )
    .into_slot_params()
    .unwrap();
    assert!(rest.parameters().unwrap().is_empty());
    assert_eq!(
        rest.decoded_span(rest.rest().unwrap().span),
        Ok(Span::new(0, 7))
    );
    assert_eq!(rest.comments().next().unwrap().text().unwrap(), "//tail");
    assert_eq!(
        rest.comments().next().unwrap().decoded_span(),
        Ok(Span::new(8, 14))
    );
}

#[test]
fn rejected_shapes_keep_the_same_original_expression_and_comments() {
    let allocator = Allocator::default();
    let tree = parse_once(
        &allocator,
        embed("/*keep*/left+right", Shape::Expr, Lang::Js),
    );
    let root = tree.expression().unwrap() as *const _;
    let comment = tree.comments().next().unwrap().text().unwrap().as_ptr();
    let original = tree.into_slot_params().unwrap_err();
    assert_eq!(original.grammar().shape, Shape::Expr);
    assert_eq!(original.expression().unwrap() as *const _, root);
    assert_eq!(
        original.comments().next().unwrap().text().unwrap().as_ptr(),
        comment
    );
}
