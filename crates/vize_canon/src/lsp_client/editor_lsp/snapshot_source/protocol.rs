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
        || !(bytes.len() - nodes).is_multiple_of(NODE)
        || bytes.len() - nodes < 2 * NODE
    {
        return Err(Refuse::Sections);
    }
    let sentinel = bytes.get(nodes..nodes + NODE).ok_or(Refuse::Root)?;
    if sentinel.iter().any(|&byte| byte != 0) {
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
        let value = bytes
            .get(strings + start..strings + end)
            .ok_or(Refuse::Strings)?;
        std::str::from_utf8(value).map_err(|_| Refuse::Utf8)
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
    if !source_identity_matches(file_name, path, expected_name) {
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

/// The encoded `fileName` is the name as requested, but `path` is the
/// checker's canonical key: on a case-insensitive host (macOS, Windows) it is
/// `tspath.ToFileNameLowerCase` of that name. A session directory such as
/// `session-EAOcM2` or a Nuxt route folder such as `[userId]` therefore never
/// matches byte for byte, and refusing it retired the backend on every decode.
fn source_identity_matches(file_name: &str, path: &str, expected_name: &str) -> bool {
    file_name == expected_name
        && (path == expected_name || path == to_file_name_lower_case(expected_name))
}

/// `tspath.ToFileNameLowerCase`: lowercase every character except U+0130,
/// whose lowercase form would change the name's length.
fn to_file_name_lower_case(name: &str) -> vize_l0::String {
    name.chars()
        .flat_map(|character| {
            let keep = character == '\u{0130}';
            keep.then_some(character)
                .into_iter()
                .chain(character.to_lowercase().filter(move |_| !keep))
        })
        .collect()
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

#[cfg(test)]
mod identity_tests {
    use super::source_identity_matches;

    #[test]
    fn case_insensitive_canonical_paths_match_their_requested_name() {
        let name =
            "/private/var/T/vize-canon/session-EAOcM2/app/pages/ticket/[userId]/index.vue.ts";
        let canonical =
            "/private/var/t/vize-canon/session-eaocm2/app/pages/ticket/[userid]/index.vue.ts";
        assert!(source_identity_matches(name, name, name));
        assert!(source_identity_matches(name, canonical, name));
        assert!(!source_identity_matches(canonical, canonical, name));
        assert!(!source_identity_matches(
            name,
            "/private/var/t/vize-canon/session-eaocm2/app/pages/ticket/[other]/index.vue.ts",
            name
        ));
    }
}
