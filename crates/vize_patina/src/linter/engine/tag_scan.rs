//! Low-level byte-oriented tag scanning primitives shared by template
//! extraction and ecosystem hint detection.

pub(super) fn find_tag_end(bytes: &[u8], start: usize) -> Option<usize> {
    memchr::memchr(b'>', &bytes[start..]).map(|offset| start + offset)
}

pub(super) fn find_closing_tag(bytes: &[u8], tag_name: &[u8], from: usize) -> Option<usize> {
    let mut pos = from;

    while pos < bytes.len() {
        let next_lt = match memchr::memmem::find(&bytes[pos..], b"</") {
            Some(offset) => pos + offset,
            None => return None,
        };

        if closing_tag_name_at(bytes, next_lt)
            .is_some_and(|(name, _)| name.eq_ignore_ascii_case(tag_name))
        {
            return Some(next_lt);
        }

        pos = next_lt + 2;
    }

    None
}

pub(super) fn tag_name_at(bytes: &[u8], lt_idx: usize) -> Option<(&[u8], usize)> {
    if bytes.get(lt_idx) != Some(&b'<') {
        return None;
    }

    let name_start = lt_idx + 1;
    match bytes.get(name_start) {
        Some(b'!' | b'/' | b'?') | None => return None,
        _ => {}
    }

    read_tag_name(bytes, name_start)
}

pub(super) fn closing_tag_name_at(bytes: &[u8], lt_idx: usize) -> Option<(&[u8], usize)> {
    if bytes.get(lt_idx) != Some(&b'<') || bytes.get(lt_idx + 1) != Some(&b'/') {
        return None;
    }

    read_tag_name(bytes, lt_idx + 2)
}

fn read_tag_name(bytes: &[u8], name_start: usize) -> Option<(&[u8], usize)> {
    let mut name_end = name_start;
    while bytes
        .get(name_end)
        .is_some_and(|byte| is_tag_name_byte(*byte))
    {
        name_end += 1;
    }

    if name_end == name_start || !is_tag_boundary(bytes, name_end) {
        return None;
    }

    Some((&bytes[name_start..name_end], name_end))
}

fn is_tag_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':' | b'.')
}

fn is_tag_boundary(bytes: &[u8], idx: usize) -> bool {
    matches!(
        bytes.get(idx),
        None | Some(b'>' | b'/' | b' ' | b'\n' | b'\r' | b'\t' | b'\x0c')
    )
}
