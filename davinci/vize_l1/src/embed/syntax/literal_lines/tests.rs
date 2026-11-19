use alloc::vec;

use super::{LiteralLineOptions, literal_line_starts};

const DIRECTIVE: LiteralLineOptions = LiteralLineOptions {
    newline: b'\n',
    whole_templates: true,
    statements: true,
};

const INTERPOLATION: LiteralLineOptions = LiteralLineOptions {
    newline: b'\n',
    whole_templates: false,
    statements: false,
};

#[test]
fn single_line_values_have_no_answer() {
    assert_eq!(literal_line_starts("a + `b`", DIRECTIVE), None);
}

#[test]
fn quasi_and_continued_string_lines_are_owned() {
    assert_eq!(
        literal_line_starts("a +\n`x\ny`", DIRECTIVE),
        Some(vec![false, false, true])
    );
    assert_eq!(
        literal_line_starts("'a\\\nb' +\nc", DIRECTIVE),
        Some(vec![false, true, false])
    );
}

#[test]
fn backticks_in_strings_and_comments_are_not_literals() {
    assert_eq!(
        literal_line_starts("a // `\n+ '`'\n+ b", DIRECTIVE),
        Some(vec![false, false, false])
    );
}

#[test]
fn substitutions_belong_to_whole_templates_only() {
    let source = "`a${\nb\n}c`";
    assert_eq!(
        literal_line_starts(source, DIRECTIVE),
        Some(vec![false, true, true])
    );
    assert_eq!(
        literal_line_starts(source, INTERPOLATION),
        Some(vec![false, false, false])
    );
}

#[test]
fn statements_are_read_only_when_allowed() {
    let source = "if (a) b()\nc(`x\ny`)";
    assert_eq!(
        literal_line_starts(source, DIRECTIVE),
        Some(vec![false, false, true])
    );
    assert_eq!(literal_line_starts(source, INTERPOLATION), None);
    assert_eq!(literal_line_starts("a +\n", DIRECTIVE), None);
}

#[test]
fn literals_after_a_leading_expression_are_owned() {
    // `track()` alone is an expression; the value is two statements.
    let source = "track();\nnotify(`x\ny`)";
    assert_eq!(
        literal_line_starts(source, DIRECTIVE),
        Some(vec![false, false, true])
    );
    assert_eq!(literal_line_starts(source, INTERPOLATION), None);
    // A trailing comment keeps a one-expression interpolation.
    assert_eq!(
        literal_line_starts("`x\ny` // tail\n", INTERPOLATION),
        Some(vec![false, true, false])
    );
}

#[test]
fn a_lone_cr_ends_lines_when_selected() {
    let options = LiteralLineOptions {
        newline: b'\r',
        ..DIRECTIVE
    };
    assert_eq!(
        literal_line_starts("a +\r`x\ry`", options),
        Some(vec![false, false, true])
    );
}
