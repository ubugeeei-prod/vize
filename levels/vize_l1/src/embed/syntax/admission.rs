//! Conservative initial admission, independent of unfinished full grammar
//! recursion accounting. Do not mistake the shared expression guard for a
//! statement-recursion bound: OXC also recurses on unbraced if/loop bodies,
//! labels, new, class heritage, assignments, arrows and TS type operators.

/// Maximum conservative word/punctuation units in the complete parser input,
/// including generated delimiters, strings and comments. Larger valid snippets
/// remain unadmitted until complete JS/TS recursion admission is implemented.
pub const NATIVE_SYNTAX_UNIT_LIMIT: usize = 31;

/// Every ASCII identifier/numeric run counts once; each other non-whitespace
/// byte counts once. No quoted/comment/regex text is skipped, and non-ASCII
/// UTF-8 bytes overcount rather than hide recursive syntax. Valid recursive
/// grammar requires separate words or punctuation; unbroken runs are a single
/// identifier/numeric token or invalid adjacency, not recursive operators.
pub(super) fn allows_small_input(input: &str) -> bool {
    let mut units = 0;
    let mut word = false;
    for byte in input.bytes() {
        let next_word = byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$');
        if (next_word && !word) || (!next_word && !byte.is_ascii_whitespace()) {
            units += 1;
            if units > NATIVE_SYNTAX_UNIT_LIMIT {
                return false;
            }
        }
        word = next_word;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::{NATIVE_SYNTAX_UNIT_LIMIT, allows_small_input};

    #[test]
    fn admits_small_real_programs_and_counts_wrapper() {
        assert!(allows_small_input(
            "/* keep */ const value: number = 1; // tail"
        ));
        assert!(allows_small_input("(\n/* lead */ (count) // tail\n)"));
        let at_limit = "+".repeat(NATIVE_SYNTAX_UNIT_LIMIT);
        assert!(allows_small_input(&at_limit));
        assert!(!allows_small_input(&vize_l0::cstr!("({at_limit})")));
    }

    #[test]
    fn rejects_statement_and_expression_recursion_even_without_deep_brackets() {
        for repeated in [
            "if(x) ",
            "while(x) ",
            "label:",
            "new ",
            "class extends ",
            "x=",
            "x=>",
            "x?x:",
            "keyof ",
            "readonly ",
        ] {
            assert!(!allows_small_input(&repeated.repeat(32)), "{repeated}");
        }
    }

    #[test]
    fn never_skips_recursion_candidates_in_comments_strings_or_non_ascii() {
        assert!(!allows_small_input(&vize_l0::cstr!(
            "/* {} */",
            "if ".repeat(32)
        )));
        assert!(!allows_small_input(&vize_l0::cstr!(
            "'{}'",
            "new ".repeat(32)
        )));
        assert!(!allows_small_input(&"α".repeat(16)));
    }

    #[test]
    fn contiguous_ascii_runs_and_ascii_whitespace_do_not_hide_operators() {
        assert!(allows_small_input("_identifier123$"));
        assert!(allows_small_input("identifier\tidentifier\r\nidentifier"));
        assert!(!allows_small_input(&"name = ".repeat(32)));
        assert!(!allows_small_input(&"\\u006e\\u0065\\u0077 ".repeat(32)));
    }
}
