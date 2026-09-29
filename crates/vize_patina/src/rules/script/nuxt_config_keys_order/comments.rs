use oxc_ast::ast::ObjectExpression;
use oxc_span::GetSpan;
use vize_l0::{String, ToCompactString};

use super::support::{legacy_property_text_ranges, next_token_offset};

#[path = "comments_text.rs"]
mod text;

use text::{
    block_comment_end, contains_blank_line, is_comment_trivia, is_whitespace_only, line_start,
    outside_prefix_is_safe, skip_hspace, skip_line_comment, trim_hspace_start,
};

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
        let prop_end = owned.get(index)?.prop_end;
        let (end, has_comma) = trailing_owned_end(source, prop_end, limit);
        if end < prop_end || end > limit {
            return None;
        }
        let slot = owned.get_mut(index)?;
        slot.end = end;
        slot.has_comma = has_comma;
    }

    let prefix = source.get(brace + 1..owned.first()?.start)?;
    if !outside_prefix_is_safe(prefix) {
        return Some(PropertyText::Unfixable);
    }
    let suffix = source.get(owned.last()?.end..close_brace)?;
    if !is_whitespace_only(suffix) {
        return Some(PropertyText::Unfixable);
    }

    let mut separators = Vec::with_capacity(owned.len().saturating_sub(1));
    for pair in owned.windows(2) {
        let (Some(left), Some(right)) = (pair.first(), pair.get(1)) else {
            return None;
        };
        if right.start < left.end {
            return None;
        }
        let gap = source.get(left.end..right.start)?;
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
        start: owned.first()?.start,
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
