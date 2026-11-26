//! Right-associative operators recurse in the parser without any bracket, so
//! one expression may chain only a bounded number of them.

use vize_l0::expression_guard::expression_is_safe_to_parse;

const ACCEPTED: usize = 256;

fn chain(link: &str, links: usize, tail: &str) -> std::string::String {
    let mut source = link.repeat(links);
    source.push_str(tail);
    source
}

#[test]
fn each_right_associative_operator_has_the_same_budget() {
    for (link, tail) in [
        ("a?b:", "c"),
        ("a=>", "a"),
        ("a=", "1"),
        ("a+=", "1"),
        ("a>>=", "1"),
        ("a**", "a"),
        ("a?.5:", "1"),
    ] {
        assert!(
            expression_is_safe_to_parse(&chain(link, ACCEPTED, tail)),
            "{link}"
        );
        assert!(
            !expression_is_safe_to_parse(&chain(link, ACCEPTED + 1, tail)),
            "{link}"
        );
    }
}

#[test]
fn the_budget_is_shared_across_operators_and_brackets() {
    let mixed = [
        chain("a?b:", ACCEPTED / 2, "("),
        chain("a=>", ACCEPTED / 2 + 1, "a)"),
    ]
    .concat();
    assert!(!expression_is_safe_to_parse(&mixed));
}

#[test]
fn operators_that_do_not_recurse_are_not_counted() {
    for (link, tail) in [
        ("a==b&&", "c"),
        ("a!==b||", "c"),
        ("a>=b&&", "c"),
        ("a<=b&&", "c"),
        ("a??", "b"),
        ("a?.", "b"),
        ("a*", "b"),
    ] {
        assert!(
            expression_is_safe_to_parse(&chain(link, 2 * ACCEPTED, tail)),
            "{link}"
        );
    }
    let quoted = ["'", &"?=>**=".repeat(ACCEPTED), "'"].concat();
    assert!(expression_is_safe_to_parse(&quoted));
}

#[test]
fn a_run_of_new_is_a_prefix_run() {
    assert!(expression_is_safe_to_parse(&chain("new ", 31, "X")));
    assert!(!expression_is_safe_to_parse(&chain("new ", 32, "X")));
}
