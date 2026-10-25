//! Private probe-state tests: no syntax fixture can assert that impossible
//! speculation never enters the checkpoint/recursive parsing path.

use oxc_allocator::Allocator;
use oxc_span::SourceType;

use crate::{ParseOptions, ParserImpl, UniquePromise, config::NoTokensParserConfig, lexer::Kind};

fn parser<'a>(allocator: &'a Allocator, source: &'a str) -> ParserImpl<'a, NoTokensParserConfig> {
    let mut parser = ParserImpl::new(
        allocator,
        source,
        SourceType::ts(),
        ParseOptions::default(),
        NoTokensParserConfig,
        UniquePromise::new_for_tests_and_benchmarks(),
    );
    parser.bump_any();
    parser
}

#[test]
fn no_later_close_never_enters_recursive_speculation_or_changes_parser_state() {
    for source in ["<T", "<<T()", "<[A, [B, C]", "><[[A, B]"] {
        let allocator = Allocator::default();
        let mut parser = parser(&allocator, source);
        if parser.cur_kind() == Kind::RAngle {
            parser.bump_any();
        }
        let token = parser.cur_token();
        assert!(parser.parse_type_arguments_in_expression().is_none());
        assert_eq!(parser.cur_token().kind(), token.kind());
        assert_eq!(parser.cur_token().span(), token.span());
        assert!(parser.errors.is_empty());
        assert!(parser.lexer.errors.is_empty());
        assert!(parser.fatal_error.is_none());
        // A rejected full probe would populate this table after rewinding.
        assert!(parser.state.failed_speculations.is_empty(), "{source}");
    }
}

#[test]
fn immutable_close_cache_survives_rewind_and_rejects_only_the_trailing_opener() {
    let allocator = Allocator::default();
    let source = "<T>() < U";
    let mut parser = parser(&allocator, source);
    let checkpoint = parser.checkpoint();
    let first = parser.parse_type_arguments_in_expression().unwrap();
    let first = format!("{first:?}");
    assert_eq!(parser.state.last_type_argument_close, source.rfind('>'));
    parser.rewind(checkpoint);
    assert_eq!(format!("{:?}", parser.parse_type_arguments_in_expression().unwrap()), first);
    while parser.cur_kind() != Kind::LAngle && parser.cur_kind() != Kind::Eof {
        parser.bump_any();
    }
    assert_eq!(parser.cur_kind(), Kind::LAngle);
    let token = parser.cur_token();
    assert!(parser.parse_type_arguments_in_expression().is_none());
    assert_eq!(parser.cur_token().span(), token.span());
    assert!(parser.state.failed_speculations.is_empty());
}

#[test]
fn close_cache_is_lazy_and_scoped_to_each_parser_source() {
    let allocator = Allocator::default();
    let mut missing = parser(&allocator, "<T");
    let mut closed = parser(&allocator, "<T>(x)");
    assert!(missing.state.last_type_argument_close.is_none());
    assert!(closed.state.last_type_argument_close.is_none());
    assert!(missing.parse_type_arguments_in_expression().is_none());
    assert_eq!(missing.state.last_type_argument_close, Some(0));
    assert!(closed.state.last_type_argument_close.is_none());
    assert!(closed.parse_type_arguments_in_expression().is_some());
    assert_eq!(closed.state.last_type_argument_close, Some(2));
}
