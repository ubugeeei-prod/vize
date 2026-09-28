//! Exact event and recovery parity while #6835 converges the profile lexer.
//!
//! Opt in with `--features native-lex-parity`; the compiler still uses the
//! moved tokenizer through Armature's existing public path.

use vize_armature::tokenizer::{Callbacks, QuoteType as CompatQuote, Tokenizer};
use vize_l0::{SmallVec, String, cstr};
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
}
