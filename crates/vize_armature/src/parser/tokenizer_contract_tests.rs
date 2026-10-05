//! Complete parser AST/diagnostic fingerprints captured before lexer retirement.
//!
//! Parser callbacks remain the production legacy route. These frozen witnesses
//! cover their actual v-pre scopes, namespace/recovery and dialect options.

use super::{Parser, WhitespaceStrategy};
use vize_l0::{Allocator, config::VueVersion, cstr, hash::StableHasher128};
use vize_relief::options::ParserOptions;

struct Oracle {
    expected: &'static [u8],
    index: usize,
    mismatches: usize,
    capture: Option<std::fs::File>,
}

impl Oracle {
    fn check(&mut self, fields: &[&[u8]], context: &str, options: &str) {
        let mut hash = StableHasher128::new();
        for bytes in fields {
            hash.update(&(bytes.len() as u64).to_le_bytes());
            hash.update(bytes);
        }
        let digest = hash.digest();
        let start = self.index * 16;
        let expected = self.expected.get(start..start + 16);
        if let Some(capture) = &mut self.capture {
            write_capture(
                capture,
                self.index as u64,
                &[
                    context.as_bytes(),
                    fields[0],
                    fields[1],
                    fields[2],
                    options.as_bytes(),
                    expected.unwrap_or_default(),
                    &digest,
                ],
            );
        }
        if Some(digest.as_slice()) != expected {
            self.mismatches += 1;
            eprintln!(
                "frozen case {}: {context}; expected {expected:?}; actual {digest:?}",
                self.index
            );
        }
        self.index += 1;
    }

    fn finish(&mut self, aggregate: &[u8]) {
        if let Some(capture) = &mut self.capture {
            write_capture(capture, u64::MAX, &[aggregate]);
        }
        assert_eq!(
            self.expected.len(),
            self.index * 16,
            "no stale frozen cases"
        );
        assert_eq!(
            self.mismatches, 0,
            "all frozen cases must match; no failed case is skipped"
        );
    }
}

fn oracle(expected: &'static [u8], name: &str) -> Oracle {
    let capture = std::env::var_os("VIZE_TOKENIZER_CAPTURE_DIR").map(|directory| {
        std::fs::File::create(std::path::Path::new(&directory).join(name)).unwrap()
    });
    Oracle {
        expected,
        index: 0,
        mismatches: 0,
        capture,
    }
}

// Test-only length-prefixed fields retain whole source, options, AST and errors.
fn write_capture(file: &mut std::fs::File, index: u64, fields: &[&[u8]]) {
    use std::io::Write;
    file.write_all(&index.to_le_bytes()).unwrap();
    file.write_all(&(fields.len() as u64).to_le_bytes())
        .unwrap();
    for bytes in fields {
        file.write_all(&(bytes.len() as u64).to_le_bytes()).unwrap();
        file.write_all(bytes).unwrap();
    }
}
fn feed(hash: &mut StableHasher128, bytes: &[u8]) {
    hash.update(&(bytes.len() as u64).to_le_bytes());
    hash.update(bytes);
}

fn record(
    hash: &mut StableHasher128,
    cases: &mut Oracle,
    source: &str,
    document: bool,
    options: ParserOptions,
    context: &str,
) {
    let option_fields = cstr!("{options:?}");
    let allocator = Allocator::new();
    let (tree, errors) = if document {
        Parser::document_with_options(&allocator, source, options).parse()
    } else {
        Parser::with_options(&allocator, source, options).parse()
    };
    let tree = cstr!("{tree:#?}");
    let errors = cstr!("{errors:#?}");
    cases.check(
        &[source.as_bytes(), tree.as_bytes(), errors.as_bytes()],
        context,
        &option_fields,
    );
    feed(hash, source.as_bytes());
    feed(hash, tree.as_bytes());
    feed(hash, errors.as_bytes());
}

#[test]
fn complete_fixture_asts_and_utf8_cuts_keep_the_frozen_tokenizer_contract() {
    let mut cases = oracle(
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/_fixtures/tokenizer_parser_fixtures.bin"
        )),
        "fixtures.bin",
    );
    let mut hash = StableHasher128::new();
    for document in [false, true] {
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
                record(
                    &mut hash,
                    &mut cases,
                    source,
                    document,
                    ParserOptions::default(),
                    &cstr!("document={document}; {} {kind}@{cut}", fixture.name),
                );
            }
        }
    }
    cases.finish(&hash.digest());
    assert_eq!(
        hash.digest(),
        [
            15, 118, 217, 141, 23, 28, 146, 11, 122, 149, 63, 93, 246, 250, 5, 228
        ],
        "captured source: 5bca3a881"
    );
}

#[test]
fn live_v_pre_recovery_and_dialect_options_keep_the_frozen_parser_contract() {
    let mut cases = oracle(
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/_fixtures/tokenizer_parser_options.bin"
        )),
        "options.bin",
    );
    let mut hash = StableHasher128::new();
    for source in [
        "<p v-pre :title='value'>{{ literal }}<i @click='f'>{{nested}}</i></p>{{normal}}",
        "<p v-pre.foo>{{ literal }}</p>{{normal}}",
        "<p v-pre:arg>{{ literal }}</p>{{normal}}",
        "<p v-pre:[unterminated>{{ literal }}</p>{{normal}}",
        "<section><a><span><a></a><span v-pre>{{ inside }}</span>{{ tail }}</span></a></section>",
        "<svg><foreignObject><p v-pre>{{x}}</p></foreignObject></svg>{{y}}",
        "<p v-pre title='&fjlig;'>&fjlig; {{x}}</p>",
        "<p v-pre :broken='unfinished",
    ] {
        record(
            &mut hash,
            &mut cases,
            source,
            false,
            ParserOptions::default(),
            source,
        );
    }
    record(
        &mut hash,
        &mut cases,
        "<p>[[value]] and {{literal}}</p>",
        false,
        ParserOptions {
            delimiters: ("[[".into(), "]]".into()),
            ..ParserOptions::default()
        },
        "dialect/profile option",
    );
    record(
        &mut hash,
        &mut cases,
        "<X // comment\n :title='&acE;'/>",
        false,
        ParserOptions {
            experimental_in_tag_comments: true,
            ..ParserOptions::default()
        },
        "dialect/profile option",
    );
    record(
        &mut hash,
        &mut cases,
        "<p>{{{raw}}} and {{escaped}}</p>",
        false,
        ParserOptions {
            dialect: VueVersion::V1,
            ..ParserOptions::default()
        },
        "dialect/profile option",
    );
    record(
        &mut hash,
        &mut cases,
        "<!DOCTYPE html><table><tr><td>{{x}}</td></tr></table>",
        true,
        ParserOptions {
            whitespace: WhitespaceStrategy::Preserve,
            ..ParserOptions::default()
        },
        "dialect/profile option",
    );
    cases.finish(&hash.digest());
    assert_eq!(
        hash.digest(),
        [
            139, 236, 84, 26, 227, 6, 37, 205, 133, 21, 41, 76, 178, 19, 23, 75
        ],
        "captured source: 5bca3a881"
    );
}
