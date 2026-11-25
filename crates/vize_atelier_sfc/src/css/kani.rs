//! Kani checks the shipped CSS string-body step (`string_body.rs`) that both
//! scoped CSS scanners use to find where a quoted string ends.
//!
//! The spec is CSS Syntax Level 3, "Consume a string token": the string ends
//! at the first code point equal to its opening quote that no REVERSE
//! SOLIDUS escapes, and a REVERSE SOLIDUS escapes the one code point after
//! it. A quote is therefore escaped exactly when an odd run of backslashes
//! precedes it. The spec below counts that run; it does not replay the step.
//!
//! Bound: the step law ranges over every Unicode scalar, both quotes and both
//! escape states, so it is exhaustive. The sequence law walks every string
//! body of four scalars, which holds every backslash run of length zero to
//! three in front of a quote.

mod string_body;

#[cfg(kani)]
mod proofs {
    use super::string_body::StringBody;

    const LEN: usize = 4;

    fn any_quote() -> char {
        if kani::any() { '"' } else { '\'' }
    }

    /// The index of the code point that ends the string, by the statement:
    /// the first opening quote preceded by an even run of backslashes.
    fn specified_end(body: &[char; LEN], quote: char) -> Option<usize> {
        let mut backslashes = 0usize;
        let mut index = 0;
        while index < LEN {
            let c = body[index];
            if c == quote && backslashes % 2 == 0 {
                return Some(index);
            }
            backslashes = if c == '\\' { backslashes + 1 } else { 0 };
            index += 1;
        }
        None
    }

    /// The index at which the shipped step reports the end.
    fn stepped_end(body: &[char; LEN], quote: char) -> Option<usize> {
        let mut state = StringBody::open(quote);
        let mut index = 0;
        while index < LEN {
            match state.step(body[index]) {
                Some(next) => state = next,
                None => return Some(index),
            }
            index += 1;
        }
        None
    }

    #[kani::proof]
    fn a_string_body_changes_only_at_a_backslash_or_its_quote() {
        let quote = any_quote();
        let c: char = kani::any();
        let plain = StringBody::open(quote);
        let escaped = plain.step('\\');
        assert!(escaped.is_some() && escaped != Some(plain));

        // An escaped code point, whatever it is, is consumed.
        assert_eq!(escaped.and_then(|body| body.step(c)), Some(plain));

        // Outside an escape, only the ASCII bytes U+005C and the opening
        // quote change the state; every other scalar, including one whose
        // low byte equals a quote (U+0122, U+0127), stays in the string.
        let scalar = u32::from(c);
        let next = plain.step(c);
        if scalar == u32::from(quote) {
            assert_eq!(next, None);
        } else if scalar == 0x5c {
            assert_eq!(next, escaped);
        } else {
            assert_eq!(next, Some(plain));
        }
        kani::cover!(next.is_none(), "the opening quote ends the string");
        kani::cover!(scalar & 0xff == u32::from(quote) && scalar > 0x7f, "a quote low byte");
        kani::cover!(scalar > 0xffff && next == Some(plain), "a supplementary scalar");
    }

    #[kani::proof]
    #[kani::unwind(5)]
    fn a_string_ends_at_its_first_unescaped_quote() {
        let quote = any_quote();
        let body: [char; LEN] = kani::any();
        let end = stepped_end(&body, quote);
        assert_eq!(end, specified_end(&body, quote));
        kani::cover!(
            body[0] == '\\' && body[1] == '\\' && end == Some(2),
            "an escaped backslash before the quote"
        );
        kani::cover!(
            body[0] == '\\' && body[1] == quote && end == Some(2),
            "an escaped quote, then the closing quote"
        );
        kani::cover!(
            body[0] == '\u{122}' && quote == '"' && end.is_none(),
            "U+0122 inside a double-quoted string"
        );
        kani::cover!(end.is_none(), "an unterminated body");
    }
}
