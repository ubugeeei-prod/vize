//! Source-authored private-consumer changes and unchanged facade event laws.

mod recorder;

use super::OriginalFirstScalarSink;
use recorder::{Event, FullRecorder, Recorder};
use vize_l1::markup::{
    Component, Delimiters, Document, LexErrorCode, LexMode, LexOptions, Lexer, Profile, QuoteType,
    Sink, entity::DecodedEntity, lex::compat::adapter::CompatSink,
};

fn public_events<P: Profile>(source: &str, options: LexOptions<'_>, verbatim: bool) -> Vec<Event> {
    let mut lexer = Lexer::<P, _>::new(
        source,
        CompatSink::new(Recorder {
            verbatim,
            ..Recorder::default()
        }),
        options,
    );
    lexer.run();
    lexer.into_sink().into_callbacks().events
}

fn consumer_events<P: Profile>(
    source: &str,
    options: LexOptions<'_>,
    verbatim: bool,
) -> Vec<Event> {
    let mut lexer = Lexer::<P, _>::new(
        source,
        CompatSink::new(FullRecorder {
            verbatim,
            ..FullRecorder::default()
        }),
        options,
    );
    lexer.run();
    lexer.into_sink().into_callbacks().events
}

fn assert_only_authored_attribute_expansions<P: Profile>(
    source: &str,
    options: LexOptions<'_>,
    verbatim: bool,
    required_expansions: usize,
) {
    let before = public_events::<P>(source, options, verbatim);
    let after = consumer_events::<P>(source, options, verbatim);
    let mut expected = Vec::new();
    let mut expansions = 0;
    for event in &before {
        expected.push(event.clone());
        let Event::Entity("attribute", first, start, end) = event else {
            continue;
        };
        // Independently authored HTML expansions; do not ask the new sink or
        // decoder for the expected characters. Every other event stays whole.
        let expansion = match source.get(*start..*end) {
            Some("&fjlig;") => Some(('f', 'j')),
            Some("&NotEqualTilde;") => Some(('≂', '\u{338}')),
            Some("&acE;") => Some(('∾', '\u{333}')),
            Some("&ThickSpace;") => Some(('\u{205f}', '\u{200a}')),
            _ => None,
        };
        if let Some((required_first, second)) = expansion {
            assert_eq!(*first, required_first, "original first scalar: {source}");
            expected.push(Event::Entity("attribute", second, *start, *end));
            expansions += 1;
        }
    }
    assert_eq!(expansions, required_expansions, "{source}");
    assert_eq!(after, expected, "whole callback vector: {source}");
}

fn assert_profiles(source: &str, options: LexOptions<'_>, expansions: usize) {
    for verbatim in [false, true] {
        assert_only_authored_attribute_expansions::<Component>(
            source, options, verbatim, expansions,
        );
        assert_only_authored_attribute_expansions::<Document>(
            source, options, verbatim, expansions,
        );
    }
}

#[test]
fn complete_named_attribute_values_preserve_every_other_callback() {
    assert_profiles(
        r#"<!DOCTYPE html><?pi?><p a='&fjlig;' b="&NotEqualTilde;" c=&acE; d='&ThickSpace;'>x&fjlig;{{ value }}</p><!--tail-->"#,
        LexOptions::default(),
        4,
    );
    assert_profiles(
        "<X\n // comment\n :title='&acE;'/>[[value]]{{{ raw }}}&fjlig;",
        LexOptions {
            delimiters: Delimiters {
                open: b"[[",
                close: b"]]",
            },
            in_tag_comments: true,
            raw_interpolation: true,
        },
        1,
    );
    assert_profiles(
        "<p title='&fjlig;'>&fjlig;{{{raw}}}{{escaped}}</p>",
        LexOptions {
            raw_interpolation: true,
            ..LexOptions::default()
        },
        1,
    );
    // Complete entities at EOF keep the original recovery/error/end vector.
    assert_profiles("<p title='&fjlig;", LexOptions::default(), 1);
}

#[test]
fn scalar_unknown_ambiguous_and_decode_once_entities_keep_whole_vectors() {
    for source in [
        "<p a='&amp;&#32;&#x1f642;'>&amp;&#x1f642;</p>",
        "<p a='&unknown;&fjlig;&amp;amp;'>text</p>",
        "<p a='&fjligx;&notit=;&ampx;&fjlig'>&unknown;</p>",
        "<p a='&#xzz;&#0;&#x110000;&'>text</p>",
    ] {
        let expansions = usize::from(source.contains("&fjlig;"));
        assert_profiles(source, LexOptions::default(), expansions);
    }
    for fixture in davinci_test_support::surface_fixture::WELL_FORMED
        .iter()
        .chain(davinci_test_support::surface_fixture::MALFORMED)
    {
        for source in core::iter::once(fixture.source).chain(
            fixture.source.char_indices().flat_map(|(at, _)| {
                [
                    fixture.source.get(..at).unwrap(),
                    fixture.source.get(at..).unwrap(),
                ]
            }),
        ) {
            assert_profiles(source, LexOptions::default(), 0);
        }
    }
}

fn all_original_events(sink: &mut impl Sink) {
    sink.on_text(1, 2);
    sink.on_text_entity('雪', 3, 4);
    sink.on_text_entity_value(DecodedEntity::Named("fj"), 5, 6);
    sink.on_interpolation(7, 8);
    sink.on_raw_interpolation(9, 10);
    sink.on_open_tag_name(11, 12);
    sink.on_open_tag_end(13);
    sink.on_self_closing_tag(14);
    sink.on_close_tag(15, 16);
    sink.on_attrib_data(17, 18);
    sink.on_attrib_entity('🙂', 19, 20);
    sink.on_attrib_entity_value(DecodedEntity::Numeric('雪'), 21, 22);
    for (i, quote) in [
        QuoteType::NoValue,
        QuoteType::Unquoted,
        QuoteType::Single,
        QuoteType::Double,
    ]
    .into_iter()
    .enumerate()
    {
        sink.on_attrib_end(quote, 23 + i);
    }
    sink.on_attrib_name(27, 28);
    sink.on_attrib_name_end(29);
    sink.on_dir_name(30, 31);
    sink.on_dir_arg(32, 33);
    sink.on_dir_modifier(34, 35);
    sink.on_comment(36, 37);
    sink.on_in_tag_comment(38, 39);
    sink.on_cdata(40, 41);
    sink.on_processing_instruction(42, 43);
    sink.on_declaration(44, 45, true);
    sink.on_declaration_recovery(46, 47, false);
    sink.on_error(LexErrorCode::EofInTag, 48);
    sink.on_end();
}

#[test]
fn all_other_sink_methods_and_live_mode_delegate_to_the_same_compat_sink() {
    for verbatim in [false, true] {
        let mut before = CompatSink::new(Recorder {
            verbatim,
            ..Recorder::default()
        });
        let mut after = CompatSink::new(FullRecorder {
            verbatim,
            ..FullRecorder::default()
        });
        all_original_events(&mut before);
        all_original_events(&mut after);
        assert_eq!(after.mode(), before.mode());
        assert_eq!(
            after.mode(),
            if verbatim {
                LexMode::Verbatim
            } else {
                LexMode::Normal
            }
        );
        assert_eq!(
            after.into_callbacks().events,
            before.into_callbacks().events
        );
    }
}

#[test]
fn first_scalar_reference_adds_no_fields_drop_owner_or_retained_state() {
    type Callback = super::super::callbacks::ParserCallbacks<'static, 'static>;
    assert_eq!(
        size_of::<OriginalFirstScalarSink<Callback>>(),
        size_of::<CompatSink<Callback>>()
    );
    assert_eq!(
        align_of::<OriginalFirstScalarSink<Callback>>(),
        align_of::<CompatSink<Callback>>()
    );
    assert_eq!(
        core::mem::needs_drop::<OriginalFirstScalarSink<Callback>>(),
        core::mem::needs_drop::<CompatSink<Callback>>()
    );
    assert!(!core::mem::needs_drop::<OriginalFirstScalarSink<Callback>>());
    #[cfg(target_pointer_width = "64")]
    {
        assert_eq!(size_of::<Option<vize_relief::RetainedJsAst<'static>>>(), 24);
        assert_eq!(size_of::<vize_relief::SimpleExpressionNode<'static>>(), 88);
    }
}

#[test]
fn provided_default_is_first_scalar_and_full_values_require_explicit_opt_in() {
    use crate::tokenizer::Callbacks;

    let mut original = Recorder::default();
    let mut complete = FullRecorder::default();
    // Both assignments compile only while the provided method stays object safe.
    let default_callback: &mut dyn Callbacks = &mut original;
    let full_callback: &mut dyn Callbacks = &mut complete;
    for (value, start, end) in [
        (DecodedEntity::Named("fj"), 3, 9),
        (DecodedEntity::Numeric('雪'), 10, 18),
        (DecodedEntity::Named(""), 19, 20),
    ] {
        default_callback.on_attrib_entity_value(value, start, end);
        full_callback.on_attrib_entity_value(value, start, end);
    }
    assert_eq!(
        original.events,
        [
            Event::Entity("attribute", 'f', 3, 9),
            Event::Entity("attribute", '雪', 10, 18),
        ]
    );
    assert_eq!(
        complete.events,
        [
            Event::Entity("attribute", 'f', 3, 9),
            Event::Entity("attribute", 'j', 3, 9),
            Event::Entity("attribute", '雪', 10, 18),
        ]
    );
}

#[test]
fn historical_sink_refuses_inheriting_the_complete_callback_override() {
    let mut historical = OriginalFirstScalarSink::new(FullRecorder::default());
    let mut current = CompatSink::new(FullRecorder::default());
    for (value, start, end) in [
        (DecodedEntity::Named("fj"), 3, 9),
        (DecodedEntity::Numeric('雪'), 10, 18),
    ] {
        historical.on_attrib_entity_value(value, start, end);
        current.on_attrib_entity_value(value, start, end);
    }
    assert_eq!(
        historical.into_callbacks().events,
        [
            Event::Entity("attribute", 'f', 3, 9),
            Event::Entity("attribute", '雪', 10, 18),
        ]
    );
    assert_eq!(
        current.into_callbacks().events,
        [
            Event::Entity("attribute", 'f', 3, 9),
            Event::Entity("attribute", 'j', 3, 9),
            Event::Entity("attribute", '雪', 10, 18),
        ]
    );
}
