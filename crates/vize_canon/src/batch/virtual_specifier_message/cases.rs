//! Whole-message controls for eligibility before authored-presence searches.

use vize_carton::{String, ToCompactString, cstr};

use super::{QUOTE_PAIRS, authored_specifier, quoted_specifiers, restore_virtual_vue_specifiers};

// Freeze the former operation order as a test oracle. In particular, retain
// the original candidate list instead of sorting it or rescanning replacements.
fn presence_first_reference(message: &str, source: &str) -> String {
    let mut rewrites = Vec::new();
    for reported in quoted_specifiers(message) {
        if source.contains(reported) {
            continue;
        }
        if let Some(authored) = authored_specifier(reported) {
            rewrites.push((reported, authored));
        }
    }
    let mut rewritten = message.to_compact_string();
    for (reported, authored) in rewrites {
        for (open, close) in QUOTE_PAIRS {
            let quoted = cstr!("{open}{reported}{close}");
            let restored = cstr!("{open}{authored}{close}");
            rewritten = rewritten.replace(quoted.as_str(), restored.as_str()).into();
        }
    }
    rewritten
}

#[test]
fn preserves_exact_authored_presence_and_quoted_replacement_boundaries() {
    for (message, source, expected) in [
        ("'./A.vue.ts'", "// ./A.vue.ts", "'./A.vue.ts'"),
        ("'./A.vue.ts'", "prefix./A.vue.tsx.trailing", "'./A.vue.ts'"),
        ("'./A.vue.ts'", "import './A.vue'", "'./A.vue'"),
        ("'./A.vue.tsx'", "", "'./A.vue'"),
        (
            "'./A.vue.ts/__vize_authored_vue_ts__'",
            "import './A.vue.ts'",
            "'./A.vue.ts'",
        ),
        (
            "'./A.vue.ts/__vize_authored_vue_ts__'",
            "// ./A.vue.ts/__vize_authored_vue_ts__",
            "'./A.vue.ts/__vize_authored_vue_ts__'",
        ),
        (
            "'./A.vue.tsx.__vize_authored_vue_ts_alias__'",
            "",
            "'./A.vue.tsx'",
        ),
        (
            "'./A.vue.ts/__vize_missing_vue_import__.js'",
            "",
            "'./A.vue'",
        ),
        ("'./a.vue.ts'", "// ./A.vue.ts; .\\a.vue.ts", "'./a.vue'"),
        (
            "'./cafe\u{301}.vue.ts'",
            "// ./café.vue.ts",
            "'./cafe\u{301}.vue'",
        ),
        (r"'./\u0041.vue.ts'", "// ./A.vue.ts", r"'./\u0041.vue'"),
        (r"'./a\'b.vue.ts'", "", r"'./a\'b.vue.ts'"),
        ("'./a\0b.vue.ts'", "", "'./a\0b.vue'"),
        ("'./実験🙂.vue.ts'", "", "'./実験🙂.vue'"),
        (
            "./A.vue.ts `./A.vue.ts` './A.vue.ts' './A.vue.ts' \"./A.vue.ts\" ‘./A.vue.ts’",
            "",
            "./A.vue.ts `./A.vue.ts` './A.vue' './A.vue' \"./A.vue\" ‘./A.vue’",
        ),
        (
            "Module '\"./A.vue.ts\"' failed.",
            "",
            "Module '\"./A.vue\"' failed.",
        ),
        (
            "'prose with spaces' ‘wide\u{3000}space’ 'unterminated",
            "",
            "'prose with spaces' ‘wide\u{3000}space’ 'unterminated",
        ),
    ] {
        assert_eq!(presence_first_reference(message, source), expected);
        assert_eq!(restore_virtual_vue_specifiers(message, source), expected);
    }
}

#[test]
fn retains_grouped_candidate_order_and_single_pass_chained_sentinels() {
    let message = "Module \"./A.vue.ts\" and './A.vue.ts/__vize_authored_vue_ts__' differ.";
    assert_eq!(
        quoted_specifiers(message),
        ["./A.vue.ts/__vize_authored_vue_ts__", "./A.vue.ts"]
    );
    assert_eq!(
        restore_virtual_vue_specifiers(message, ""),
        "Module \"./A.vue\" and './A.vue' differ."
    );
    let single = "'./A.vue.ts/__vize_authored_vue_ts__'";
    let first = restore_virtual_vue_specifiers(single, "");
    assert_eq!(first, "'./A.vue.ts'");
    assert_eq!(restore_virtual_vue_specifiers(&first, ""), "'./A.vue'");

    let specifiers = [
        "./A.vue.ts/__vize_authored_vue_ts__",
        "./A.vue.ts",
        "./B.vue.ts.__vize_authored_vue_ts_alias__",
    ];
    let permutations = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    for specifier_order in permutations {
        for quote_order in permutations {
            let mut message = String::new("Module ");
            for (index, (specifier_index, quote_index)) in
                specifier_order.into_iter().zip(quote_order).enumerate()
            {
                if index != 0 {
                    message.push_str(" and ");
                }
                let &(open, close) = QUOTE_PAIRS
                    .get(quote_index)
                    .expect("known quote permutation");
                message.push(open);
                message.push_str(
                    specifiers
                        .get(specifier_index)
                        .expect("known specifier permutation"),
                );
                message.push(close);
            }
            message.push_str(" differ.");
            let expected = presence_first_reference(&message, "");
            assert_eq!(restore_virtual_vue_specifiers(&message, ""), expected);
            assert_eq!(
                restore_virtual_vue_specifiers(&expected, ""),
                presence_first_reference(&expected, "")
            );
        }
    }
}

#[test]
fn preserves_entire_long_messages_and_source_tail_presence() {
    let prefix = "prefix🙂\r\n".repeat(4096);
    let message = [prefix.as_str(), "Cannot use './A.vue.ts'.\r\nEND違"].concat();
    let expected = [prefix.as_str(), "Cannot use './A.vue'.\r\nEND違"].concat();
    let source = ["x".repeat(1_048_576).as_str(), "// ./A.vue.ts"].concat();
    assert_eq!(restore_virtual_vue_specifiers(&message, ""), expected);
    assert_eq!(restore_virtual_vue_specifiers(&message, &source), message);
    let unrelated = "Type 'number' differs from 'string' and 'undefined'.";
    assert_eq!(
        restore_virtual_vue_specifiers(unrelated, &source),
        unrelated
    );
}
