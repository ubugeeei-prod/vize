//! P2-2 pipeline-syntax laws: byte-exact round trip and exact rejections.
//!
//! The round-trip law is `print(parse(s)) == s` **byte-for-byte** for
//! canonical strings, which is why the grammar admits no whitespace and no
//! alternative spellings — with them, the law would have to be stated over an
//! equivalence class instead of over bytes.
//!
//! Every rejection is asserted on its **full** rendered message, never a
//! substring (assurance §4, "strict oracles — no partial matching", which
//! TS-13 enforces mechanically). The table in
//! `crates/vize_davinci/src/pass/pipeline.rs`'s module docs is the contract
//! these assertions pin: a message change has to be made in both places, on
//! purpose.

use vize_davinci::pass::pipeline::{PipelineSpec, PipelineSyntaxError};
use vize_davinci::pass::{parse_pipelines, print_pipelines};
use vize_s0::cstr;

/// Canonical strings, each of which must survive `print(parse(s)) == s`.
const CANONICAL: [&str; 8] = [
    "s2()",
    "s2(normalize)",
    "s2(normalize,fold)",
    "s2(normalize,fold),s2-to-s3(lower)",
    "s2-to-s3(lower)",
    "a(b)",
    "s2(pass-with-hyphens,x9)",
    "s2(),s3(),s4()",
];

#[test]
fn printing_a_parsed_canonical_string_reproduces_it_byte_for_byte() {
    for source in CANONICAL {
        let parsed = parse_pipelines(source)
            .unwrap_or_else(|error| panic!("{source} is canonical but was rejected: {error}"));
        assert_eq!(
            print_pipelines(&parsed).as_str(),
            source,
            "round trip changed the bytes of {source}"
        );
    }
}

#[test]
fn parsing_a_printed_value_reproduces_the_value() {
    for source in CANONICAL {
        let parsed = parse_pipelines(source).expect("canonical input parses");
        let printed = print_pipelines(&parsed);
        let reparsed = parse_pipelines(printed.as_str()).expect("printed output parses");
        assert_eq!(
            reparsed, parsed,
            "structural round trip failed for {source}"
        );
    }
}

#[test]
fn segments_carry_their_stage_and_pass_names_in_order() {
    let parsed = parse_pipelines("s2(normalize,fold),s2-to-s3(lower)").expect("input parses");
    assert_eq!(
        parsed,
        alloc_vec(&[
            PipelineSpec {
                stage: "s2",
                passes: alloc_vec(&["normalize", "fold"]),
            },
            PipelineSpec {
                stage: "s2-to-s3",
                passes: alloc_vec(&["lower"]),
            },
        ])
    );
}

#[test]
fn an_empty_pass_list_is_legal_and_prints_back_empty() {
    let parsed = parse_pipelines("s2()").expect("an empty pass list is legal");
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].stage, "s2");
    assert!(parsed[0].passes.is_empty());
    assert_eq!(print_pipelines(&parsed).as_str(), "s2()");
}

/// Every documented rejection, asserted on the full message and the variant.
///
/// The table in the module docs is the contract; this is its executable form.
#[test]
fn malformed_input_yields_the_exact_documented_error() {
    let cases: [(&str, PipelineSyntaxError, &str); 10] = [
        (
            "",
            PipelineSyntaxError::Empty,
            "empty pipeline string at offset 0",
        ),
        (
            "s2",
            PipelineSyntaxError::ExpectedOpenParen { offset: 2 },
            "expected `(` after stage name at offset 2",
        ),
        (
            "s2(a",
            PipelineSyntaxError::UnterminatedPassList { offset: 4 },
            "unterminated pass list, expected `)` at offset 4",
        ),
        (
            "(a)",
            PipelineSyntaxError::UnexpectedCharacter {
                offset: 0,
                character: '(',
            },
            "unexpected character `(` at offset 0",
        ),
        (
            "s2(,a)",
            PipelineSyntaxError::ExpectedPass { offset: 3 },
            "expected a pass name at offset 3",
        ),
        (
            "s2(a,)",
            PipelineSyntaxError::ExpectedPass { offset: 5 },
            "expected a pass name at offset 5",
        ),
        (
            "s2(a))",
            PipelineSyntaxError::ExpectedCommaOrEnd { offset: 5 },
            "expected `,` or end of input after `)` at offset 5",
        ),
        (
            "S2(a)",
            PipelineSyntaxError::UnexpectedCharacter {
                offset: 0,
                character: 'S',
            },
            "unexpected character `S` at offset 0",
        ),
        (
            "s2(a),",
            PipelineSyntaxError::ExpectedStage { offset: 6 },
            "expected a stage name at offset 6",
        ),
        (
            "s2-(a)",
            PipelineSyntaxError::TrailingHyphen { offset: 2 },
            "identifier must not end with `-` at offset 2",
        ),
    ];

    for (input, expected, rendered) in cases {
        let error = parse_pipelines(input)
            .err()
            .unwrap_or_else(|| panic!("{input:?} must be rejected"));
        assert_eq!(error, expected, "wrong variant for {input:?}");
        assert_eq!(
            cstr!("{error}").as_str(),
            rendered,
            "wrong rendering for {input:?}"
        );
        assert_eq!(error.offset(), expected.offset());
    }
}

#[test]
fn whitespace_is_not_a_spelling_of_the_canonical_form() {
    // Accepting these would break `print(parse(s)) == s` for exactly one of
    // the spellings, which is why they are rejected rather than normalized.
    // Pinned by variant and offset rather than `is_err()`: which production
    // rejects the space is part of the contract, and a rejection that moved
    // to a different rule would still pass a bare `is_err()`.
    let cases: [(&str, PipelineSyntaxError); 4] = [
        (
            "s2 (a)",
            PipelineSyntaxError::ExpectedOpenParen { offset: 2 },
        ),
        ("s2( a)", PipelineSyntaxError::ExpectedPass { offset: 3 }),
        ("s2(a, b)", PipelineSyntaxError::ExpectedPass { offset: 5 }),
        (
            " s2(a)",
            PipelineSyntaxError::UnexpectedCharacter {
                offset: 0,
                character: ' ',
            },
        ),
    ];
    for (input, expected) in cases {
        let error = parse_pipelines(input)
            .expect_err("whitespace has no canonical spelling, so it must be rejected");
        assert_eq!(error, expected, "wrong rejection for {input:?}");
    }
}

#[test]
fn an_underscore_is_not_an_identifier_byte() {
    let error = parse_pipelines("s_2(a)").expect_err("underscores are not kebab-case");
    assert_eq!(
        error,
        PipelineSyntaxError::ExpectedOpenParen { offset: 1 },
        "the ident stops at the underscore, so the `(` is what is missing"
    );
    assert_eq!(
        cstr!("{error}").as_str(),
        "expected `(` after stage name at offset 1"
    );
}

fn alloc_vec<T: Clone>(items: &[T]) -> Vec<T> {
    items.to_vec()
}
