//! Vue attribute-name states, independent of the argument's expression syntax.

pub(super) fn argument(input: &[u8], start: usize) -> (usize, bool) {
    let mut dynamic = false;
    let mut index = start;
    while let Some(&byte) = input.get(index) {
        let is_arrow = dynamic && byte == b'=' && input.get(index + 1) == Some(&b'>');
        if is_arrow {
            index += 2;
            continue;
        }
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
