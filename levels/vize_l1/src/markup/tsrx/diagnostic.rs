//! TSRX syntax and template-rule errors, with the specification's codes
//! (`TSRX1xxx` markup syntax, `TSRX2xxx` template rules) and the pinned
//! reference compiler's wording.

use oxc_diagnostics::OxcDiagnostic;
use vize_l0::Span;

fn tsrx(
    code: &'static str,
    message: impl Into<alloc::borrow::Cow<'static, str>>,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::error(message)
        .with_error_code("TSRX", code)
        .with_label(oxc_span::Span::new(span.start, span.end))
}

pub(super) fn unclosed_tag(name: &str, at: Span) -> OxcDiagnostic {
    tsrx(
        "1001",
        [
            "Unclosed tag '<",
            name,
            ">'. Expected '</",
            name,
            ">' before end of template.",
        ]
        .concat(),
        at,
    )
}

pub(super) fn mismatched_closing_tag(expected: &str, found: &str, at: Span) -> OxcDiagnostic {
    tsrx(
        "1002",
        [
            "Expected closing tag to match opening tag. Expected '</",
            expected,
            ">' but found '</",
            found,
            ">'",
        ]
        .concat(),
        at,
    )
}

pub(super) fn unexpected_closing_tag(at: Span) -> OxcDiagnostic {
    tsrx("1003", "Unexpected closing tag", at)
}

/// `written` is the end tag as written, such as `</SCRIPT`.
pub(super) fn script_end_tag_in_body(written: &str, at: Span) -> OxcDiagnostic {
    let rest = written.get(2..).unwrap_or_default();
    let message = [
        "'",
        written,
        "' can end a script in HTML, so a '<script>' body can't contain it. Write '<\\/",
        rest,
        "' instead.",
    ]
    .concat();
    tsrx("1004", message, at)
}

pub(super) fn trailing_semicolon(at: Span) -> OxcDiagnostic {
    tsrx(
        "1007",
        "TSRX expression containers do not use semicolons. Remove this semicolon.",
        at,
    )
}

pub(super) fn namespaced_element(name: &str, at: Span) -> OxcDiagnostic {
    let message = [
        "Namespaced elements are not supported in TSRX templates: <",
        name,
        ">.",
    ]
    .concat();
    tsrx("1005", message, at)
}

pub(super) fn attribute_value_spread(at: Span) -> OxcDiagnostic {
    tsrx(
        "1006",
        "Attribute values cannot be spread. Use a spread attribute (`{...props}`) instead.",
        at,
    )
}

/// A directive branch written without its `@`, such as `else` after an `@if` block.
pub(super) fn directive_branch_expected(branch: &str, directive: &str, at: Span) -> OxcDiagnostic {
    let message = ["Expected `@", branch, "` after `@", directive, "` block."].concat();
    tsrx("1009", message, at)
}

pub(super) fn directive_body_expected(at: Span) -> OxcDiagnostic {
    tsrx("1008", "Expected `{` after JSX control-flow directive.", at)
}

pub(super) fn try_handler_missing(at: Span) -> OxcDiagnostic {
    tsrx(
        "1010",
        "Missing `@catch` or `@pending` after `@try` block.",
        at,
    )
}

pub(super) fn for_clause(at: Span) -> OxcDiagnostic {
    tsrx(
        "1011",
        "\"index\" must come before \"key\" in for-of loop",
        at,
    )
}

/// Direct exits that are invalid in a template body, by body kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Exit {
    Return,
    Break,
    Continue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BodyKind {
    /// A statement container that is not a function body.
    CodeBlock,
    If,
    For,
    SwitchCase,
    /// `@try`, `@pending` and `@catch` bodies.
    Try,
}

pub(super) fn invalid_exit(body: BodyKind, exit: Exit, at: Span) -> OxcDiagnostic {
    let (code, message) = match (body, exit) {
        (BodyKind::If, Exit::Return) => (
            "2002",
            "Return statements are not allowed inside TSRX template @if blocks. Move the return before the template output or render conditionally instead.",
        ),
        (BodyKind::If, Exit::Break) => (
            "2003",
            "Break statements are not allowed inside TSRX template @if blocks.",
        ),
        (BodyKind::If, Exit::Continue) => (
            "2004",
            "Continue statements are not allowed inside TSRX template @if blocks. Filter before rendering or use conditional output instead.",
        ),
        (BodyKind::For, Exit::Return) => (
            "2005",
            "Return statements are not allowed inside TSRX template for...of loops. Filter the iterable before rendering or use an @empty fallback for empty lists.",
        ),
        (BodyKind::For, Exit::Break) => (
            "2006",
            "Break statements are not allowed inside TSRX template for...of loops.",
        ),
        (BodyKind::For, Exit::Continue) => (
            "2007",
            "Continue statements are not allowed inside TSRX template for...of loops. Filter the iterable before rendering.",
        ),
        (BodyKind::SwitchCase, Exit::Break | Exit::Continue) => {
            ("2008", "`break` is invalid inside `@switch` cases.")
        }
        (BodyKind::SwitchCase, Exit::Return) => {
            ("2009", "`return` is invalid inside `@switch` cases.")
        }
        (BodyKind::CodeBlock | BodyKind::Try, _) => (
            "2001",
            "Return statements are not allowed inside TSRX templates. Move the return before the TSRX return value, or use conditional rendering instead.",
        ),
    };
    tsrx(code, message, at)
}

pub(super) fn single_output(at: Span) -> OxcDiagnostic {
    tsrx(
        "2011",
        "A code block renders a single node; wrap multiple nodes or text in a fragment '<>…</>'.",
        at,
    )
}

pub(super) fn statement_after_output(at: Span) -> OxcDiagnostic {
    tsrx(
        "2012",
        "Code must be at the top of '@{ }'; statements cannot follow the rendered output.",
        at,
    )
}

pub(super) fn spread_child(at: Span) -> OxcDiagnostic {
    tsrx(
        "2013",
        "JSX spread children (`{...items}`) are not supported. Render the array as an expression child instead: `{items}`.",
        at,
    )
}

pub(super) fn dynamic_tag_expression(at: Span) -> OxcDiagnostic {
    tsrx(
        "2014",
        "A dynamic tag expression must be an identifier, a member access such as `props.as` or `registry[name]`, or a string literal. Compute anything else before the element: `const Tag = c ? Child : Fallback;`, then `<{Tag} />`.",
        at,
    )
}

pub(super) fn for_of_only(at: Span) -> OxcDiagnostic {
    tsrx(
        "2015",
        "TSRX `@for` currently supports `for...of` loops in template output.",
        at,
    )
}

pub(super) fn try_finally(at: Span) -> OxcDiagnostic {
    tsrx("2018", "'@try' does not support a '@finally' block.", at)
}

/// A syntax error the host grammar did not already report.
pub(super) fn unexpected(at: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Unexpected token")
        .with_error_code("TS", "1012")
        .with_label(oxc_span::Span::new(at.start, at.end))
}

pub(super) fn expected(token: &'static str, at: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(["'", token, "' expected."].concat())
        .with_error_code("TS", "1005")
        .with_label(oxc_span::Span::new(at.start, at.end))
}
