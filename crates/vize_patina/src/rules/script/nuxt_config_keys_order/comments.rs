use oxc_ast::ast::ObjectExpression;
use oxc_span::GetSpan;
use vize_l0::{String, ToCompactString};

use super::support::{legacy_property_text_ranges, next_token_offset};

pub(super) enum PropertyText {
    Fixable {
        start: usize,
        end: usize,
        pieces: Vec<String>,
        separators: Vec<String>,
    },
    Unfixable,
}

pub(super) fn property_text_ranges(
    object: &ObjectExpression<'_>,
    source: &str,
) -> Option<PropertyText> {
    let close_brace = (object.span.end as usize).checked_sub(1)?;
    if gaps_contain_comment(object, source, close_brace) {
        return comment_aware_ranges(object, source, close_brace);
    }
    legacy_property_text_ranges(object, source).map(|(start, end, pieces)| {
        let separators = vec![String::new(""); pieces.len().saturating_sub(1)];
        PropertyText::Fixable {
            start,
            end,
            pieces,
            separators,
        }
    })
}

fn gaps_contain_comment(object: &ObjectExpression<'_>, source: &str, close_brace: usize) -> bool {
    let mut cursor = object.span.start as usize + 1;
    for property in &object.properties {
        let start = property.span().start as usize;
        let end = property.span().end as usize;
        if start < cursor || end > close_brace {
            return false;
        }
        if region_has_comment(source, cursor, start) {
            return true;
        }
        cursor = end;
    }
    region_has_comment(source, cursor, close_brace)
}

fn region_has_comment(source: &str, start: usize, end: usize) -> bool {
    source
        .get(start..end)
        .is_some_and(|text| text.contains("//") || text.contains("/*"))
}

struct Owned {
    start: usize,
    prop_end: usize,
    end: usize,
    has_comma: bool,
}

fn comment_aware_ranges(
    object: &ObjectExpression<'_>,
    source: &str,
    close_brace: usize,
) -> Option<PropertyText> {
    let brace = object.span.start as usize;
    if brace >= close_brace {
        return None;
    }
    let mut owned = Vec::with_capacity(object.properties.len());
    let mut cursor = brace + 1;
    for property in &object.properties {
        let prop_start = property.span().start as usize;
        let prop_end = property.span().end as usize;
        if prop_start < cursor || prop_end > close_brace || prop_end < prop_start {
            return None;
        }
        let start = leading_owned_start(source, cursor, prop_start);
        if start < cursor || start > prop_start {
            return None;
        }
        owned.push(Owned {
            start,
            prop_end,
            end: prop_end,
            has_comma: false,
        });
        cursor = prop_end;
    }

    for index in 0..owned.len() {
        let limit = owned.get(index + 1).map_or(close_brace, |next| next.start);
        let (end, has_comma) = trailing_owned_end(source, owned[index].prop_end, limit);
        if end < owned[index].prop_end || end > limit {
            return None;
        }
        owned[index].end = end;
        owned[index].has_comma = has_comma;
    }

    let prefix = source.get(brace + 1..owned[0].start)?;
    if !outside_prefix_is_safe(prefix) {
        return Some(PropertyText::Unfixable);
    }
    let suffix = source.get(owned.last()?.end..close_brace)?;
    if !is_whitespace_only(suffix) {
        return Some(PropertyText::Unfixable);
    }

    let mut separators = Vec::with_capacity(owned.len().saturating_sub(1));
    for pair in owned.windows(2) {
        if pair[1].start < pair[0].end {
            return None;
        }
        let gap = source.get(pair[0].end..pair[1].start)?;
        if !is_whitespace_only(gap) {
            return Some(PropertyText::Unfixable);
        }
        separators.push(gap.to_compact_string());
    }

    let mut pieces = Vec::with_capacity(owned.len());
    for (index, property_owned) in owned.iter().enumerate() {
        let insert_comma = index + 1 == owned.len()
            && !property_owned.has_comma
            && next_token_offset(source, property_owned.prop_end, close_brace) == close_brace;
        pieces.push(render_piece(source, property_owned, insert_comma)?);
    }
    Some(PropertyText::Fixable {
        start: owned[0].start,
        end: owned.last()?.end,
        pieces,
        separators,
    })
}

fn render_piece(source: &str, owned: &Owned, insert_comma: bool) -> Option<String> {
    let mut text = source.get(owned.start..owned.prop_end)?.to_compact_string();
    if insert_comma {
        text.push(',');
    }
    text.push_str(source.get(owned.prop_end..owned.end)?);
    Some(text)
}

fn leading_owned_start(source: &str, floor: usize, prop_start: usize) -> usize {
    let line = line_start(source, prop_start);
    if line < floor {
        return trim_hspace_start(source, floor, prop_start);
    }
    if source
        .get(line..prop_start)
        .is_some_and(|head| head.bytes().any(|byte| !matches!(byte, b' ' | b'\t')))
    {
        return trim_hspace_start(source, line, prop_start);
    }

    let mut region = line;
    loop {
        if region <= floor {
            break;
        }
        let Some(prev_nl) = region.checked_sub(1) else {
            break;
        };
        if source.as_bytes().get(prev_nl) != Some(&b'\n') {
            break;
        }
        let prev = line_start(source, prev_nl);
        if prev < floor {
            break;
        }
        let Some(prev_line) = source.get(prev..prev_nl) else {
            break;
        };
        if prev_line.trim().is_empty() {
            break;
        }
        region = prev;
    }
    if region == line {
        return line;
    }
    let mut candidate = region;
    while candidate < line {
        if source.get(candidate..line).is_some_and(is_comment_trivia) {
            return candidate;
        }
        let Some(tail) = source.get(candidate..line) else {
            return line;
        };
        let Some(nl) = tail.find('\n') else {
            return line;
        };
        let next = candidate + nl + 1;
        if next >= line {
            return line;
        }
        candidate = next;
    }
    line
}

fn trailing_owned_end(source: &str, prop_end: usize, limit: usize) -> (usize, bool) {
    let bytes = source.as_bytes();
    let mut index = prop_end.min(limit);
    index = skip_hspace(source, index, limit);
    let mut has_comma = false;
    if bytes.get(index) == Some(&b',') && index < limit {
        has_comma = true;
        index += 1;
        index = skip_hspace(source, index, limit);
    }
    if source
        .get(index..)
        .is_some_and(|tail| tail.starts_with("//"))
    {
        index = skip_line_comment(source, index, limit);
    } else if source
        .get(index..)
        .is_some_and(|tail| tail.starts_with("/*"))
        && let Some(end) = block_comment_end(source, index, limit)
        && source
            .get(index + 2..end.saturating_sub(2))
            .is_some_and(|body| !contains_blank_line(body))
    {
        index = skip_hspace(source, end, limit);
        if !has_comma && bytes.get(index) == Some(&b',') && index < limit {
            has_comma = true;
            index += 1;
            index = skip_hspace(source, index, limit);
        }
    }
    if bytes.get(index) == Some(&b'\r') && index < limit {
        index += 1;
    }
    if bytes.get(index) == Some(&b'\n') && index < limit {
        index += 1;
    }
    (index.min(limit), has_comma)
}

fn outside_prefix_is_safe(prefix: &str) -> bool {
    match prefix.find('\n') {
        None => is_whitespace_only(prefix),
        Some(newline) => {
            prefix.get(..newline).is_some_and(is_comment_trivia)
                && prefix.get(newline + 1..).is_some_and(is_whitespace_only)
        }
    }
}

fn is_comment_trivia(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b' ' | b'\t' | b'\n' | b'\r' => index += 1,
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index += 2;
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                let Some(relative) = text.get(index..).and_then(|tail| tail.find("*/")) else {
                    return false;
                };
                let body_start = index + 2;
                let body_end = index + relative;
                if text
                    .get(body_start..body_end)
                    .is_none_or(contains_blank_line)
                {
                    return false;
                }
                index = body_end + 2;
            }
            _ => return false,
        }
    }
    true
}

fn contains_blank_line(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'\n' {
            let mut next = index + 1;
            while next < bytes.len() && matches!(bytes[next], b' ' | b'\t' | b'\r') {
                next += 1;
            }
            if bytes.get(next) == Some(&b'\n') {
                return true;
            }
        }
        index += 1;
    }
    false
}

fn is_whitespace_only(text: &str) -> bool {
    text.bytes()
        .all(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b'\r'))
}

fn line_start(source: &str, index: usize) -> usize {
    source
        .get(..index)
        .and_then(|head| head.rfind('\n'))
        .map_or(0, |found| found + 1)
}

fn trim_hspace_start(source: &str, floor: usize, prop_start: usize) -> usize {
    let mut start = prop_start;
    while start > floor && matches!(source.as_bytes().get(start - 1), Some(b' ' | b'\t')) {
        start -= 1;
    }
    start
}

fn skip_hspace(source: &str, mut index: usize, limit: usize) -> usize {
    let bytes = source.as_bytes();
    while index < limit && matches!(bytes.get(index), Some(b' ' | b'\t')) {
        index += 1;
    }
    index
}

fn skip_line_comment(source: &str, mut index: usize, limit: usize) -> usize {
    let bytes = source.as_bytes();
    while index < limit && bytes.get(index) != Some(&b'\n') {
        index += 1;
    }
    index
}

fn block_comment_end(source: &str, index: usize, limit: usize) -> Option<usize> {
    let tail = source.get(index..limit)?;
    if !tail.starts_with("/*") {
        return None;
    }
    let relative = tail.find("*/")?;
    Some(index + relative + 2)
}
