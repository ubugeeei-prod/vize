use vize_atelier_core::steps::expression::{
    MAX_EXPRESSION_NESTING_DEPTH, expression_exceeds_max_depth, expression_is_safe_to_parse,
    expression_nesting_depth, prefix_identifiers_in_expression, strip_typescript_from_expression,
};

#[test]
fn expression_guard_rejects_jsdoc_non_nullable_type_angle_chains() {
    // Minimized from the js_ts_expression stack-overflow reproducer (#3213):
    // in `a<!!a<!!a<...`, OXC's type-argument speculation parses every `!` as
    // a JSDoc non-nullable type and every following `<` as nested type
    // arguments, recursing once per token until the parser overflows the Rust
    // stack. `<` followed by `!` therefore joins the speculative type-angle
    // class, making the unclosed angle run count toward the depth budget.
    let rejected = "a<!!".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1);
    assert!(expression_nesting_depth(&rejected) > MAX_EXPRESSION_NESTING_DEPTH);
    assert!(expression_exceeds_max_depth(&rejected));
    assert!(!expression_is_safe_to_parse(&rejected));
    assert_eq!(
        prefix_identifiers_in_expression(&rejected).as_str(),
        rejected
    );
    assert_eq!(
        strip_typescript_from_expression(&rejected).as_str(),
        rejected
    );

    // Whitespace between `<` and `!` reaches the same speculation path.
    let spaced = "a< !!".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1);
    assert!(!expression_is_safe_to_parse(&spaced));

    // The fuzzer's stack-overflow shape: hundreds of interleaved `<!!` units.
    let overflow = "a<!!".repeat(500);
    assert!(!expression_is_safe_to_parse(&overflow));

    // Real comparisons against negated operands stay inside the depth budget:
    // each `&&` ends the candidate type chain, so the speculative tracking
    // never latches and no angle depth accumulates.
    let legitimate = "x < !a && y < !b && z < !!c";
    assert_eq!(expression_nesting_depth(legitimate), 0);
    assert!(expression_is_safe_to_parse(legitimate));
    let rewritten = prefix_identifiers_in_expression(legitimate);
    assert!(rewritten.contains("_ctx.x < !_ctx.a"), "{rewritten}");
}

#[test]
fn expression_guard_rejects_identifier_type_angle_chains() {
    // The js_ts_expression slow-unit reproducer (#3712) is dominated by
    // identifier-starting type references such as `froppy<s$my<s$mbod...`.
    // OXC recursively speculates that every `<Identifier` opens another type
    // argument list, so the same depth budget used for structural types must
    // bound this shape before parsing.
    let at_limit = format!(
        "root{}",
        "<TypeReference".repeat(MAX_EXPRESSION_NESTING_DEPTH)
    );
    assert_eq!(
        expression_nesting_depth(&at_limit),
        MAX_EXPRESSION_NESTING_DEPTH
    );
    assert!(expression_is_safe_to_parse(&at_limit));

    let rejected = format!(
        "root{}",
        "<TypeReference".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1)
    );
    assert_eq!(
        expression_nesting_depth(&rejected),
        MAX_EXPRESSION_NESTING_DEPTH + 1
    );
    assert!(expression_exceeds_max_depth(&rejected));
    assert!(!expression_is_safe_to_parse(&rejected));
    assert_eq!(
        prefix_identifiers_in_expression(&rejected).as_str(),
        rejected
    );
    assert_eq!(
        strip_typescript_from_expression(&rejected).as_str(),
        rejected
    );

    // `$` and `_` are valid identifier starts in TypeScript type references, a
    // `\uXXXX` escape lexes as one, and so does a non-ASCII identifier start.
    // Trivia between `<` and the identifier is skipped exactly as OXC skips it.
    for identifier in [
        "<$type",
        "<_type",
        "<\\u0041type",
        "<\u{65e5}\u{672c}",
        "< spaced",
        "</* t */typed",
    ] {
        let chain = format!(
            "root{}",
            identifier.repeat(MAX_EXPRESSION_NESTING_DEPTH + 1)
        );
        assert!(!expression_is_safe_to_parse(&chain), "{identifier}");
    }
}

#[test]
fn expression_guard_keeps_ordinary_identifier_comparisons_safe() {
    // The identifier arm is by far the broadest of the four speculative
    // classes — nearly every `<` in real code is followed by one — so the two
    // escapes that keep it off ordinary expressions are pinned here.

    // A closing `>` pays its angle back, so balanced generics never accumulate
    // however deeply they nest.
    let nested_generics = "useState<Map<string, Set<number>>>(initial)";
    assert_eq!(expression_nesting_depth(nested_generics), 3);
    assert!(expression_is_safe_to_parse(nested_generics));

    // An arrow's `=>` closes an angle too, which is why callback-heavy template
    // expressions stay flat: 40 of them peak at the one open call parenthesis
    // plus the one comparison angle live inside it.
    let callbacks = std::iter::repeat_n("items.filter(i => i.score < i.limit)", 40)
        .collect::<Vec<_>>()
        .join(" + ");
    assert_eq!(expression_nesting_depth(&callbacks), 2);
    assert!(expression_is_safe_to_parse(&callbacks));

    // Logical operators end the candidate type chain, so a flat boolean chain
    // of identifier comparisons past the limit never latches.
    let chain = std::iter::repeat_n("a < b", MAX_EXPRESSION_NESTING_DEPTH + 5)
        .collect::<Vec<_>>()
        .join(" && ");
    assert_eq!(expression_nesting_depth(&chain), 0);
    assert!(expression_is_safe_to_parse(&chain));

    // And the accepted expressions really are rewritten, rather than returned
    // unchanged the way a guard rejection would return them.
    assert_eq!(
        prefix_identifiers_in_expression("a < b && c < d").as_str(),
        "_ctx.a < _ctx.b && _ctx.c < _ctx.d"
    );
}

#[test]
fn expression_guard_resets_type_angle_speculation_after_logical_operators() {
    // A flat boolean chain of `< !` comparisons past the limit must stay
    // accepted: `&&`/`||`/`??` cannot appear inside a type-argument list, so
    // each ends the candidate type chain and the guard must not keep
    // accumulating `angle_depth` across the whole expression (#3213 follow-up).
    for separator in [" && ", " || ", " ?? "] {
        let chain = std::iter::repeat_n("a < !b", MAX_EXPRESSION_NESTING_DEPTH + 5)
            .collect::<Vec<_>>()
            .join(separator);
        assert_eq!(
            expression_nesting_depth(&chain),
            0,
            "separator {separator:?}"
        );
        assert!(
            !expression_exceeds_max_depth(&chain),
            "separator {separator:?}"
        );
        assert!(
            expression_is_safe_to_parse(&chain),
            "separator {separator:?}"
        );

        // End-to-end: the guard accepts the chain, so the parser path rewrites
        // every identifier reference and the negated comparison survives intact.
        let rewritten = prefix_identifiers_in_expression(&chain);
        assert!(
            rewritten.contains("_ctx.a < !_ctx.b"),
            "separator {separator:?}: {rewritten}"
        );
    }

    // Even two speculative opens per segment reset at the operator, so a long
    // chain of `a < !b < !c` comparisons never latches the guard.
    let paired = std::iter::repeat_n("a < !b < !c", MAX_EXPRESSION_NESTING_DEPTH + 5)
        .collect::<Vec<_>>()
        .join(" && ");
    assert!(!expression_exceeds_max_depth(&paired));
    assert!(expression_is_safe_to_parse(&paired));

    // End-to-end: the paired chain is accepted, so the parser path rewrites the
    // whole doubled comparison rather than returning the input unchanged.
    let paired_rewritten = prefix_identifiers_in_expression(&paired);
    assert!(
        paired_rewritten.contains("_ctx.a < !_ctx.b < !_ctx.c"),
        "{paired_rewritten}"
    );

    // The reset is scoped to logical/nullish operators: an unbroken speculative
    // run (no `&&`/`||`/`??`) still latches and is rejected. The parser path
    // therefore returns the input unchanged instead of rewriting identifiers.
    let unbroken = "a<!!".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1);
    assert!(!expression_is_safe_to_parse(&unbroken));
    assert_eq!(
        prefix_identifiers_in_expression(&unbroken).as_str(),
        unbroken
    );
}

#[test]
fn expression_guard_sees_type_angle_markers_behind_trivia() {
    // OXC skips comments and ECMAScript whitespace between `<` and the marker,
    // so the guard must too: otherwise a marker hidden behind trivia evades the
    // speculative-angle classification while OXC still recurses per token into
    // the stack-overflow path (#3213).
    let block_comment = "a</* t */!!".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1);
    assert!(expression_exceeds_max_depth(&block_comment));
    assert!(!expression_is_safe_to_parse(&block_comment));

    let line_comment = "a<//t\n!!".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1);
    assert!(expression_exceeds_max_depth(&line_comment));
    assert!(!expression_is_safe_to_parse(&line_comment));

    // NBSP (U+00A0), LS (U+2028), and PS (U+2029) are ECMAScript whitespace.
    for whitespace in ["\u{00a0}", "\u{2028}", "\u{2029}"] {
        let hidden = format!("a<{whitespace}!!").repeat(MAX_EXPRESSION_NESTING_DEPTH + 1);
        assert!(
            expression_exceeds_max_depth(&hidden),
            "whitespace {:?}",
            whitespace.escape_unicode()
        );
        assert!(
            !expression_is_safe_to_parse(&hidden),
            "whitespace {:?}",
            whitespace.escape_unicode()
        );
    }
}

#[test]
fn expression_guard_rejects_parenthesized_type_angle_runs() {
    // Minimized from the js_ts_expression slow-unit/timeout reproducers
    // (#3277, #3279, #3281): a 2.8KB `<`-dense input parsed for ~10 seconds,
    // and neutralizing only its two `<(` adjacencies dropped it to 3ms. OXC's
    // type-argument speculation enters a parenthesized type at `<(` and keeps
    // re-running the whole `<` cascade when a late token fails the outer
    // parse, so `<(` joins the speculative type-angle class: two occurrences
    // latch the tracking and the unclosed `<` run counts toward the budget.
    let reproducer = ["id<(xx) ".repeat(2), "sT<s<\\jjjjjjjjjj".repeat(16)].concat();
    assert!(expression_nesting_depth(&reproducer) > MAX_EXPRESSION_NESTING_DEPTH);
    assert!(expression_exceeds_max_depth(&reproducer));
    assert!(!expression_is_safe_to_parse(&reproducer));
    assert_eq!(
        prefix_identifiers_in_expression(&reproducer).as_str(),
        reproducer
    );
    assert_eq!(
        strip_typescript_from_expression(&reproducer).as_str(),
        reproducer
    );

    // Whitespace between `<` and `(` reaches the same speculation path.
    let spaced = ["id< (xx) ".repeat(2), "sT<s<\\jjjjjjjjjj".repeat(16)].concat();
    assert!(!expression_is_safe_to_parse(&spaced));
}

#[test]
fn expression_guard_keeps_ordinary_parenthesized_shapes_safe() {
    // A single generic call with a function-type argument opens one
    // speculative angle: tracking needs two, so real code stays accepted.
    let generic_call = "useHandler<(payload: MouseEvent) => void>(handler)";
    assert!(expression_is_safe_to_parse(generic_call));

    // Two latched `<(` opens whose angles close again stay within budget:
    // latching alone rejects nothing, only unclosed accumulation does.
    let closed = "f<(A)>(x) + g<(B)>(y)";
    assert!(expression_is_safe_to_parse(closed));

    // Comparisons against parenthesized operands reset at `&&`, so a flat
    // boolean chain past the limit never latches.
    let chain = std::iter::repeat_n("a < (b + 1)", MAX_EXPRESSION_NESTING_DEPTH + 5)
        .collect::<Vec<_>>()
        .join(" && ");
    assert_eq!(expression_nesting_depth(&chain), 1);
    assert!(expression_is_safe_to_parse(&chain));
    let rewritten = prefix_identifiers_in_expression(&chain);
    assert!(rewritten.contains("_ctx.a < (_ctx.b + 1)"), "{rewritten}");

    // A latched ternary chain accumulates only as deep as its real nesting.
    let ternary = "count < (limit) ? a : total < (max) ? b : c";
    assert!(expression_is_safe_to_parse(ternary));
}

#[test]
fn expression_guard_does_not_treat_relational_operators_as_type_angles() {
    let expression = std::iter::repeat_n("value < limit", MAX_EXPRESSION_NESTING_DEPTH + 1)
        .collect::<Vec<_>>()
        .join(" || ");

    assert_eq!(expression_nesting_depth(&expression), 0);
    assert!(!expression_exceeds_max_depth(&expression));
    let rewritten = prefix_identifiers_in_expression(&expression);
    assert!(rewritten.contains("_ctx.value < _ctx.limit"), "{rewritten}");
}
