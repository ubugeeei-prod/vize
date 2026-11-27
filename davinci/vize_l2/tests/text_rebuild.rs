//! Differential test for `rebuild_source`'s single-pass copy (it writes into
//! the string's spare capacity under `unsafe`): seeded random part lists must
//! rebuild byte for byte as the plain `push_str` spelling does. The lengths
//! straddle the 24-byte inline capacity, and the text mixes 1- to 4-byte
//! UTF-8 so a piece boundary never lands inside a character by accident.
//! `tests/fuzz/fuzz_targets/text_rebuild.rs` runs the same oracle under
//! libFuzzer.

use vize_l0::{Span, String};
use vize_l2::analysis::text_analysis::{TextPart, rebuild_source};

/// The rebuild rule spelled with one `push_str` per piece.
fn oracle(parts: &[TextPart]) -> String {
    let mut out = String::default();
    for part in parts {
        if part.raw_html {
            out.push_str("{{{ ");
            out.push_str(&part.text);
            out.push_str(" }}}");
        } else if part.dynamic {
            out.push_str("{{ ");
            out.push_str(&part.text);
            out.push_str(" }}");
        } else {
            out.push_str(&part.text);
        }
    }
    out
}

/// xorshift64*: deterministic, so a failure names its seed.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn below(&mut self, bound: u64) -> usize {
        usize::try_from(self.next() % bound).unwrap_or(0)
    }
}

const ALPHABET: [char; 10] = ['a', 'Z', ' ', '{', '}', '<', 'é', '日', '😀', '\n'];

fn part(rng: &mut Rng) -> TextPart {
    let length = rng.below(30);
    let text: String = (0..length)
        .map(|_| ALPHABET.get(rng.below(10)).copied().unwrap_or('a'))
        .collect();
    let kind = rng.below(3);
    TextPart {
        text,
        span: Span::new(0, 0),
        dynamic: kind > 0,
        raw_html: kind == 2,
    }
}

#[test]
fn rebuild_matches_the_push_str_spelling() {
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    for case in 0..100_000 {
        let count = rng.below(7);
        let parts: Vec<TextPart> = (0..count).map(|_| part(&mut rng)).collect();
        let rebuilt = rebuild_source(&parts);
        let expected = oracle(&parts);
        assert_eq!(rebuilt, expected, "case {case}: {parts:?}");
        assert_eq!(rebuilt.len(), expected.len(), "case {case}");
    }
}

#[test]
fn rebuild_fills_the_inline_capacity_exactly() {
    // 24 bytes is the inline capacity, so the last byte also holds the tag.
    for split in 0..=18 {
        let parts = [
            TextPart {
                text: String::from("x".repeat(split)),
                span: Span::new(0, 0),
                dynamic: false,
                raw_html: false,
            },
            TextPart {
                text: String::from("y".repeat(18 - split)),
                span: Span::new(0, 0),
                dynamic: true,
                raw_html: false,
            },
        ];
        let rebuilt = rebuild_source(&parts);
        assert_eq!(rebuilt.len(), 24);
        assert_eq!(rebuilt, oracle(&parts));
        assert!(!rebuilt.is_heap_allocated());
    }
    // One past it moves to the heap.
    let parts = [TextPart {
        text: String::from("é".repeat(9)),
        span: Span::new(0, 0),
        dynamic: true,
        raw_html: true,
    }];
    assert_eq!(rebuild_source(&parts), oracle(&parts));
    assert_eq!(rebuild_source(&[]), "");
}
