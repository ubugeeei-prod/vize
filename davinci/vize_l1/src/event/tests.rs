//! Exact event and diagnostic laws for the direct native recorder.

use super::{Event, EventKind, Recorder};
use crate::markup::lex::compat::Callbacks;
use crate::markup::lex::compat::Tokenizer;
use crate::markup::{Component, LexOptions, Lexer};
use crate::parse::SurfaceError;
use vize_l0::{Allocator, Vec, cstr};

fn record<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    native: bool,
    in_tag_comments: bool,
) -> (Vec<'a, Event>, Vec<'a, SurfaceError>) {
    let mut events = Vec::new_in(&allocator);
    let mut errors = Vec::new_in(&allocator);
    let recorder = Recorder {
        events: &mut events,
        errors: &mut errors,
    };
    if native {
        Lexer::<Component, _>::new(
            source,
            recorder,
            LexOptions {
                in_tag_comments,
                ..LexOptions::default()
            },
        )
        .run();
    } else {
        let mut tokenizer = Tokenizer::new(source, recorder);
        tokenizer.set_in_tag_comments(in_tag_comments);
        tokenizer.tokenize();
    }
    (events, errors)
}

fn assert_trace(source: &str, in_tag_comments: bool) {
    let allocator = Allocator::new();
    let native = record(&allocator, source, true, in_tag_comments);
    let compat = record(&allocator, source, false, in_tag_comments);
    assert_eq!(native.0.as_slice(), compat.0.as_slice(), "{source}");
    assert_eq!(
        cstr!("{:?}", native.1),
        cstr!("{:?}", compat.1),
        "diagnostic code, order and byte offset: {source}"
    );
}

#[test]
fn direct_native_recorder_matches_fixture_events_and_utf8_cuts() {
    for fixture in davinci_test_support::surface_fixture::WELL_FORMED
        .iter()
        .chain(davinci_test_support::surface_fixture::MALFORMED)
    {
        assert_trace(fixture.source, false);
        for (index, _) in fixture.source.char_indices() {
            for source in [
                fixture.source.get(..index).expect("UTF-8 boundary"),
                fixture.source.get(index..).expect("UTF-8 boundary"),
            ] {
                assert_trace(source, false);
            }
        }
    }
    assert_trace("<Comp // comment\n :title='日'/>", true);
    assert_trace("<div title=\"日", false);
}

#[test]
fn full_entity_values_record_one_authored_span_each() {
    let source = "<p title='&fjlig;&acE;&nGt;'>&fjlig;&acE;&nGt;</p>";
    let allocator = Allocator::new();
    let (events, _) = record(&allocator, source, true, false);
    for entity in ["&fjlig;", "&acE;", "&nGt;"] {
        for (start, _) in source.match_indices(entity) {
            assert_eq!(
                events
                    .iter()
                    .filter(|event| {
                        matches!(event.kind, EventKind::Text | EventKind::AttrData)
                            && event.start as usize == start
                            && event.end as usize == start + entity.len()
                    })
                    .count(),
                1,
                "one event per authored reference: {entity} at {start}"
            );
        }
    }
    assert_trace(source, false);
}

#[test]
fn opening_mode_aux_preserves_quote_interpretation_and_event_layout() {
    use crate::markup::token::QuoteType;
    let allocator = Allocator::new();
    let (mut events, mut errors) = record(&allocator, "", true, false);
    let mut recorder = Recorder {
        events: &mut events,
        errors: &mut errors,
    };
    for quote in [
        QuoteType::NoValue,
        QuoteType::Unquoted,
        QuoteType::Single,
        QuoteType::Double,
    ] {
        recorder.attr_end(quote, 7);
        assert_eq!(recorder.events.last().unwrap().quote(), quote);
    }
    for (kind, verbatim) in [
        (EventKind::OpenTagEnd, true),
        (EventKind::SelfClosingTag, true),
        (EventKind::OpenTagEnd, false),
    ] {
        recorder.opening_end(kind, 11, verbatim);
        let event = recorder.events.last().unwrap();
        assert_eq!(event.kind, kind);
        assert_eq!((event.start, event.end), (11, 11));
        assert_eq!(event.is_verbatim_opening(), verbatim);
    }
    assert_eq!(core::mem::size_of::<Event>(), 12);
    assert_eq!(events.len(), 7);
    assert!(errors.is_empty());
}

#[test]
fn raw_interpolation_width_is_disjoint_from_quote_and_opening_aux() {
    use crate::markup::token::QuoteType;
    let allocator = Allocator::new();
    let (mut events, mut errors) = record(&allocator, "", true, false);
    let mut recorder = Recorder {
        events: &mut events,
        errors: &mut errors,
    };
    recorder.raw_interpolation(3, 8);
    let raw = recorder.events.last().unwrap();
    assert_eq!(
        (raw.kind, raw.start, raw.end, raw.interpolation_width()),
        (EventKind::Interpolation, 3, 8, 3)
    );
    recorder.attr_end(QuoteType::Double, 9);
    let quote = recorder.events.last().unwrap();
    assert_eq!(quote.quote(), QuoteType::Double);
    recorder.on_interpolation(2, 10);
    assert_eq!(recorder.events.last().unwrap().interpolation_width(), 2);
    assert_eq!(core::mem::size_of::<Event>(), 12);
    assert_eq!(events.len(), 3);
    assert!(errors.is_empty());
}

#[test]
fn selected_lint_tag_aux_never_changes_mode_quotes_width_or_event_layout() {
    use crate::surface::LintTagFact;
    let allocator = Allocator::new();
    let (mut events, mut errors) = record(&allocator, "", true, false);
    let mut recorder = Recorder {
        events: &mut events,
        errors: &mut errors,
    };
    for kind in [EventKind::OpenTagEnd, EventKind::SelfClosingTag] {
        for fact in [
            LintTagFact::Element,
            LintTagFact::Component,
            LintTagFact::Slot,
            LintTagFact::Template,
            LintTagFact::AmbiguousVerbatim,
            LintTagFact::InheritedTemplate,
        ] {
            for verbatim in [false, true] {
                for literal in [false, true] {
                    recorder.opening_end_with_lint(kind, 19, verbatim, Some(fact), literal);
                    let event = recorder.events.last().unwrap();
                    assert_eq!(event.kind, kind);
                    assert_eq!((event.start, event.end), (19, 19));
                    assert_eq!(event.is_verbatim_opening(), verbatim);
                    assert_eq!(event.lint_tag().unwrap() as u8, fact as u8);
                    assert_eq!(event.lint_header_is_literal(), literal);
                }
            }
        }
        recorder.opening_end(kind, 23, false);
        let ordinary = recorder.events.last().unwrap();
        assert_eq!(ordinary.aux, 0);
        assert!(ordinary.lint_tag().is_none());
        assert!(!ordinary.lint_header_is_literal());
    }
    assert_eq!(core::mem::size_of::<Event>(), 12);
    assert_eq!(events.len(), 50);
    assert!(errors.is_empty());
}
