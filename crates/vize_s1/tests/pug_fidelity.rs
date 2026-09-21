#![allow(clippy::disallowed_types)] // Fixture files arrive through `std::fs` as std strings.
//! TS-19 for the pug dialect (Davinci P4-12c): `render(parse_pug(src))
//! == src` as bytes over the committed pug fixture matrix, a malformed
//! battery with its typed holes and recovered error codes pinned exactly,
//! every prefix and suffix truncation of all of it, and a deterministic
//! generated corpus — exact-equality oracles only (assurance §4).

use std::path::PathBuf;

use vize_s0::{Allocator, String};
use vize_s1::pug::{
    PugErrorCode, PugHoleCounts, PugNode, check_pug_fidelity, parse_pug, pug_hole_counts,
    render_pug,
};

fn rendered(source: &str) -> String {
    let allocator = Allocator::new();
    let (tree, _errors) = parse_pug(&allocator, source);
    let mut out = String::default();
    render_pug(&tree, &mut |piece| out.push_str(piece));
    assert_eq!(
        check_pug_fidelity(&tree),
        Ok(()),
        "fidelity check: {source:?}"
    );
    out
}

fn assert_fidelity(source: &str) {
    assert_eq!(
        rendered(source).as_str(),
        source,
        "render(parse(src)) != src"
    );
}

/// Every committed `.pug` fixture (the P4-12c matrix and refusal sets).
fn fixture_sources() -> Vec<(std::string::String, std::string::String)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/_fixtures/davinci-pug");
    let mut sources = Vec::new();
    for dir in ["matrix", "refused"] {
        let mut entries: Vec<_> = std::fs::read_dir(root.join(dir))
            .expect("pug fixture directory")
            .map(|entry| entry.expect("fixture entry").path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "pug"))
            .collect();
        entries.sort();
        for path in entries {
            let name = path.file_stem().unwrap().to_string_lossy().into_owned();
            sources.push((name, std::fs::read_to_string(&path).expect("fixture")));
        }
    }
    sources
}

/// Malformed pug: every recovered error code and typed hole, pinned.
const MALFORMED: &[(&str, &str, &[PugErrorCode], PugHoleCounts)] = {
    use PugErrorCode::*;
    const fn holes(missing_tokens: usize, unexpected: usize) -> PugHoleCounts {
        PugHoleCounts {
            missing_tokens,
            unexpected,
        }
    }
    &[
        (
            "unclosed-attrs",
            "div(a=\"1\"\n  span",
            &[NoEndBracket],
            holes(0, 1),
        ),
        ("bad-class", "p.1 x", &[InvalidClassName], holes(0, 1)),
        ("bad-id", "# x", &[InvalidId], holes(0, 1)),
        (
            "mixed-indent",
            "div\n  span\n\tb",
            &[InvalidIndentation],
            holes(0, 0),
        ),
        (
            "inconsistent-indent",
            "div\n    a\n  b",
            &[InconsistentIndentation],
            holes(0, 0),
        ),
        (
            "open-tag-interpolation",
            "p #[b x",
            &[NoEndBracket],
            holes(1, 0),
        ),
        (
            "open-code-interpolation",
            "p #{x",
            &[NoEndBracket],
            holes(0, 0),
        ),
        (
            "key-junk",
            "div(\"a\"b c)",
            &[InvalidAttribute],
            holes(0, 1),
        ),
        (
            "bang-without-eq",
            "div(a!b)",
            &[InvalidAttribute],
            holes(0, 1),
        ),
        ("stray-paren", ")", &[UnexpectedText], holes(0, 1)),
        (
            "orphan-indent",
            "div\n  | a\n    | b",
            &[InvalidToken],
            holes(0, 1),
        ),
        ("expansion-at-eol", "p: ", &[InvalidToken], holes(0, 1)),
        (
            "mismatched-bracket",
            "div(a=[)",
            &[BracketMismatch],
            holes(0, 1),
        ),
        (
            "tag-interpolation-at-line-start",
            "#[b x]",
            &[InvalidId],
            holes(0, 1),
        ),
        (
            "empty-tag-interpolation",
            "p #[]",
            &[InvalidToken],
            holes(0, 0),
        ),
        ("nested-bare-dot", "div\n  .", &[InvalidToken], holes(0, 0)),
        ("junk-after-tag", "p<b>", &[InvalidToken], holes(0, 1)),
        ("malformed-case", "case", &[MalformedKeyword], holes(0, 0)),
    ]
};

#[test]
fn ts19_pug_fixture_matrix_round_trips_without_errors() {
    let sources = fixture_sources();
    assert_eq!(sources.len(), 40, "26 matrix + 14 refused fixtures, pinned");
    for (name, source) in &sources {
        let allocator = Allocator::new();
        let (tree, errors) = parse_pug(&allocator, source);
        assert_eq!(check_pug_fidelity(&tree), Ok(()), "fidelity: {name}");
        assert!(errors.is_empty(), "{name}: {:?}", errors.as_slice());
        assert_eq!(pug_hole_counts(&tree), PugHoleCounts::default(), "{name}");
    }
}

#[test]
fn ts19_pug_malformed_battery_pins_errors_and_holes() {
    assert_eq!(MALFORMED.len(), 18);
    for (name, source, codes, holes) in MALFORMED {
        let allocator = Allocator::new();
        let (tree, errors) = parse_pug(&allocator, source);
        assert_eq!(check_pug_fidelity(&tree), Ok(()), "fidelity: {name}");
        let found: Vec<_> = errors.iter().map(|error| error.code).collect();
        assert_eq!(found.as_slice(), *codes, "error codes: {name}");
        assert_eq!(pug_hole_counts(&tree), *holes, "hole census: {name}");
    }
}

#[test]
fn ts19_pug_truncations_round_trip() {
    let sources = fixture_sources();
    let all = sources
        .iter()
        .map(|(_, source)| source.as_str())
        .chain(MALFORMED.iter().map(|(_, source, _, _)| *source));
    for source in all {
        for (at, _) in source.char_indices() {
            assert_fidelity(&source[..at]);
            assert_fidelity(&source[at..]);
        }
    }
}

/// A deterministic xorshift over pug-significant fragments: thousands of
/// adversarial inputs, identical on every run.
#[test]
fn ts19_pug_generated_inputs_round_trip() {
    const FRAGMENTS: &[&str] = &[
        "div",
        "p",
        ".a",
        "#b",
        "(",
        ")",
        "a=\"x\"",
        ",",
        " ",
        "  ",
        "\t",
        "\n",
        "\r\n",
        "\r",
        "\n  ",
        "\n    ",
        "|",
        "| t",
        ".",
        ":",
        ": ",
        "/",
        "=",
        "!=",
        "-",
        "#[",
        "]",
        "#{",
        "}",
        "\\",
        "//",
        "//-",
        "<b>",
        "&attributes(x)",
        "'",
        "\"",
        "`",
        "{{ x }}",
        "mixin",
        "+m",
        "if",
        "each",
        "é",
        "\u{feff}",
        "img",
        "[",
        "!",
    ];
    let mut state = 0x9e37_79b9_u32;
    let mut next = |bound: usize| {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        state as usize % bound
    };
    for _ in 0..3000 {
        let mut source = std::string::String::new();
        for _ in 0..next(24) {
            source.push_str(FRAGMENTS[next(FRAGMENTS.len())]);
        }
        assert_fidelity(&source);
    }
}

#[test]
fn deep_nesting_is_stack_safe() {
    let mut indented = std::string::String::new();
    for depth in 0..3000 {
        indented.push_str(&" ".repeat(depth));
        indented.push_str("div\n");
    }
    assert_fidelity(&indented);
    assert_fidelity(&"a: ".repeat(20_000));
}

#[test]
fn the_tree_follows_pug_structure() {
    let allocator = Allocator::new();
    let (tree, errors) = parse_pug(
        &allocator,
        "ul\n  li: a(href=\"/\") Home\n  //- trivia\n  li two\n",
    );
    assert!(errors.is_empty());
    assert_eq!(tree.nodes.len(), 1);
    let PugNode::Tag(list) = &tree.nodes[0] else {
        panic!("a tag");
    };
    assert_eq!(list.tag_name(), "ul");
    let vize_s1::pug::PugBlock::Nodes(items) = &list.block else {
        panic!("children");
    };
    // The `//-` comment is trivia: pug strips it, so it is not a node.
    assert_eq!(items.len(), 2);
    let PugNode::Tag(second) = &items[1] else {
        panic!("a tag");
    };
    let name = second.name.expect("named");
    assert!(
        name.leading.contains("//- trivia"),
        "trivia rides in leading"
    );
}
