use oxc_span::GetSpan;
use vize_l0::{Allocator, Span};

use super::super::{NativeForHead, parse_vue_for_head_once, refused};
use super::{EmbedHole, ForHeadHole, ForHeadPart, Lang, Shape, embed, parse};
use crate::embed::syntax::parse_once;

#[test]
fn collection_trailing_line_comment_bytes_and_first_separator_refusals_stay_exact() {
    let allocator = Allocator::default();
    let input = "x in xs // tail  ";
    let head = parse(&allocator, input, Lang::Js);
    assert_eq!(head.hole(), None);
    assert_eq!(
        head.dense().unwrap().collection_source().text(),
        "xs // tail  "
    );
    let (part, comment) = head.comments().next().unwrap();
    assert_eq!(part, ForHeadPart::Collection);
    assert_eq!(comment.text().unwrap(), "// tail  ");
    let span = comment.authored_span().unwrap();
    assert_eq!(
        input.get(span.start as usize..span.end as usize).unwrap(),
        "// tail  "
    );
    // The selected grammar is the first textual separator. A keyword inside
    // alias literal/comment spelling is not silently reinterpreted as a later
    // separator in search of a successful JavaScript parse.
    for input in ["x='a in b' in xs", "(x /* in */) in xs"] {
        let head = parse(&allocator, input, Lang::Js);
        assert!(head.hole().is_some() && head.dense().is_none());
        assert_eq!(head.source().text(), input);
    }
}

#[test]
fn whole_head_admission_stops_before_slicing_or_parsing_either_part() {
    let allocator = Allocator::default();
    let text = vize_l0::cstr!("a in {}", "new ".repeat(32));
    let before = allocator.allocated_bytes();
    let head = parse(&allocator, &text, Lang::Js);
    assert_eq!(
        head.hole(),
        Some(ForHeadHole::WholeHeadAdmission(EmbedHole::TokenBudget))
    );
    assert_eq!(head.source().text(), text);
    assert!(head.aliases().is_none() && head.collection().is_none());
    assert!(head.comments().next().is_none() && head.diagnostics().next().is_none());
    assert_eq!(allocator.allocated_bytes(), before);

    // Whole text fits the bound but the true Params wrapper does not. Both
    // part owners remain visible; wrapper overhead is not silently ignored.
    let text = vize_l0::cstr!("({}b) in xs", "a,".repeat(13));
    let head = parse(&allocator, &text, Lang::Js);
    assert_eq!(
        head.hole(),
        Some(ForHeadHole::AliasesUnavailable(EmbedHole::TokenBudget))
    );
    assert!(head.aliases().is_some() && head.collection().is_some());
    assert!(head.collection().unwrap().unwrap().expression().is_some());
}

#[test]
fn strict_split_failures_and_wrong_shape_keep_whole_source_without_part_owners() {
    let allocator = Allocator::default();
    for (input, hole) in [
        ("items", ForHeadHole::MissingSeparator),
        ("in items", ForHeadHole::MissingSeparator),
        ("kind of", ForHeadHole::MissingSeparator),
        ("item in   ", ForHeadHole::MissingCollection),
        ("(item in xs", ForHeadHole::UnpairedAliasParentheses),
        ("item) in xs", ForHeadHole::UnpairedAliasParentheses),
    ] {
        let head = parse(&allocator, input, Lang::Js);
        assert_eq!(head.hole(), Some(hole), "{input}");
        assert_eq!(head.source().text(), input);
        assert!(head.aliases().is_none() && head.collection().is_none());
        assert!(head.dense().is_none());
    }
    let mut input = embed("x in xs", Lang::Js);
    input.grammar.shape = Shape::Expr;
    let head = parse_vue_for_head_once(&allocator, input);
    assert_eq!(head.hole(), Some(ForHeadHole::WrongShape));
    assert_eq!(head.source().text(), "x in xs");
    assert!(head.aliases().is_none() && head.collection().is_none());
}

#[test]
fn generated_wrapper_escapes_and_nested_module_context_remain_local_holes() {
    let allocator = Allocator::default();
    for input in [
        "(x)=>{};(y) in xs",
        "x=>{} in xs",
        "x=()=>{import 'x'} in xs",
    ] {
        let head = parse(&allocator, input, Lang::Js);
        assert!(head.dense().is_none(), "{input}");
        assert!(matches!(
            head.hole(),
            Some(ForHeadHole::AliasesUnavailable(_))
        ));
        assert_eq!(head.source().text(), input);
        assert!(head.aliases().is_some() && head.collection().is_some());
        assert!(head.collection().unwrap().unwrap().expression().is_some());
    }
    let head = parse(&allocator, "x=await y in xs", Lang::Js);
    assert_eq!(
        head.hole(),
        Some(ForHeadHole::AliasesUnavailable(
            EmbedHole::InvalidParameterContext
        ))
    );
    assert!(head.dense().is_none());
}

#[test]
fn defensive_rejection_keeps_original_ast_and_all_observation_records() {
    let allocator = Allocator::default();
    let mut wrong = embed("/*a*/x", Lang::Js);
    wrong.grammar.shape = Shape::Expr;
    let syntax = parse_once(&allocator, wrong);
    let root = syntax.expression().unwrap() as *const _;
    let comment = syntax.comments().next().unwrap().text().unwrap().as_ptr();
    let original = syntax.into_slot_params(&allocator).unwrap_err();
    let head = NativeForHead {
        grammar: embed("x in xs", Lang::Js).grammar,
        source: embed("x in xs", Lang::Js).source,
        separator: None,
        aliases: Some(Err(original)),
        collection: None,
        hole: Some(ForHeadHole::RejectedPart(ForHeadPart::Aliases)),
    };
    assert_eq!(refused(&head), head.hole());
    assert_eq!(
        head.aliases().unwrap().unwrap_err().expression().unwrap() as *const _,
        root
    );
    assert_eq!(
        head.comments().next().unwrap().1.text().unwrap().as_ptr(),
        comment
    );
    assert_eq!(
        head.aliases().unwrap().unwrap_err().authored_span(
            head.aliases()
                .unwrap()
                .unwrap_err()
                .expression()
                .unwrap()
                .span()
        ),
        Ok(Span::new(5, 6))
    );
}
