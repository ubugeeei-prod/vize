mod interpolation;
mod tags;
#[cfg(test)]
mod tests;

use interpolation::InterpolationScan;
use tags::{RawRegion, starts_v_pre_attribute, tag_name_at};

/// Per-line "this line is inside a whitespace-significant block" mask.
///
/// Lines inside `<pre>`, `<textarea>`, `v-pre`, multi-line comments,
/// literal multi-line attribute values, and multi-line template literals
/// inside `{{ }}` interpolations are raw. Directive expression continuation
/// lines are formatter output, so they still get SFC indentation unless the
/// value starts on the following line and was preserved verbatim.
///
/// The interpolation case matters because every byte between the backticks is
/// part of the string's runtime value. Indenting those lines rewrote the
/// emitted string, and since each pass re-indented the spaces the previous one
/// added, `vize fmt` drifted the content further on every run (#3334).
///
/// `v-pre` needs the same treatment for the same reason. Its content is copied
/// verbatim by the template formatter, so an SFC indent laid on top of it moved
/// the content two columns further on every `vize fmt` run — unbounded drift,
/// not a one-off reformat (#3379).
pub(super) fn compute_raw_line_mask<'a>(lines: &[&'a [u8]]) -> Vec<bool> {
    let mut mask = vec![false; lines.len()];
    let mut depth_stack: Vec<RawRegion<'a>> = Vec::new();
    let mut in_tag = false;
    let mut open_quote: Option<OpenQuote> = None;
    let mut pending_raw_tag: Option<&'static str> = None;
    // The element the opening tag currently being lexed declares, and whether
    // it has shown a `v-pre` attribute yet. Both outlive a single line: an
    // opening tag may be split across lines with `v-pre` on its own.
    let mut open_tag_name: Option<&'a [u8]> = None;
    let mut open_tag_is_pre = false;
    let mut in_comment = false;
    // Lexer state for the inside of a `{{ … }}` interpolation. The template
    // formatter already emits template-literal quasi lines verbatim; the SFC
    // layer must not indent them on top.
    let mut interpolation = InterpolationScan::default();
    // A `{{` with no `}}` after it is text, not an interpolation (Vue treats
    // an unterminated marker as plain text). Entering expression mode there
    // would disable tag, comment and `<pre>`/`<textarea>` tracking for the
    // rest of the document, so activation is gated on a closing pair
    // actually following.
    let last_close_line = lines.iter().rposition(|line| contains(line, b"}}"));
    const TAGS: [(&str, &str, &str); 2] = [
        ("pre", "<pre", "</pre>"),
        ("textarea", "<textarea", "</textarea>"),
    ];

    for (i, line) in lines.iter().enumerate() {
        // `'…'` / `"…"` cannot span a newline in JS, so an unbalanced quote
        // must not swallow the following lines as string content.
        interpolation.string = None;
        if !depth_stack.is_empty()
            || open_quote.is_some_and(OpenQuote::marks_line_raw)
            || in_comment
            || interpolation.line_starts_in_quasi()
        {
            mask[i] = true;
        }

        let bytes = line;
        let mut cursor = 0;
        while cursor < bytes.len() {
            if in_comment {
                if bytes[cursor..].starts_with(b"-->") {
                    in_comment = false;
                    cursor += 3;
                } else {
                    cursor += 1;
                }
                continue;
            }
            if let Some(mut quote) = open_quote {
                if quote.directive
                    && !quote.raw
                    && bytes[cursor] == b'`'
                    && !is_escaped(bytes, cursor)
                {
                    quote.in_template_literal = !quote.in_template_literal;
                    open_quote = Some(quote);
                    cursor += 1;
                    continue;
                }
                if bytes[cursor] == quote.quote && !quote.in_template_literal {
                    open_quote = None;
                }
                cursor += 1;
                continue;
            }
            if in_tag {
                match bytes[cursor] {
                    b'"' | b'\'' => {
                        open_quote = Some(OpenQuote::new(bytes, cursor));
                    }
                    b'>' => {
                        in_tag = false;
                        // `<Foo v-pre />` has no content to keep raw. The
                        // template formatter takes its self-closing branch
                        // before the whitespace-significant one, so the mask
                        // must not open a region the formatter never opened.
                        let self_closing = cursor > 0 && bytes[cursor - 1] == b'/';
                        if let Some(tag) = pending_raw_tag.take() {
                            depth_stack.push(RawRegion {
                                tag: tag.as_bytes(),
                                v_pre: false,
                            });
                        } else if let Some(tag) = open_tag_name.filter(|_| open_tag_is_pre)
                            && !self_closing
                        {
                            depth_stack.push(RawRegion { tag, v_pre: true });
                        }
                        open_tag_name = None;
                        open_tag_is_pre = false;
                    }
                    _ => {
                        if starts_v_pre_attribute(bytes, cursor) {
                            open_tag_is_pre = true;
                        }
                    }
                }
                cursor += 1;
                continue;
            }
            // Inside an interpolation the bytes are a JS expression, not
            // markup: `<` is a comparison and `}}` only closes the
            // interpolation when reached in code position.
            if interpolation.active {
                cursor = interpolation.step(bytes, cursor);
                continue;
            }
            if bytes[cursor..].starts_with(b"{{") {
                interpolation.active = contains(&bytes[cursor + 2..], b"}}")
                    || last_close_line.is_some_and(|last| last > i);
                cursor += 2;
                continue;
            }
            if bytes[cursor] != b'<' {
                cursor += 1;
                continue;
            }
            if bytes[cursor..].starts_with(b"<!--") {
                in_comment = true;
                cursor += 4;
                continue;
            }

            let mut matched = false;
            for (tag, open_needle, close_needle) in &TAGS {
                if starts_with_ascii_ci(&bytes[cursor..], close_needle.as_bytes()) {
                    if let Some(idx) = depth_stack
                        .iter()
                        .rposition(|region| !region.v_pre && region.tag == tag.as_bytes())
                    {
                        depth_stack.remove(idx);
                    }
                    cursor += close_needle.len();
                    matched = true;
                    break;
                }
                if starts_with_ascii_ci(&bytes[cursor..], open_needle.as_bytes())
                    && bytes
                        .get(cursor + open_needle.len())
                        .copied()
                        .is_none_or(|after| matches!(after, b'>' | b' ' | b'\t' | b'\r' | b'/'))
                {
                    pending_raw_tag = Some(tag);
                    in_tag = true;
                    cursor += open_needle.len();
                    matched = true;
                    break;
                }
            }
            if matched {
                continue;
            }
            // A `v-pre` region is named by the element that carries the
            // directive, so it ends at that element's own `</tag>`.
            if bytes[cursor..].starts_with(b"</")
                && let Some(name) = tag_name_at(bytes, cursor + 2)
                && let Some(idx) = depth_stack
                    .iter()
                    .rposition(|region| region.v_pre && region.tag.eq_ignore_ascii_case(name))
            {
                depth_stack.remove(idx);
                cursor += 2 + name.len();
                continue;
            }
            // Generic opening tags are not lexed inside a raw region, so a
            // same-name element nested in a `v-pre` one has to be counted here
            // or its `</tag>` would end the region early. This mirrors how a
            // `<pre>` nested in a `<pre>` stacks through the `TAGS` loop above.
            if let Some(region) = depth_stack.last().copied().filter(|region| region.v_pre)
                && let Some(name) = tag_name_at(bytes, cursor + 1)
                && name.eq_ignore_ascii_case(region.tag)
                && !self_closes_on_this_line(bytes, cursor)
            {
                depth_stack.push(region);
                cursor += 1 + name.len();
                continue;
            }
            if depth_stack.is_empty()
                && let Some(after) = bytes.get(cursor + 1).copied()
                && (after.is_ascii_alphabetic() || after == b'/')
            {
                in_tag = true;
                // `None` for a closing tag, which opens no region.
                open_tag_name = tag_name_at(bytes, cursor + 1);
                open_tag_is_pre = false;
            }
            cursor += 1;
        }
    }
    mask
}

/// Whether the tag starting at `cursor` closes itself before this line ends.
///
/// Only the current line is available here, and a tag name cannot be split, so
/// an opening tag that runs past the line end is treated as not self-closing —
/// the `/>` form is written on one line in every formatter output.
fn self_closes_on_this_line(bytes: &[u8], cursor: usize) -> bool {
    memchr::memchr(b'>', &bytes[cursor..])
        .is_some_and(|offset| offset > 0 && bytes[cursor + offset - 1] == b'/')
}

fn literal_attr_quote(line: &[u8], quote_pos: usize) -> bool {
    attr_name_before_quote(line, quote_pos).is_none_or(|name| {
        !directive_expr_attr(name) || verbatim_multiline_directive_attr(name, line, quote_pos)
    })
}

#[derive(Clone, Copy)]
struct OpenQuote {
    quote: u8,
    raw: bool,
    directive: bool,
    in_template_literal: bool,
}

impl OpenQuote {
    fn new(line: &[u8], quote_pos: usize) -> Self {
        let attr_name = attr_name_before_quote(line, quote_pos);
        Self {
            quote: line[quote_pos],
            raw: literal_attr_quote(line, quote_pos),
            directive: attr_name.is_some_and(directive_expr_attr),
            in_template_literal: false,
        }
    }

    fn marks_line_raw(self) -> bool {
        self.raw || self.in_template_literal
    }
}

fn verbatim_multiline_directive_attr(name: &[u8], line: &[u8], quote_pos: usize) -> bool {
    name == b"v-for" || value_starts_on_following_line(line, quote_pos)
}

fn value_starts_on_following_line(line: &[u8], quote_pos: usize) -> bool {
    line.get(quote_pos + 1..)
        .is_none_or(|tail| tail.iter().all(|b| matches!(b, b' ' | b'\t' | b'\r')))
}

fn is_escaped(line: &[u8], pos: usize) -> bool {
    let mut backslashes = 0;
    let mut cursor = pos;
    while cursor > 0 && line[cursor - 1] == b'\\' {
        backslashes += 1;
        cursor -= 1;
    }
    backslashes % 2 == 1
}

fn attr_name_before_quote(line: &[u8], quote_pos: usize) -> Option<&[u8]> {
    let mut pos = quote_pos;
    while pos > 0 && matches!(line[pos - 1], b' ' | b'\t') {
        pos -= 1;
    }
    if pos == 0 || line[pos - 1] != b'=' {
        return None;
    }
    pos -= 1;
    while pos > 0 && matches!(line[pos - 1], b' ' | b'\t') {
        pos -= 1;
    }
    let end = pos;
    while pos > 0
        && !matches!(
            line[pos - 1],
            b' ' | b'\t' | b'\r' | b'\n' | b'<' | b'>' | b'/'
        )
    {
        pos -= 1;
    }
    (pos < end).then_some(&line[pos..end])
}

fn directive_expr_attr(name: &[u8]) -> bool {
    name.starts_with(b":")
        || name.starts_with(b"@")
        || name.starts_with(b"v-if")
        || name.starts_with(b"v-else-if")
        || name.starts_with(b"v-show")
        || name.starts_with(b"v-for")
        || name.starts_with(b"v-model")
        || name.starts_with(b"v-bind")
        || name.starts_with(b"v-on")
        || name == b"v-html"
        || name == b"v-text"
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.len() >= needle.len() && haystack.windows(needle.len()).any(|w| w == needle)
}

fn starts_with_ascii_ci(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.len() >= needle.len()
        && haystack[..needle.len()]
            .iter()
            .zip(needle.iter())
            .all(|(a, b)| a.eq_ignore_ascii_case(b))
}
