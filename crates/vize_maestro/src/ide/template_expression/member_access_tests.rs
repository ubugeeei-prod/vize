use super::is_at_member_access_position;

/// Offset of the caret marker `|`, with the marker removed.
fn at_caret(marked: &str) -> (String, usize) {
    let offset = marked.find('|').expect("test input needs a `|` caret");
    (marked.replace('|', ""), offset)
}

fn is_member_access(marked: &str) -> bool {
    let (content, offset) = at_caret(marked);
    is_at_member_access_position(&content, offset)
}

#[test]
fn member_access_covers_dot_optional_and_non_null_chains() {
    assert!(is_member_access("{{ it.| }}"));
    assert!(is_member_access("{{ it.na| }}"));
    assert!(is_member_access("{{ theme.notFound?.co| }}"));
    assert!(is_member_access("{{ user!.na| }}"));
    assert!(is_member_access("{{ a.b.c.| }}"));
    assert!(is_member_access("{{ items[0].| }}"));
    assert!(is_member_access("{{ f().| }}"));
    assert!(is_member_access("{{ it.$pr| }}"));
    assert!(is_member_access("{{ it._pr| }}"));
}

#[test]
fn member_names_may_be_unicode_identifiers() {
    assert!(is_member_access("{{ it.名| }}"));
    assert!(is_member_access("{{ it.名前| }}"));
    assert!(is_member_access("{{ 名.前| }}"));
    // A digit inside the receiver keeps it an identifier, not a literal.
    assert!(is_member_access("{{ foo1.ba| }}"));
    // Decomposed text: the caret sits on a combining mark, which is an
    // identifier part rather than a token boundary.
    assert!(is_member_access("{{ it.cafe\u{301}| }}"));
    assert!(is_member_access("{{ cafe\u{301}.na| }}"));
    // Zero-width joiners are identifier parts too.
    assert!(is_member_access("{{ it.a\u{200D}b| }}"));
}

#[test]
fn identifier_positions_are_not_member_access() {
    assert!(!is_member_access("{{ | }}"));
    assert!(!is_member_access("{{ cou| }}"));
    assert!(!is_member_access("{{ val| }}"));
    assert!(!is_member_access("{{ it.name + val| }}"));
    assert!(!is_member_access("{{ f(arg| ) }}"));
}

#[test]
fn spreads_and_numeric_literals_are_not_member_access() {
    assert!(!is_member_access("{{ f(...arg| ) }}"));
    assert!(!is_member_access("{{ 1.5| }}"));
    // Every fragment of a numeric literal, not just the one with a digit
    // under the caret.
    assert!(!is_member_access("{{ 1.| }}"));
    assert!(!is_member_access("{{ 1.na| }}"));
    assert!(!is_member_access("{{ 42.toStrin| }}"));
    // Separators do not close the integer part either.
    assert!(!is_member_access("{{ 1_000.toStrin| }}"));
}

#[test]
fn numbers_still_expose_their_members() {
    // Both spellings that let a `.` follow an integer literal read a member
    // off the number: the second dot of `42..x`, and a dot separated from
    // the literal by whitespace.
    assert!(is_member_access("{{ 42..toStrin| }}"));
    assert!(is_member_access("{{ 42..| }}"));
    assert!(is_member_access("{{ 42 .toStrin| }}"));
    assert!(is_member_access("{{ 1.5.toFixe| }}"));
}

#[test]
fn literals_that_cannot_take_a_decimal_point_read_members() {
    // An exponent ends the literal, so the `.` after it is a member read
    // even though the digits next to it look like an integer part.
    assert!(is_member_access("{{ 1e3.toFixe| }}"));
    assert!(is_member_access("{{ 1e-3.toFixe| }}"));
    assert!(is_member_access("{{ 1E+3.toFixe| }}"));
    assert!(is_member_access("{{ 1.5e-3.toFixe| }}"));
    // A leading-dot decimal already spent its decimal point.
    assert!(is_member_access("{{ .5.toFixe| }}"));
    // Non-decimal radices have no decimal point to spend.
    assert!(is_member_access("{{ 0xFF.toStrin| }}"));
    assert!(is_member_access("{{ 0b11.toStrin| }}"));
    assert!(is_member_access("{{ 0o17.toStrin| }}"));
    // Neither does a BigInt.
    assert!(is_member_access("{{ 1n.toStrin| }}"));
    // The `e` still has to belong to a number: `abcde - 3.foo` is an
    // identifier minus a numeric literal, not an exponent.
    assert!(!is_member_access("{{ abcde-3.toFixe| }}"));
}
