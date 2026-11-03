//! Consumer contracts for the ordinary library build, without `cfg(test)` or
//! `native-markup-lex` on vize_l1 itself.

use vize_l0::SmallVec;
use vize_l1::markup::entity::{DecodedEntity, EntityContext, decode_one};
use vize_l1::markup::{
    Component, Delimiters, Document, LexErrorCode, LexOptions, Lexer, Profile, QuoteType, Sink,
};

#[derive(Default)]
struct Trace {
    ranges: SmallVec<[(&'static str, usize, usize); 32]>,
    entities: SmallVec<[(char, usize, usize); 8]>,
    errors: SmallVec<[(LexErrorCode, usize); 4]>,
    ended: bool,
}

macro_rules! range_methods {
    ($($method:ident),* $(,)?) => {
        $(fn $method(&mut self, start: usize, end: usize) {
            self.ranges.push((stringify!($method), start, end));
        })*
    };
}

impl Sink for Trace {
    range_methods!(
        on_text,
        on_interpolation,
        on_raw_interpolation,
        on_open_tag_name,
        on_close_tag,
        on_attrib_data,
        on_attrib_name,
        on_dir_name,
        on_dir_arg,
        on_dir_modifier,
        on_comment,
        on_cdata,
        on_processing_instruction,
    );

    fn on_text_entity(&mut self, ch: char, start: usize, end: usize) {
        self.entities.push((ch, start, end));
    }
    fn on_attrib_entity(&mut self, ch: char, start: usize, end: usize) {
        self.entities.push((ch, start, end));
    }
    fn on_open_tag_end(&mut self, end: usize) {
        self.ranges.push(("on_open_tag_end", end, end));
    }
    fn on_self_closing_tag(&mut self, end: usize) {
        self.ranges.push(("on_self_closing_tag", end, end));
    }
    fn on_attrib_end(&mut self, _quote: QuoteType, end: usize) {
        self.ranges.push(("on_attrib_end", end, end));
    }
    fn on_attrib_name_end(&mut self, end: usize) {
        self.ranges.push(("on_attrib_name_end", end, end));
    }
    fn on_error(&mut self, code: LexErrorCode, index: usize) {
        self.errors.push((code, index));
    }
    fn on_end(&mut self) {
        self.ended = true;
    }
}

fn lex<P: Profile>(source: &str, options: LexOptions<'_>) -> Trace {
    let mut lexer = Lexer::<P, _>::new(source, Trace::default(), options);
    assert_eq!(lexer.input(), source.as_bytes());
    lexer.run();
    let trace = lexer.into_sink();
    assert!(trace.ended);
    for &(kind, start, end) in &trace.ranges {
        assert!(
            start <= end && end <= source.len(),
            "{kind} {start}..{end} in {source:?}"
        );
    }
    for &(_, start, end) in &trace.entities {
        assert!(source.get(start..end).is_some());
    }
    trace
}

#[test]
fn default_library_exposes_both_profiles_and_their_declaration_rule() {
    let source = "<!DOCTYPE html><p>{{ value }}</p>";
    let document = lex::<Document>(source, LexOptions::default());
    assert!(document.errors.is_empty());
    assert!(
        document
            .ranges
            .iter()
            .any(|&(kind, _, _)| kind == "on_interpolation")
    );
    let component = lex::<Component>(source, LexOptions::default());
    assert!(
        component
            .errors
            .iter()
            .any(|&(code, _)| code == LexErrorCode::IncorrectlyOpenedComment)
    );
}

#[test]
fn default_entity_provider_preserves_full_values_context_and_source_spans() {
    for context in [EntityContext::Text, EntityContext::Attribute] {
        assert_eq!(
            decode_one(b"&fjlig;tail", context),
            Some((DecodedEntity::Named("fj"), 7))
        );
    }
    assert_eq!(
        decode_one(b"&timesX", EntityContext::Text),
        Some((DecodedEntity::Named("×"), 6))
    );
    assert_eq!(decode_one(b"&timesX", EntityContext::Attribute), None);
    assert_eq!(
        decode_one(b"&#128;tail", EntityContext::Text),
        Some((DecodedEntity::Numeric('€'), 6))
    );
    let source = "<p title='&fjlig;'>&fjlig;</p>";
    let trace = lex::<Component>(source, LexOptions::default());
    assert!(trace.errors.is_empty());
    assert_eq!(
        trace.entities.as_slice(),
        [('f', 10, 17), ('j', 10, 17), ('f', 19, 26), ('j', 19, 26)]
    );
}

#[test]
fn default_provider_is_total_on_every_surface_fixture_and_utf8_truncation() {
    for fixture in davinci_test_support::surface_fixture::WELL_FORMED
        .iter()
        .chain(davinci_test_support::surface_fixture::MALFORMED)
    {
        lex::<Component>(fixture.source, LexOptions::default());
        for (index, _) in fixture.source.char_indices() {
            for source in [
                fixture.source.get(..index).expect("UTF-8 prefix"),
                fixture.source.get(index..).expect("UTF-8 suffix"),
            ] {
                lex::<Component>(source, LexOptions::default());
                lex::<Document>(source, LexOptions::default());
            }
        }
    }
}

#[test]
fn ordinary_consumers_can_configure_delimiters_and_raw_interpolation() {
    let custom = lex::<Component>(
        "[[ value ]]",
        LexOptions {
            delimiters: Delimiters {
                open: b"[[",
                close: b"]]",
            },
            ..LexOptions::default()
        },
    );
    assert!(custom.errors.is_empty());
    assert_eq!(custom.ranges.as_slice(), [("on_interpolation", 2, 9)]);
    let raw = lex::<Component>(
        "{{{ value }}}",
        LexOptions {
            raw_interpolation: true,
            ..LexOptions::default()
        },
    );
    assert!(raw.errors.is_empty());
    assert_eq!(raw.ranges.as_slice(), [("on_raw_interpolation", 3, 10)]);
}
