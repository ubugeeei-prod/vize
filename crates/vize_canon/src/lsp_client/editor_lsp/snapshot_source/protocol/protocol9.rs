//! Text-root view ONLY of TypeScript 7.1.0-beta's protocol 9.
//!
//! Primary: microsoft/TypeScript@3e9236361fa97a9e6c685f4833ac457ae2d8c50a,
//! tsc/internal/api/encoder/encoder.go and tsc/internal/ast/kind_generated.go.
//! The complete source bytes and identities belong to the caller's SAME lease.
//! Nodes, binder data and content-mapper metadata remain opaque: this is not an
//! AST/mapper decoder, and no metadata here supplies a diagnostic position.

use super::{SourceTextRefusal, index, source_identity_matches, word};

const HEADER: usize = 64;
const NODE: usize = 28;
const SOURCE_FILE_KIND: u32 = 308;
const SOURCE_FILE_EXTENDED: usize = 76;

pub(super) fn source_text<'a>(
    bytes: &'a [u8],
    expected_name: &str,
) -> Result<(&'a str, &'a str, &'a str), SourceTextRefusal> {
    use SourceTextRefusal as Refuse;
    if bytes.len() < HEADER || u32::try_from(bytes.len()).is_err() {
        return Err(Refuse::Header);
    }
    if word(bytes, 0)? != 9 << 24 {
        return Err(Refuse::Version);
    }
    if word(bytes, 20)? & !3 != 0 {
        return Err(Refuse::Header);
    }
    let table = index(bytes, 24)?;
    let strings = index(bytes, 28)?;
    let extended = index(bytes, 32)?;
    let structured = index(bytes, 36)?;
    let nodes = index(bytes, 40)?;
    let binder = index(bytes, 60)?;
    let node_end = if binder == 0 { bytes.len() } else { binder };
    if !(table == HEADER
        && table <= strings
        && strings <= extended
        && extended <= structured
        && structured <= nodes
        && nodes <= node_end
        && node_end <= bytes.len())
        || (strings - table) % 8 != 0
        || (structured - extended) % 4 != 0
        || !(node_end - nodes).is_multiple_of(NODE)
        || node_end - nodes < 2 * NODE
    {
        return Err(Refuse::Sections);
    }
    let sentinel = bytes.get(nodes..nodes + NODE).ok_or(Refuse::Root)?;
    if sentinel.iter().any(|&byte| byte != 0) {
        return Err(Refuse::Root);
    }
    let root = nodes + NODE;
    if word(bytes, root)? != SOURCE_FILE_KIND
        || word(bytes, root + 4)? != 0
        || word(bytes, root + 12)? != 0
        || word(bytes, root + 16)? != 0
        || word(bytes, root + 20)? != 0x8000_0000
        || structured - extended < SOURCE_FILE_EXTENDED
    {
        return Err(Refuse::Root);
    }
    for offset in (table..strings).step_by(8) {
        let start = index(bytes, offset)?;
        let end = index(bytes, offset + 4)?;
        if start > end || end > extended - strings {
            return Err(Refuse::Strings);
        }
    }
    let string_bytes = |pair: usize| {
        if !pair.is_multiple_of(2) {
            return Err(Refuse::Strings);
        }
        let offset = pair
            .checked_mul(4)
            .and_then(|pair| table.checked_add(pair))
            .ok_or(Refuse::Strings)?;
        if offset.checked_add(8).is_none_or(|end| end > strings) {
            return Err(Refuse::Strings);
        }
        let start = index(bytes, offset)?;
        let end = index(bytes, offset + 4)?;
        bytes
            .get(strings + start..strings + end)
            .ok_or(Refuse::Strings)
    };
    let read_string = |field| {
        let value = string_bytes(index(bytes, extended + field)?)?;
        std::str::from_utf8(value).map_err(|_| Refuse::Utf8)
    };
    let text = read_string(0)?;
    let file_name = read_string(4)?;
    let path = read_string(8)?;
    if word(bytes, extended)? != 0
        || word(bytes, table)? != 0
        || usize::try_from(word(bytes, root + 8)?).ok() != Some(text.encode_utf16().count())
    {
        return Err(Refuse::Root);
    }
    if !source_identity_matches(file_name, path, expected_name) {
        return Err(Refuse::SourceIdentity);
    }
    // Bound every known root reference, without interpreting its opaque payload.
    let structured_len = nodes - structured;
    for field in [20, 24, 28, 32, 36, 40, 52, 56, 72] {
        let offset = word(bytes, extended + field)?;
        if offset != u32::MAX
            && usize::try_from(offset)
                .ok()
                .is_none_or(|offset| offset >= structured_len)
        {
            return Err(Refuse::Root);
        }
    }
    // originalText is always a string; the other three strings are optional.
    string_bytes(index(bytes, extended + 48)?)?;
    for field in [60, 64, 68] {
        let pair = word(bytes, extended + field)?;
        if pair != u32::MAX {
            string_bytes(usize::try_from(pair).map_err(|_| Refuse::Strings)?)?;
        }
    }
    if index(bytes, extended + 44)? >= (node_end - nodes) / NODE {
        return Err(Refuse::Root);
    }
    Ok((text, file_name, path))
}

#[cfg(test)]
mod tests;
