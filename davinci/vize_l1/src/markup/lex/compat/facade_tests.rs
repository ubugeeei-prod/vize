//! Frozen callback contract captured before retiring the duplicate machine.
//!
//! Goldens include complete callback streams and exact mode-poll counts.
//! They are independent of the candidate native lexer.

use super::{Callbacks, QuoteType, Tokenizer};
use core::cell::Cell;
use vize_l0::{ErrorCode, String, cstr, hash::StableHasher128};

struct Probe<'a> {
    trace: &'a mut String,
    polls: &'a Cell<usize>,
    pause: Option<&'a Cell<bool>>,
    live_mode: bool,
    verbatim: bool,
}

macro_rules! events {
    ($($method:ident($($arg:ident: $kind:ty),*);)*) => {$ (
        fn $method(&mut self, $($arg: $kind),*) {
            self.trace.push_str(&cstr!("{}:{:?};", stringify!($method), ($($arg,)*)));
        }
    )*};
}

impl Callbacks for Probe<'_> {
    events! {
        on_text(start: usize, end: usize);
        on_text_entity(ch: char, start: usize, end: usize);
        on_interpolation(start: usize, end: usize);
        on_raw_interpolation(start: usize, end: usize);
        on_self_closing_tag(end: usize);
        on_attrib_data(start: usize, end: usize);
        on_attrib_entity(ch: char, start: usize, end: usize);
        on_attrib_end(quote: QuoteType, end: usize);
        on_attrib_name(start: usize, end: usize);
        on_attrib_name_end(end: usize);
        on_dir_name(start: usize, end: usize);
        on_dir_arg(start: usize, end: usize);
        on_dir_modifier(start: usize, end: usize);
        on_comment(start: usize, end: usize);
        on_in_tag_comment(start: usize, end: usize);
        on_cdata(start: usize, end: usize);
        on_processing_instruction(start: usize, end: usize);
        on_end();
        on_error(code: ErrorCode, index: usize);
    }

    fn on_open_tag_name(&mut self, start: usize, end: usize) {
        self.trace
            .push_str(&cstr!("on_open_tag_name:{:?};", (start, end)));
        if let Some(pause) = self.pause
            && !pause.replace(true)
        {
            panic!("deliberate callback pause");
        }
    }

    fn on_open_tag_end(&mut self, end: usize) {
        self.trace.push_str(&cstr!("on_open_tag_end:{end};"));
        if self.live_mode {
            self.verbatim = true;
        }
    }

    fn on_close_tag(&mut self, start: usize, end: usize) {
        self.trace.push_str(&cstr!("on_close_tag:{start}..{end};"));
        if self.live_mode {
            self.verbatim = false;
        }
    }

    fn is_in_v_pre(&self) -> bool {
        // The callback owns this switch. This probe tests live polling, not
        // the surface builder's unfinished dialect-owned scope controller.
        self.polls.set(self.polls.get() + 1);
        self.verbatim
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Switches {
    document: bool,
    raw: bool,
    comments: bool,
    live_mode: bool,
}

struct Oracle {
    expected: &'static [u8],
    index: usize,
}

impl Oracle {
    fn check(&mut self, fields: &[&[u8]], context: &str) {
        let mut hash = StableHasher128::new();
        for bytes in fields {
            hash.update(&(bytes.len() as u64).to_le_bytes());
            hash.update(bytes);
        }
        let digest = hash.digest();
        let start = self.index * 16;
        let expected = self.expected.get(start..start + 16);
        assert_eq!(
            Some(digest.as_slice()),
            expected,
            "{context}; observed fields: {fields:?}"
        );
        self.index += 1;
    }

    fn finish(&self) {
        assert_eq!(
            self.expected.len(),
            self.index * 16,
            "no stale frozen cases"
        );
    }
}

fn oracle(expected: &'static [u8]) -> Oracle {
    Oracle { expected, index: 0 }
}
fn feed(hasher: &mut StableHasher128, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

fn trace(source: &str, open: &[u8], close: &[u8], switches: Switches, repeat: bool) -> String {
    let mut result = String::default();
    let polls = Cell::new(0);
    {
        let probe = Probe {
            trace: &mut result,
            polls: &polls,
            pause: None,
            live_mode: switches.live_mode,
            verbatim: false,
        };
        let mut tokenizer = Tokenizer::with_delimiters(source, probe, open, close);
        tokenizer.set_tolerate_declarations(!switches.document);
        tokenizer.set_tolerate_declarations(switches.document);
        tokenizer.set_triple_mustache(!switches.raw);
        tokenizer.set_triple_mustache(switches.raw);
        tokenizer.set_in_tag_comments(!switches.comments);
        tokenizer.set_in_tag_comments(switches.comments);
        tokenizer.tokenize();
        if repeat {
            // Public mutators remain valid after EOF. A new lexer would reset the
            // cursor and emit the source again, violating this frozen contract.
            tokenizer.set_tolerate_declarations(!switches.document);
            tokenizer.set_triple_mustache(!switches.raw);
            tokenizer.set_in_tag_comments(!switches.comments);
            tokenizer.tokenize();
            tokenizer.set_tolerate_declarations(switches.document);
            tokenizer.set_triple_mustache(switches.raw);
            tokenizer.set_in_tag_comments(switches.comments);
            tokenizer.tokenize();
        }
    }
    result.push_str(&cstr!("mode-polls:{};", polls.get()));
    result
}

#[test]
fn fixture_callbacks_and_utf8_cuts_keep_the_frozen_machine_contract() {
    let mut cases = oracle(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/_fixtures/tokenizer_callback_fixtures.bin"
    )));
    let mut hash = StableHasher128::new();
    for switches in [
        Switches::default(),
        Switches {
            document: true,
            ..Switches::default()
        },
        Switches {
            raw: true,
            comments: true,
            ..Switches::default()
        },
    ] {
        for fixture in davinci_test_support::surface_fixture::WELL_FORMED
            .iter()
            .chain(davinci_test_support::surface_fixture::MALFORMED)
        {
            for (cut_index, source) in core::iter::once(fixture.source)
                .chain(fixture.source.char_indices().flat_map(|(index, _)| {
                    [
                        fixture.source.get(..index).unwrap(),
                        fixture.source.get(index..).unwrap(),
                    ]
                }))
                .enumerate()
            {
                let output = trace(source, b"{{", b"}}", switches, true);
                let kind = if cut_index == 0 {
                    "full"
                } else if cut_index % 2 == 1 {
                    "prefix"
                } else {
                    "suffix"
                };
                let cut = if kind == "suffix" {
                    fixture.source.len() - source.len()
                } else {
                    source.len()
                };
                cases.check(
                    &[source.as_bytes(), output.as_bytes()],
                    &cstr!("{switches:?}; {} {kind}@{cut}", fixture.name),
                );
                feed(&mut hash, source.as_bytes());
                feed(&mut hash, output.as_bytes());
            }
        }
    }
    cases.finish();
    assert_eq!(
        hash.digest(),
        [
            173, 64, 234, 60, 59, 234, 59, 248, 88, 106, 171, 216, 217, 247, 117, 32
        ],
        "captured source: 5bca3a881"
    );
}

#[test]
fn public_switches_delimiters_live_callbacks_and_repeated_eof_keep_the_frozen_contract() {
    let mut cases = oracle(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/_fixtures/tokenizer_callback_switches.bin"
    )));
    let mut hash = StableHasher128::new();
    for (source, open, close, switches) in [
        (
            "abc",
            b"{{".as_slice(),
            b"}}".as_slice(),
            Switches::default(),
        ),
        ("", b"{{", b"}}", Switches::default()),
        ("{{ value", b"{{", b"}}", Switches::default()),
        ("<!-- unfinished", b"{{", b"}}", Switches::default()),
        ("<div title='unfinished", b"{{", b"}}", Switches::default()),
        (
            "<!DOCTYPE html><?pi?><![CDATA[x]]><p>&fjlig; {{x}}</p>",
            b"{{",
            b"}}",
            Switches {
                document: true,
                ..Switches::default()
            },
        ),
        (
            "<X // comment\n :title='&acE;'/>{{x}}",
            b"{{",
            b"}}",
            Switches {
                comments: true,
                ..Switches::default()
            },
        ),
        (
            "{{{ raw }}}",
            b"{{",
            b"}}",
            Switches {
                raw: true,
                ..Switches::default()
            },
        ),
        (
            "{{{ raw ]]}",
            b"{{",
            b"]]",
            Switches {
                raw: true,
                ..Switches::default()
            },
        ),
        (
            "[[{ raw ]]]",
            b"[[",
            b"]]",
            Switches {
                raw: true,
                ..Switches::default()
            },
        ),
        ("{{x}}", b"", b"}}", Switches::default()),
        ("{{x}}", b"{{", b"", Switches::default()),
        ("🙂val🙂tail", b"\xf0\x9f", b"\x99\x82", Switches::default()),
        (
            "<p>{{in}}<b :name='x'>&fjlig;</b>{{out}}</p>",
            b"{{",
            b"}}",
            Switches {
                live_mode: true,
                ..Switches::default()
            },
        ),
    ] {
        let output = trace(source, open, close, switches, true);
        cases.check(
            &[source.as_bytes(), output.as_bytes()],
            &cstr!("switches {switches:?}; delimiters {open:?}/{close:?}"),
        );
        feed(&mut hash, source.as_bytes());
        feed(&mut hash, output.as_bytes());
    }
    cases.finish();
    assert_eq!(
        hash.digest(),
        [
            201, 33, 1, 114, 254, 142, 42, 99, 91, 170, 10, 136, 119, 246, 107, 72
        ],
        "captured source: 5bca3a881"
    );
}

mod resume;
