//! Vue attribute-name states, independent of the argument's expression syntax.

pub(super) fn argument(input: &[u8], start: usize) -> (usize, bool) {
    if input.get(start) == Some(&b'[') {
        let tail = input
            .get(start + 1..)
            .and_then(|bytes| core::str::from_utf8(bytes).ok())
            .unwrap_or_default();
        let (boundary, closed) = crate::markup::lex::dynamic_argument_boundary(tail);
        let end = start + 1 + boundary + usize::from(closed);
        if !closed || end == input.len() || input.get(end) == Some(&b'.') {
            return (end, closed);
        }
    }
    let mut dynamic = false;
    let mut index = start;
    while let Some(&byte) = input.get(index) {
        if byte == b'=' || byte.is_ascii_whitespace() || matches!(byte, b'/' | b'>') {
            return (index, !dynamic);
        }
        match byte {
            b'[' => dynamic = true,
            b']' => dynamic = false,
            b'.' if !dynamic => return (index, true),
            _ => {}
        }
        index += 1;
    }
    (index, !dynamic)
}
