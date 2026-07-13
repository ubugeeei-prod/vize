//! Allocation-light top-level SFC block classification.

use memchr::{memchr, memchr_iter, memmem::Finder};

use super::block::{parse_block_fast, tag_name_eq};

const TAG_TEMPLATE: &[u8] = b"template";
const TAG_SCRIPT: &[u8] = b"script";

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct SfcSourceStructure {
    pub has_template: bool,
    pub has_script: bool,
    pub vapor_script: bool,
}

/// Classify top-level blocks with the same robust boundary scanner as `parse_sfc`.
///
/// This deliberately does not construct a descriptor, syntax tree, or owned
/// source snapshot. Atlas uses it only to select the dependency closure before
/// the descriptor provider executes exactly once.
pub(crate) fn scan_sfc_structure(source: &str) -> Option<SfcSourceStructure> {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut pos = 0;
    let mut line = 1;
    let comment_end_finder = Finder::new(b"-->");
    let mut structure = SfcSourceStructure::default();
    let mut script_seen = false;
    let mut script_setup_seen = false;

    while pos < len {
        while pos < len && bytes[pos].is_ascii_whitespace() {
            if bytes[pos] == b'\n' {
                line += 1;
            }
            pos += 1;
        }
        if pos >= len {
            break;
        }
        if bytes[pos] != b'<' {
            let Some(next_lt) = memchr(b'<', &bytes[pos..]) else {
                break;
            };
            line += memchr_iter(b'\n', &bytes[pos..pos + next_lt]).count();
            pos += next_lt;
        }
        if bytes[pos..].starts_with(b"<!--") {
            let end = comment_end_finder
                .find(&bytes[pos + 4..])
                .map_or(len, |offset| pos + 4 + offset + 3);
            line += memchr_iter(b'\n', &bytes[pos..end]).count();
            pos = end;
            continue;
        }

        match parse_block_fast(bytes, source, pos, line) {
            Ok(Some((tag, attrs, _, _, _, end, end_line, _))) => {
                if tag_name_eq(tag, TAG_TEMPLATE) {
                    if structure.has_template {
                        return None;
                    }
                    structure.has_template = true;
                } else if tag_name_eq(tag, TAG_SCRIPT) {
                    let setup = attrs.contains_key("setup");
                    if (setup && script_setup_seen) || (!setup && script_seen) {
                        return None;
                    }
                    script_setup_seen |= setup;
                    script_seen |= !setup;
                    structure.has_script = true;
                    structure.vapor_script |= attrs.contains_key("vapor");
                }
                pos = end;
                line = end_line;
            }
            Ok(None) => pos += 1,
            Err(_) => return None,
        }
    }

    Some(structure)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_block_spellings_in_comments_and_script_literals() {
        let structure = scan_sfc_structure(
            r#"<!-- <template><fake /></template> -->
<script>const template = "<template>"; const vapor = "<script vapor>"</script>"#,
        )
        .unwrap();
        assert_eq!(
            structure,
            SfcSourceStructure {
                has_template: false,
                has_script: true,
                vapor_script: false,
            }
        );
    }

    #[test]
    fn classifies_script_attributes_without_substring_matching() {
        let structure = scan_sfc_structure(
            r#"<script setup lang="ts" vapor>const ready = true</script>
<template><main>{{ ready }}</main></template>"#,
        )
        .unwrap();
        assert_eq!(
            structure,
            SfcSourceStructure {
                has_template: true,
                has_script: true,
                vapor_script: true,
            }
        );
    }
}
