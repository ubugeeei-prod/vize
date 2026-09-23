mod dynamic_argument;

/// Check if a cursor offset is inside a Vue template expression.
///
/// This covers interpolations, directive values and dynamic arguments, but
/// deliberately excludes plain text nodes and static attribute values.
pub(crate) fn is_in_vue_template_expression(content: &str, offset: usize) -> bool {
    if content.is_empty() {
        return false;
    }

    let mut offset = offset.min(content.len());
    while offset > 0 && !content.is_char_boundary(offset) {
        offset -= 1;
    }
    if crate::ide::completion::is_inside_html_comment(content, offset) {
        return false;
    }

    is_in_mustache_expression(content, offset) || is_in_vue_directive_expression(content, offset)
}

/// Check if a cursor offset completes a *member* of the expression to its left,
/// i.e. the partial name under the caret is introduced by `.`, `?.`, or `!.`.
///
/// Member access is the one template position whose answer only the type checker
/// knows: the structural binding list cannot enumerate the properties of an
/// arbitrary type (#3911). Identifier positions keep the structural answer, so
/// this predicate is what separates the two routes.
///
/// Deliberately not `CursorContext::MemberAccess`, which only recognizes a caret
/// sitting directly on the `.`. A completion request arrives with the member
/// name partially typed (`it.na|`, `theme.notFound?.co|`), and its receiver may
/// end in `?`/`!`, both of which that detector reports as an identifier. Widening
/// it would change what hover, definition, and references see at the same
/// position, so the routing question is answered here instead.
#[cfg(feature = "native")]
pub(crate) fn is_at_member_access_position(content: &str, offset: usize) -> bool {
    if content.is_empty() {
        return false;
    }

    let mut offset = offset.min(content.len());
    while offset > 0 && !content.is_char_boundary(offset) {
        offset -= 1;
    }

    // Step back over the partial member name already typed. Member names are
    // identifiers, so this walks characters rather than ASCII bytes: `it.名|`
    // reads a member exactly like `it.na|` does.
    let mut pos = identifier_start(content, offset);
    // Skip any whitespace between the `.` and the name (`it. na|`).
    while let Some(ch) = content.get(..pos).and_then(|head| head.chars().next_back()) {
        if !ch.is_whitespace() {
            break;
        }
        pos -= ch.len_utf8();
    }
    if !content.get(..pos).is_some_and(|head| head.ends_with('.')) {
        return false;
    }

    let dot = pos - 1;
    // `...rest` spreads an expression rather than reading a member off it. A
    // spread is always three dots, so this looks for the other two rather than
    // for any preceding dot: in `42..toStrin` the first dot closes the numeric
    // literal `42.` and the second one still reads a member off it.
    if content.get(..dot).is_some_and(|head| head.ends_with("..")) {
        return false;
    }
    !is_decimal_point(content, dot)
}

/// Whether the `.` at `dot` is the decimal point of a numeric literal rather
/// than a member-access operator.
///
/// Only a literal that can still take a decimal point owns the dot, and that is
/// a bare run of decimal digits (`1.`, `1_000.`). Every other numeric spelling
/// has already ended by the time the `.` arrives, so the dot reads a member off
/// the finished number: a decimal point the literal already spent
/// (`1.5.toFixed`, `.5.toFixed`), an exponent (`1e3.toFixed`, `1e-3.toFixed`), a
/// radix prefix (`0xFF.toString`), or the BigInt suffix (`1n.toString`).
#[cfg(feature = "native")]
fn is_decimal_point(content: &str, dot: usize) -> bool {
    let start = identifier_start(content, dot);
    let Some(token) = content.get(start..dot) else {
        return false;
    };
    // A digit-led token before the `.` is the integer part of a literal, which
    // covers every fragment of one (`1.`, `1.5`, `1.na`). An identifier that
    // merely contains digits (`foo1.bar`) is still a member read, so this tests
    // the start of the token, not the character next to the dot.
    if !token.starts_with(|ch: char| ch.is_ascii_digit()) {
        return false;
    }
    // Anything other than decimal digits inside that token closed the literal
    // before the dot: a radix prefix (`0xFF`), an exponent indicator (`1e3`), or
    // the BigInt suffix (`1n`). None of those can take a decimal point.
    if !token
        .bytes()
        .all(|byte| byte.is_ascii_digit() || byte == b'_')
    {
        return false;
    }
    // Those digits can still be the tail of a literal that ended before them,
    // in which case the `.` reads a member too.
    content
        .get(..start)
        .is_some_and(|before| !closes_numeric_literal(before))
}

/// Whether the text ending right before a run of decimal digits already closed
/// the numeric literal those digits belong to.
#[cfg(feature = "native")]
fn closes_numeric_literal(before: &str) -> bool {
    // A decimal point the literal already spent (`1.5.`, `.5.`): the scan back
    // over identifier characters stops at that dot, so the digits behind the
    // caret's dot are only the fraction.
    if before.ends_with('.') {
        return true;
    }
    // A signed exponent (`1e-3.`, `1E+3.`): the sign is not an identifier
    // character, so those digits are only the exponent. The `e` has to belong to
    // a number rather than to an identifier, so `abcde-3.` stays a decimal point.
    let Some(mantissa) = before
        .strip_suffix(|ch: char| ch == '+' || ch == '-')
        .and_then(|signed| signed.strip_suffix(|ch: char| ch == 'e' || ch == 'E'))
    else {
        return false;
    };
    mantissa
        .get(identifier_start(mantissa, mantissa.len())..)
        .is_some_and(|token| token.starts_with(|ch: char| ch.is_ascii_digit()))
}

/// Walk back from `end` over identifier characters and return the token start.
#[cfg(feature = "native")]
fn identifier_start(content: &str, end: usize) -> usize {
    let mut start = end;
    while let Some(ch) = content
        .get(..start)
        .and_then(|head| head.chars().next_back())
    {
        if !is_identifier_char(ch) {
            break;
        }
        start -= ch.len_utf8();
    }
    start
}

/// ECMAScript `IdentifierPart`, so the scan keeps every character an editor can
/// have already typed into a member name: `is_alphanumeric` alone drops the
/// combining marks of decomposed text (`café` as `cafe` + U+0301) and the
/// zero-width joiners, both of which end the token one character too early.
#[cfg(feature = "native")]
fn is_identifier_char(ch: char) -> bool {
    oxc_syntax::identifier::is_identifier_part(ch)
}

fn is_in_mustache_expression(content: &str, offset: usize) -> bool {
    let Some(before) = content.get(..offset) else {
        return false;
    };
    let Some(mustache_start) = before.rfind("{{") else {
        return false;
    };

    let closed_before_cursor = before
        .rfind("}}")
        .is_some_and(|mustache_end| mustache_end > mustache_start);
    if closed_before_cursor {
        return false;
    }

    content
        .get(offset..)
        .is_some_and(|rest| rest.contains("}}"))
}

fn is_in_vue_directive_expression(content: &str, offset: usize) -> bool {
    let bytes = content.as_bytes();
    let Some(before) = content.get(..offset) else {
        return false;
    };
    for (tag_start, _) in before.match_indices('<').rev() {
        let name_start = tag_start + 1;
        if matches!(bytes.get(name_start), Some(b'/' | b'!' | b'?')) {
            continue;
        }
        let mut name_end = name_start;
        while bytes
            .get(name_end)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            name_end += 1;
        }
        if name_start == name_end {
            continue;
        }

        let mut quote = None;
        let mut quote_start = None;
        let mut pos = name_end;
        while pos < offset {
            let Some(&byte) = bytes.get(pos) else {
                break;
            };
            if let Some(open_quote) = quote {
                if byte == open_quote {
                    quote = None;
                    quote_start = None;
                }
            } else if byte == b'[' && dynamic_argument::contains_cursor(content, pos, offset) {
                return true;
            } else if byte == b'"' || byte == b'\'' {
                quote = Some(byte);
                quote_start = Some(pos);
            } else if byte == b'>' {
                break;
            }
            pos += 1;
        }

        if pos < offset || quote.is_none() {
            continue;
        }
        let Some(quote_start) = quote_start else {
            continue;
        };
        return directive_attribute_name_before_quote(content, quote_start)
            .is_some_and(is_vue_expression_attribute);
    }

    false
}

fn directive_attribute_name_before_quote(content: &str, quote_start: usize) -> Option<&str> {
    let bytes = content.as_bytes();
    // The byte right before `pos`, if any.
    let before = |pos: usize| {
        pos.checked_sub(1)
            .and_then(|index| bytes.get(index))
            .copied()
    };
    let mut pos = quote_start;
    while before(pos).is_some_and(|byte| byte.is_ascii_whitespace()) {
        pos -= 1;
    }
    if before(pos) != Some(b'=') {
        return None;
    }
    pos -= 1;

    while before(pos).is_some_and(|byte| byte.is_ascii_whitespace()) {
        pos -= 1;
    }
    let attr_end = pos;
    while let Some(byte) = before(pos) {
        if byte.is_ascii_whitespace() || matches!(byte, b'<' | b'>' | b'/') {
            break;
        }
        pos -= 1;
    }

    content.get(pos..attr_end)
}

fn is_vue_expression_attribute(attr_name: &str) -> bool {
    attr_name.starts_with(':')
        || attr_name.starts_with('@')
        || attr_name.starts_with('#')
        || attr_name.starts_with("v-")
}

#[cfg(all(test, feature = "native"))]
mod member_access_tests;
