use crate::fast::SmallVec;

pub(crate) type SplitParts<'a> = SmallVec<[&'a str; 4]>;

#[derive(Clone, Copy, Default)]
struct Nesting {
    angle: usize,
    square: usize,
    paren: usize,
    brace: usize,
    quote: Option<char>,
}

impl Nesting {
    fn at_top_level(self) -> bool {
        self.angle == 0
            && self.square == 0
            && self.paren == 0
            && self.brace == 0
            && self.quote.is_none()
    }

    fn step(&mut self, ch: char) {
        if let Some(active_quote) = self.quote {
            if ch == active_quote {
                self.quote = None;
            }
            return;
        }
        match ch {
            '\'' | '"' | '`' => self.quote = Some(ch),
            '<' => self.angle += 1,
            '>' if self.angle > 0 => self.angle -= 1,
            '[' => self.square += 1,
            ']' if self.square > 0 => self.square -= 1,
            '(' => self.paren += 1,
            ')' if self.paren > 0 => self.paren -= 1,
            '{' => self.brace += 1,
            '}' if self.brace > 0 => self.brace -= 1,
            _ => {}
        }
    }
}

pub(crate) fn split_top_level_owned(text: &str, delimiter: char) -> Vec<String> {
    split_refs(text, delimiter, None)
        .into_iter()
        .map(str::to_owned)
        .collect()
}

pub(crate) fn split_type_text_owned(text: &str) -> Vec<String> {
    split_refs(text, '|', Some('&'))
        .into_iter()
        .map(str::to_owned)
        .collect()
}

pub(crate) fn split_top_level_once(text: &str, delimiter: char) -> Option<SplitParts<'_>> {
    let parts = split_refs(text, delimiter, None);
    (parts.len() > 1).then_some(parts)
}

pub(crate) fn split_comma_refs(text: &str) -> SplitParts<'_> {
    split_refs(text, ',', None)
}

pub(crate) fn split_generic(text: &str) -> Option<(&str, &str)> {
    let text = text.trim();
    let mut nesting = Nesting::default();
    let mut start = None;
    for (index, ch) in text.char_indices() {
        if ch == '<' && nesting.at_top_level() {
            start.get_or_insert(index);
        }
        nesting.step(ch);
    }
    let start = start?;
    if nesting.angle != 0 || start == 0 || !text.ends_with('>') {
        return None;
    }
    let base = text[..start].trim();
    if base.is_empty() {
        return None;
    }
    Some((base, &text[start + 1..text.len() - 1]))
}

pub(crate) fn strip_wrapping_parens(text: &str) -> &str {
    let mut current = text.trim();
    while current.starts_with('(') && current.ends_with(')') && is_wrapped_by(current, '(', ')') {
        current = current[1..current.len() - 1].trim();
    }
    current
}

pub(crate) fn is_wrapped_by(text: &str, open: char, close: char) -> bool {
    if !text.starts_with(open) || !text.ends_with(close) {
        return false;
    }
    let mut depth = 0usize;
    let mut quote = None;
    for (index, ch) in text.char_indices() {
        if let Some(active_quote) = quote {
            if ch == active_quote {
                quote = None;
            }
            continue;
        }
        match ch {
            '\'' | '"' | '`' => quote = Some(ch),
            _ if ch == open => depth += 1,
            _ if ch == close => {
                if depth == 0 {
                    return false;
                }
                depth -= 1;
                if depth == 0 && index + ch.len_utf8() != text.len() {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0
}

fn split_refs(text: &str, primary: char, secondary: Option<char>) -> SplitParts<'_> {
    let text = text.trim();
    if text.is_empty() {
        return SplitParts::new();
    }
    let mut parts = SplitParts::new();
    let mut nesting = Nesting::default();
    let mut start = 0usize;
    for (index, ch) in text.char_indices() {
        if nesting.at_top_level() && (ch == primary || secondary == Some(ch)) {
            push_part(&mut parts, &text[start..index]);
            start = index + ch.len_utf8();
            continue;
        }
        nesting.step(ch);
    }
    push_part(&mut parts, &text[start..]);
    parts
}

fn push_part<'a>(parts: &mut SplitParts<'a>, part: &'a str) {
    let part = part.trim();
    if !part.is_empty() {
        parts.push(part);
    }
}
