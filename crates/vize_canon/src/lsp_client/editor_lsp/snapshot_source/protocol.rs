//! Text-only view of the pinned encoder, not an AST decoder.
//!
//! Authority: microsoft/typescript-go@2bd066d8, internal/api/encoder/{encoder,
//! decoder,stringtable}.go and internal/ast/kind_generated.go. The actual encoder
//! writes the version in byte 3 (its introductory table incorrectly says byte 0).

use super::SourceTextRefusal;

const HEADER: usize = 44;
const NODE: usize = 28;
const SOURCE_FILE_KIND: u32 = 307;
const SOURCE_FILE_EXTENDED: usize = 48;

pub(super) fn source_text<'a>(
    bytes: &'a [u8],
    expected_name: &str,
) -> Result<(&'a str, &'a str, &'a str), SourceTextRefusal> {
    use SourceTextRefusal as Refuse;
    if bytes.len() < HEADER || u32::try_from(bytes.len()).is_err() {
        return Err(Refuse::Header);
    }
    if word(bytes, 0)? != 5 << 24 {
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
    if !(table == HEADER
        && table <= strings
        && strings <= extended
        && extended <= structured
        && structured <= nodes
        && nodes <= bytes.len())
        || (strings - table) % 8 != 0
        || (structured - extended) % 4 != 0
        || (bytes.len() - nodes) % NODE != 0
        || bytes.len() - nodes < 2 * NODE
    {
        return Err(Refuse::Sections);
    }
    if bytes[nodes..nodes + NODE].iter().any(|&byte| byte != 0) {
        return Err(Refuse::Root);
    }
    let root = nodes + NODE;
    let data = word(bytes, root + 20)?;
    // The encoder writes the root first, before any other extended data.
    if word(bytes, root)? != SOURCE_FILE_KIND
        || word(bytes, root + 4)? != 0
        || word(bytes, root + 12)? != 0
        || word(bytes, root + 16)? != 0
        || data != 0x8000_0000
        || structured - extended < SOURCE_FILE_EXTENDED
    {
        return Err(Refuse::Root);
    }
    // Each pair is bounded by the string region, including unused AST strings.
    // WTF-8 in unused node strings remains opaque; the three read strings must
    // individually be UTF-8. We do not assert validity of the remaining AST.
    for offset in (table..strings).step_by(8) {
        let start = index(bytes, offset)?;
        let end = index(bytes, offset + 4)?;
        if start > end || end > extended - strings {
            return Err(Refuse::Strings);
        }
    }
    let read_string = |field| {
        let pair = index(bytes, extended + field)?;
        if pair % 2 != 0 {
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
        std::str::from_utf8(&bytes[strings + start..strings + end]).map_err(|_| Refuse::Utf8)
    };
    let text = read_string(0)?;
    let file_name = read_string(4)?;
    let path = read_string(8)?;
    if word(bytes, extended)? != 0
        || word(bytes, extended + 4)? != 2
        || word(bytes, extended + 8)? != 4
        || word(bytes, table)? != 0
        || usize::try_from(word(bytes, root + 8)?).ok() != Some(text.encode_utf16().count())
    {
        return Err(Refuse::Root);
    }
    if file_name != expected_name || path != expected_name {
        return Err(Refuse::SourceIdentity);
    }
    let structured_len = nodes - structured;
    for field in (20..=40).step_by(4) {
        let offset = word(bytes, extended + field)?;
        if offset != u32::MAX
            && usize::try_from(offset)
                .ok()
                .is_none_or(|offset| offset >= structured_len)
        {
            return Err(Refuse::Root);
        }
    }
    if index(bytes, extended + 44)? >= (bytes.len() - nodes) / NODE {
        return Err(Refuse::Root);
    }
    Ok((text, file_name, path))
}

fn word(bytes: &[u8], offset: usize) -> Result<u32, SourceTextRefusal> {
    let end = offset.checked_add(4).ok_or(SourceTextRefusal::Sections)?;
    let value = bytes
        .get(offset..end)
        .and_then(|slice| <[u8; 4]>::try_from(slice).ok())
        .ok_or(SourceTextRefusal::Sections)?;
    Ok(u32::from_le_bytes(value))
}

fn index(bytes: &[u8], offset: usize) -> Result<usize, SourceTextRefusal> {
    usize::try_from(word(bytes, offset)?).map_err(|_| SourceTextRefusal::Sections)
}
