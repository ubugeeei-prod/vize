//! Exact event and recovery parity while #6835 converges the profile lexer.
//!
//! Opt in with `--features native-lex-parity`; the compiler still uses the
//! moved tokenizer through Armature's existing public path.

use vize_armature::tokenizer::{Callbacks, QuoteType as CompatQuote, Tokenizer};
use vize_l0::{SmallVec, String, cstr};
use vize_l1::markup::lex::compat::adapter::CompatSink;
use vize_l1::markup::token::{LexErrorCode, QuoteType as NativeQuote, Sink};
use vize_l1::markup::{Component, Document, LexOptions, Lexer, Profile};
use vize_relief::ErrorCode;

#[derive(Debug, Default, PartialEq, Eq)]
struct Trace {
    events: SmallVec<[String; 32]>,
}

struct CompatTrace<'a>(&'a mut Trace);
struct NativeTrace<'a>(&'a mut Trace);

macro_rules! event_methods {
    ($quote:ty) => {
        fn on_text(&mut self, start: usize, end: usize) {
            self.0.events.push(cstr!("text:{start}..{end}"));
        }
        fn on_text_entity(&mut self, ch: char, start: usize, end: usize) {
            self.0
                .events
                .push(cstr!("text-entity:{ch:?}:{start}..{end}"));
        }
        fn on_interpolation(&mut self, start: usize, end: usize) {
            self.0.events.push(cstr!("interpolation:{start}..{end}"));
        }
        fn on_raw_interpolation(&mut self, start: usize, end: usize) {
            self.0
                .events
                .push(cstr!("raw-interpolation:{start}..{end}"));
        }
        fn on_open_tag_name(&mut self, start: usize, end: usize) {
            self.0.events.push(cstr!("open-name:{start}..{end}"));
        }
        fn on_open_tag_end(&mut self, end: usize) {
            self.0.events.push(cstr!("open-end:{end}"));
        }
        fn on_self_closing_tag(&mut self, end: usize) {
            self.0.events.push(cstr!("self-close:{end}"));
        }
        fn on_close_tag(&mut self, start: usize, end: usize) {
            self.0.events.push(cstr!("close-name:{start}..{end}"));
        }
        fn on_attrib_data(&mut self, start: usize, end: usize) {
            self.0.events.push(cstr!("attr-data:{start}..{end}"));
        }
        fn on_attrib_entity(&mut self, ch: char, start: usize, end: usize) {
            self.0
                .events
                .push(cstr!("attr-entity:{ch:?}:{start}..{end}"));
        }
        fn on_attrib_end(&mut self, quote: $quote, end: usize) {
            self.0.events.push(cstr!("attr-end:{quote:?}:{end}"));
        }
        fn on_attrib_name(&mut self, start: usize, end: usize) {
            self.0.events.push(cstr!("attr-name:{start}..{end}"));
        }
        fn on_attrib_name_end(&mut self, end: usize) {
            self.0.events.push(cstr!("attr-name-end:{end}"));
        }
        fn on_dir_name(&mut self, start: usize, end: usize) {
            self.0.events.push(cstr!("dir-name:{start}..{end}"));
        }
        fn on_dir_arg(&mut self, start: usize, end: usize) {
            self.0.events.push(cstr!("dir-arg:{start}..{end}"));
        }
        fn on_dir_modifier(&mut self, start: usize, end: usize) {
            self.0.events.push(cstr!("dir-modifier:{start}..{end}"));
        }
        fn on_comment(&mut self, start: usize, end: usize) {
            self.0.events.push(cstr!("comment:{start}..{end}"));
        }
        fn on_cdata(&mut self, start: usize, end: usize) {
            self.0.events.push(cstr!("cdata:{start}..{end}"));
        }
        fn on_processing_instruction(&mut self, start: usize, end: usize) {
            self.0.events.push(cstr!("pi:{start}..{end}"));
        }
        fn on_end(&mut self) {
            self.0.events.push(cstr!("end"));
        }
    };
}

impl Callbacks for CompatTrace<'_> {
    event_methods!(CompatQuote);

    fn on_error(&mut self, code: ErrorCode, index: usize) {
        self.0.events.push(cstr!("error:{code:?}:{index}"));
    }
}

impl Sink for NativeTrace<'_> {
    event_methods!(NativeQuote);

    fn on_error(&mut self, code: LexErrorCode, index: usize) {
        self.0.events.push(cstr!("error:{code:?}:{index}"));
    }
}

fn traces_with<P: Profile>(source: &str, raw_interpolation: bool) -> (Trace, Trace) {
    let mut compat = Trace::default();
    {
        let mut lexer = Tokenizer::new(source, CompatTrace(&mut compat));
        lexer.set_tolerate_declarations(P::TOLERATE_DECLARATIONS);
        lexer.set_triple_mustache(raw_interpolation);
        lexer.tokenize();
    }
    let mut native = Trace::default();
    {
        let mut lexer = Lexer::<P, _>::new(
            source,
            NativeTrace(&mut native),
            LexOptions {
                raw_interpolation,
                ..LexOptions::default()
            },
        );
        lexer.run();
    }
    (compat, native)
}

fn traces(source: &str) -> (Trace, Trace) {
    traces_with::<Component>(source, false)
}

fn adapted_trace_with<P: Profile>(source: &str, raw_interpolation: bool) -> Trace {
    let mut trace = Trace::default();
    let mut lexer = Lexer::<P, _>::new(
        source,
        CompatSink::new(CompatTrace(&mut trace)),
        LexOptions {
            raw_interpolation,
            ..LexOptions::default()
        },
    );
    lexer.run();
    trace
}

#[test]
fn component_lexer_events_and_recovery_match_moved_tokenizer() {
    for source in [
        "<div a=\"b\">hello {{ x }}</div>",
        "<x />",
        "<x / >",
        "<div / a>",
        "<div a=\"b\"c>",
        "<a :[name].stop=\"x\">",
        "<!-- a --><p>one &amp; two</p>",
        "<!DOCTYPE html><p>x</p>",
        "<div a='unfinished",
    ] {
        let (compat, native) = traces(source);
        assert_eq!(native, compat, "{source}");
    }

    for fixture in davinci_test_support::surface_fixture::WELL_FORMED
        .iter()
        .chain(davinci_test_support::surface_fixture::MALFORMED)
    {
        let (compat, native) = traces(fixture.source);
        assert_eq!(native, compat, "{}: {}", fixture.name, fixture.source);
    }
}

#[test]
fn truncated_component_corpus_keeps_exact_recovery_events() {
    for fixture in davinci_test_support::surface_fixture::WELL_FORMED
        .iter()
        .chain(davinci_test_support::surface_fixture::MALFORMED)
    {
        for (index, _) in fixture.source.char_indices() {
            let prefix = fixture.source.get(..index).expect("UTF-8 boundary");
            let suffix = fixture.source.get(index..).expect("UTF-8 boundary");
            for source in [prefix, suffix] {
                let (compat, native) = traces(source);
                assert_eq!(native, compat, "{} at {index}: {source}", fixture.name);
            }
        }
    }
}

#[test]
fn document_profile_and_raw_interpolation_keep_exact_events() {
    for source in [
        "<!DOCTYPE html><main id='x'>{{ value }}</main>",
        "<!bogus><table><tr><td>x</td></tr></table>",
        "<title>a &amp; {{ title }}</title>",
    ] {
        let (compat, native) = traces_with::<Document>(source, false);
        assert_eq!(native, compat, "document: {source}");
    }
    for source in [
        "{{{ raw }}}",
        "before {{{ raw }}} after {{ escaped }}",
        "<p>{{{ raw }}}</p>",
    ] {
        let (compat, native) = traces_with::<Component>(source, true);
        assert_eq!(native, compat, "raw interpolation: {source}");
    }
}

#[test]
fn multi_scalar_entity_marks_the_legacy_adapter_boundary() {
    let source = "<p title='&fjlig;'>&fjlig;</p>";
    let (compat, native) = traces(source);
    let attr_start = source.find("&fjlig;").expect("attribute entity");
    let text_start = source.rfind("&fjlig;").expect("text entity");
    let attr_end = attr_start + "&fjlig;".len();
    let text_end = text_start + "&fjlig;".len();
    let entities = |trace: &Trace| {
        trace
            .events
            .iter()
            .filter(|event| event.starts_with("attr-entity:") || event.starts_with("text-entity:"))
            .cloned()
            .collect::<SmallVec<[String; 4]>>()
    };

    assert_eq!(
        entities(&compat).as_slice(),
        [
            cstr!("attr-entity:'f':{attr_start}..{attr_end}"),
            cstr!("text-entity:'f':{text_start}..{text_end}"),
        ]
    );
    assert_eq!(
        entities(&native).as_slice(),
        [
            cstr!("attr-entity:'f':{attr_start}..{attr_end}"),
            cstr!("attr-entity:'j':{attr_start}..{attr_end}"),
            cstr!("text-entity:'f':{text_start}..{text_end}"),
            cstr!("text-entity:'j':{text_start}..{text_end}"),
        ]
    );
    let adapted = adapted_trace_with::<Component>(source, false);
    assert_eq!(adapted, compat, "adapter must retain the old first scalar");
}

#[test]
fn native_adapter_keeps_moved_tokenizer_events_and_recovery() {
    for fixture in davinci_test_support::surface_fixture::WELL_FORMED
        .iter()
        .chain(davinci_test_support::surface_fixture::MALFORMED)
    {
        let (compat, _) = traces(fixture.source);
        let adapted = adapted_trace_with::<Component>(fixture.source, false);
        assert_eq!(adapted, compat, "{}: {}", fixture.name, fixture.source);
        for (index, _) in fixture.source.char_indices() {
            let prefix = fixture.source.get(..index).expect("UTF-8 boundary");
            let suffix = fixture.source.get(index..).expect("UTF-8 boundary");
            for source in [prefix, suffix] {
                let (compat, _) = traces(source);
                let adapted = adapted_trace_with::<Component>(source, false);
                assert_eq!(adapted, compat, "{} at {index}: {source}", fixture.name);
            }
        }
    }

    for source in [
        "<!DOCTYPE html><main id='x'>{{ value }}</main>",
        "<!bogus><table><tr><td>x</td></tr></table>",
        "<title>a &amp; {{ title }}</title>",
    ] {
        let (compat, _) = traces_with::<Document>(source, false);
        assert_eq!(adapted_trace_with::<Document>(source, false), compat);
    }
    for source in ["{{{ raw }}}", "before {{{ raw }}} after {{ escaped }}"] {
        let (compat, _) = traces_with::<Component>(source, true);
        assert_eq!(adapted_trace_with::<Component>(source, true), compat);
    }
}

mod markup_lex_parity_support {
    //! Complete authored event packets; parity alone would compare the same lexer.

    use super::{Component, Document, Profile, traces_with};

    fn assert_events<P: Profile>(source: &str, expected: &[&str]) {
        let (published, native) = traces_with::<P>(source, false);
        for (entry, trace) in [("published", published), ("native", native)] {
            let actual: Vec<_> = trace.events.iter().map(|event| event.as_str()).collect();
            assert_eq!(actual, expected, "{entry}: {source}");
        }
    }

    #[test]
    fn eof_missing_value_keeps_the_complete_published_default_stream() {
        let expected = [
            "open-name:1..4",
            "dir-name:5..6",
            "dir-arg:6..8",
            "attr-name-end:8",
            "error:EofInTag:9",
            "error:MissingAttributeValue:9",
            "attr-end:NoValue:9",
            "open-end:8",
            "end",
        ];
        assert_events::<Component>("<div :id=", &expected);
        assert_events::<Document>("<div :id=", &expected);
    }

    #[test]
    fn document_rawtext_names_fold_both_leading_bytes() {
        for name in ["script", "Script", "sCript", "SCRIPT"] {
            assert_events::<Document>(
                &format!("<{name}><b/>{{{{x}}}}&amp;</sCrIpT><p/>"),
                &[
                    "open-name:1..7",
                    "open-end:7",
                    "text:8..22",
                    "close-name:24..30",
                    "open-name:32..33",
                    "self-close:34",
                    "end",
                ],
            );
        }
        for name in ["style", "Style", "sTyle", "STYLE"] {
            assert_events::<Document>(
                &format!("<{name}><b/>{{{{x}}}}&amp;</sTyLe><p/>"),
                &[
                    "open-name:1..6",
                    "open-end:6",
                    "text:7..21",
                    "close-name:23..28",
                    "open-name:30..31",
                    "self-close:32",
                    "end",
                ],
            );
        }
    }

    #[test]
    fn document_rcdata_names_preserve_entities_interpolations_and_source_spans() {
        for name in ["title", "Title", "tItle", "TITLE"] {
            assert_events::<Document>(
                &format!("<{name}><b/>{{{{x}}}}&amp;</tItLe><p/>"),
                &[
                    "open-name:1..6",
                    "open-end:6",
                    "text:7..11",
                    "interpolation:13..14",
                    "text-entity:'&':16..21",
                    "close-name:23..28",
                    "open-name:30..31",
                    "self-close:32",
                    "end",
                ],
            );
        }
        for name in ["textarea", "Textarea", "tExtarea", "TEXTAREA"] {
            assert_events::<Document>(
                &format!("<{name}><b/>{{{{x}}}}&amp;</tExTaReA><p/>"),
                &[
                    "open-name:1..9",
                    "open-end:9",
                    "text:10..14",
                    "interpolation:16..17",
                    "text-entity:'&':19..24",
                    "close-name:26..34",
                    "open-name:36..37",
                    "self-close:38",
                    "end",
                ],
            );
        }
    }

    #[test]
    fn component_uppercase_names_remain_ordinary_markup() {
        for name in ["Script", "sCript", "SCRIPT"] {
            assert_events::<Component>(
                &format!("<{name}><b/>{{{{x}}}}&amp;</{name}><p/>"),
                &[
                    "open-name:1..7",
                    "open-end:7",
                    "open-name:9..10",
                    "self-close:11",
                    "interpolation:14..15",
                    "text-entity:'&':17..22",
                    "close-name:24..30",
                    "open-name:32..33",
                    "self-close:34",
                    "end",
                ],
            );
        }
        // The existing component machine accepts case variants after its initial
        // two lowercase bytes. This correction does not broaden that legacy rule.
        assert_events::<Component>(
            "<scrIPT><b/>{{x}}&amp;</script><p/>",
            &[
                "open-name:1..7",
                "open-end:7",
                "text:8..22",
                "close-name:24..30",
                "open-name:32..33",
                "self-close:34",
                "end",
            ],
        );
    }

    #[test]
    fn document_special_name_prefixes_remain_ordinary_markup() {
        for source in ["<SCRIPTx><b/></SCRIPTx>", "<TEXTARE><b/></TEXTARE>"] {
            assert_events::<Document>(
                source,
                &[
                    "open-name:1..8",
                    "open-end:8",
                    "open-name:10..11",
                    "self-close:12",
                    "close-name:15..22",
                    "end",
                ],
            );
        }
    }

    #[test]
    fn document_rawtext_keeps_authored_utf8_byte_offsets() {
        assert_events::<Document>(
            "<SCRIPT>日本<b/></script>",
            &[
                "open-name:1..7",
                "open-end:7",
                "text:8..18",
                "close-name:20..26",
                "end",
            ],
        );
    }
}
