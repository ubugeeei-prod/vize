//! Attribute scanning helpers for template semantic tokens.
//!
//! Locate directive names, dynamic arguments and attribute values inside a
//! template by byte offset. Every lookup goes through `str::get`, so offsets
//! that fall outside the template or inside a multi-byte character simply
//! produce no match.

pub(super) fn shorthand_name_end(
    template: &str,
    attr_start: usize,
    is_plain_name_char: impl Fn(char) -> bool,
    include_modifiers: bool,
) -> Option<usize> {
    let mut pos = attr_start + 1;
    if pos >= template.len() {
        return None;
    }

    if template.get(pos..)?.starts_with('[') {
        pos = find_matching_square_bracket(template, pos)? + 1;
        if include_modifiers {
            pos = consume_modifier_suffix(template, pos);
        }
        return Some(pos);
    }

    let name_start = pos;
    while pos < template.len() {
        let ch = template.get(pos..)?.chars().next()?;
        if !is_plain_name_char(ch) {
            break;
        }
        pos += ch.len_utf8();
    }

    if pos == name_start { None } else { Some(pos) }
}

fn consume_modifier_suffix(template: &str, mut pos: usize) -> usize {
    while template
        .get(pos..)
        .is_some_and(|rest| rest.starts_with('.'))
    {
        pos += 1;
        while pos < template.len() {
            let Some(ch) = template.get(pos..).and_then(|rest| rest.chars().next()) else {
                break;
            };
            if !ch.is_ascii_alphanumeric() && ch != '-' && ch != '_' {
                break;
            }
            pos += ch.len_utf8();
        }
    }
    pos
}

pub(super) fn dynamic_argument_value(template: &str, attr_start: usize) -> Option<(usize, usize)> {
    let name_end = attribute_name_end(template, attr_start);
    let search = template.get(attr_start..name_end)?;
    let bracket_offset = search.find('[')? + attr_start;
    let bracket_end = find_matching_square_bracket(template, bracket_offset)?;
    Some((bracket_offset + 1, bracket_end))
}

fn find_matching_square_bracket(template: &str, open_offset: usize) -> Option<usize> {
    let rest = template.get(open_offset..)?;
    if !rest.starts_with('[') {
        return None;
    }

    let mut depth = 0i32;
    let mut quote = None;
    let mut prev = '\0';

    for (relative, ch) in rest.char_indices() {
        if let Some(open_quote) = quote {
            if ch == open_quote && prev != '\\' {
                quote = None;
            }
            prev = ch;
            continue;
        }

        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(open_offset + relative);
                }
            }
            _ => {}
        }
        prev = ch;
    }

    None
}

pub(super) fn is_attribute_start(template: &str, offset: usize) -> bool {
    if offset >= template.len() {
        return false;
    }
    let Some(before) = template.get(..offset) else {
        return false;
    };

    let Some(prev) = before.chars().next_back() else {
        return false;
    };
    if !prev.is_ascii_whitespace() {
        return false;
    }

    let Some(tag_start) = before.rfind('<') else {
        return false;
    };
    // `<` is one byte, so the tag body always starts on a char boundary.
    let tag_body = before.get(tag_start + 1..).unwrap_or_default();

    let mut quote = None;
    for ch in tag_body.chars() {
        if let Some(open_quote) = quote {
            if ch == open_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => quote = Some(ch),
            '>' => return false,
            _ => {}
        }
    }

    if quote.is_some() {
        return false;
    }

    let tag_body = tag_body.trim_start();
    !tag_body.is_empty()
        && !tag_body.starts_with('/')
        && !tag_body.starts_with('!')
        && !tag_body.starts_with('?')
}

pub(super) fn is_attribute_name_boundary(template: &str, offset: usize) -> bool {
    template
        .get(offset..)
        .unwrap_or_default()
        .chars()
        .next()
        .is_none_or(|ch| {
            matches!(ch, '=' | ':' | '.' | '/' | '>' | '"' | '\'') || ch.is_ascii_whitespace()
        })
}

fn attribute_name_end(template: &str, start: usize) -> usize {
    let mut end = start;
    for (relative, ch) in template.get(start..).unwrap_or_default().char_indices() {
        if ch == '=' || ch == '/' || ch == '>' || ch.is_ascii_whitespace() {
            break;
        }
        end = start + relative + ch.len_utf8();
    }
    end
}

pub(super) fn attribute_value(template: &str, attr_start: usize) -> Option<(usize, usize)> {
    let mut pos = attribute_name_end(template, attr_start);

    while pos < template.len() {
        let ch = template.get(pos..)?.chars().next()?;
        if !ch.is_ascii_whitespace() {
            break;
        }
        pos += ch.len_utf8();
    }

    if template.get(pos..)?.chars().next()? != '=' {
        return None;
    }
    pos += 1;

    while pos < template.len() {
        let ch = template.get(pos..)?.chars().next()?;
        if !ch.is_ascii_whitespace() {
            break;
        }
        pos += ch.len_utf8();
    }

    let quote = template.as_bytes().get(pos).copied()?;
    if quote == b'"' || quote == b'\'' {
        let value_start = pos + 1;
        let quote_char = quote as char;
        let value_end = template.get(value_start..)?.find(quote_char)? + value_start;
        return Some((value_start, value_end));
    }

    let value_start = pos;
    while pos < template.len() {
        let rest = template.get(pos..)?;
        let ch = rest.chars().next()?;
        if ch.is_ascii_whitespace()
            || ch == '>'
            || (ch == '/'
                && rest
                    .get(ch.len_utf8()..)
                    .is_some_and(|after| after.starts_with('>')))
        {
            break;
        }
        pos += ch.len_utf8();
    }

    if pos == value_start {
        None
    } else {
        Some((value_start, pos))
    }
}
