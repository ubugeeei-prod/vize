use crate::support::fix_history_runner::check_pack;

#[test]
fn setup_dependent_ambient_captures_preserve_crlf_batch_diagnostics() {
    check_pack(
        "setup-ambient-captures-crlf-6232",
        "fix_history_cases::setup_ambient_captures_crlf_6232::setup_dependent_ambient_captures_preserve_crlf_batch_diagnostics",
        "0c94290f5f02b35a76f8ebc86128bcd1e1193e4a",
        Some(6232),
        &[
            "literal",
            "transitive",
            "overloads",
            "heritage",
            "expose",
            "export",
            "async",
            "adjacent",
            "adjacent-flow",
            "flow",
            "class-type",
            "local-type",
            "generic-signature",
            "hygiene",
            "escaped-hygiene",
            "invalid-heritage",
            "invalid-type-property",
            "invalid-literal-and-template",
            "flow-errors",
            "forward-type-query",
            "heritage-expression",
            "export-consumer",
        ],
    );
}
