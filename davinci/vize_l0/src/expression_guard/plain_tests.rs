//! The short-text certificate never admits text the complete scan refuses.

use super::analysis_admits;
use super::plain::{MAX_PLAIN_EXPRESSION_BYTES, is_short_plain_expression};

/// Bytes the certificate reads, bytes it must refuse, and keyword letters for
/// prefix-operator runs.
const RICH: &[u8] = b"av ()[]{}.,:?=!~+-*%^&|>'\"`${}<\\/@#\n\t0_dneotypw";
/// Mostly admitted bytes, so long admitted texts are generated too.
const PLAIN: &[u8] = b"ab (){}[].,:?=!+&|>'`${}0_";

fn generated(alphabet: &[u8], seed: u64, count: usize) -> usize {
    // xorshift64*, so the generated text is identical on every run.
    let mut state = seed;
    let mut next = move || {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        state.wrapping_mul(0x2545_f491_4f6c_dd1d)
    };
    let mut admitted = 0usize;
    let mut text = Vec::with_capacity(MAX_PLAIN_EXPRESSION_BYTES + 8);
    for _ in 0..count {
        text.clear();
        let len = (next() % (MAX_PLAIN_EXPRESSION_BYTES as u64 + 8)) as usize;
        for _ in 0..len {
            let index = (next() % alphabet.len() as u64) as usize;
            text.push(alphabet.get(index).copied().unwrap_or(b'a'));
        }
        let Ok(content) = core::str::from_utf8(&text) else {
            continue;
        };
        if is_short_plain_expression(content) {
            admitted += 1;
            assert!(analysis_admits(content), "certificate admitted {content:?}");
        }
    }
    admitted
}

/// One byte of every class the certificate distinguishes: identifier, digit,
/// space, a plain operator, a prefix operator, each delimiter, both quotes,
/// backtick and `$`, plus `<` (refused in code), `\` (refused everywhere) and
/// a line break (which ends a string).
const EXHAUSTIVE: &[u8] = b"a0 .!()[]{}'\"`$<\\\n";
/// Every text up to this length is checked.
const EXHAUSTIVE_LEN: u32 = 6;

#[test]
fn certificate_agrees_with_the_complete_scan_on_every_short_text() {
    let mut text = Vec::with_capacity(EXHAUSTIVE_LEN as usize);
    let mut checked = 0usize;
    for len in 0..=EXHAUSTIVE_LEN {
        let total = EXHAUSTIVE.len().pow(len);
        for mut code in 0..total {
            text.clear();
            for _ in 0..len {
                text.push(
                    EXHAUSTIVE
                        .get(code % EXHAUSTIVE.len())
                        .copied()
                        .unwrap_or(b'a'),
                );
                code /= EXHAUSTIVE.len();
            }
            let Ok(content) = core::str::from_utf8(&text) else {
                continue;
            };
            if is_short_plain_expression(content) {
                assert!(analysis_admits(content), "certificate admitted {content:?}");
            }
            checked += 1;
        }
    }
    assert_eq!(
        checked,
        (0..=EXHAUSTIVE_LEN)
            .map(|len| EXHAUSTIVE.len().pow(len))
            .sum::<usize>()
    );
}

#[test]
fn certificate_agrees_with_the_complete_scan_on_generated_text() {
    assert!(generated(RICH, 0x9e37_79b9_7f4a_7c15, 300_000) > 1_000);
    assert!(generated(PLAIN, 0x6a09_e667_f3bc_c908, 300_000) > 10_000);
}

#[test]
fn certificate_admits_template_expression_shapes() {
    for content in [
        "post.title",
        "item in navItems",
        "formatDate(post.date)",
        "items[index].label",
        "{ 'header--sticky': isSticky }",
        "`/posts/${post.id}`",
        "`${a}-${`${b}`}`",
        "!loading && count > 0",
        "a === b ? c : d",
        "(x) => x * 2",
        "user?.profile?.name ?? 'Guest'",
        "\"é\"",
        "   ",
    ] {
        assert!(is_short_plain_expression(content), "{content:?}");
        assert!(analysis_admits(content), "{content:?}");
    }
    let deepest = "(".repeat(31) + "a" + &")".repeat(31);
    assert!(is_short_plain_expression(&deepest));
    assert!(analysis_admits(&deepest));
}

#[test]
fn certificate_defers_every_other_shape_to_the_complete_scan() {
    let too_deep = "(".repeat(32) + &")".repeat(32);
    let prefix_run = "!".repeat(32) + "a";
    for content in [
        "",
        too_deep.as_str(),
        prefix_run.as_str(),
        "a(]",
        "(a",
        "a)",
        "{a",
        "a < b",
        "a / b",
        "'a\\'b'",
        "'a",
        "'a\nb'",
        "`a\\``",
        "`a",
        "`${a`",
        "a\tb",
        "@a",
        "#a",
        "a;b",
        "é",
    ] {
        assert!(!is_short_plain_expression(content), "{content:?}");
    }
    assert!(!analysis_admits(&too_deep));
    assert!(!analysis_admits(&prefix_run));
}

/// Callers answer empty text before they consult the guard, which relies on
/// the guard admitting it.
#[test]
fn the_complete_scan_admits_empty_text() {
    assert!(super::expression_is_safe_to_parse(""));
}
