//! Low-level Art-tag scanning and standalone script-metadata extraction.

use memchr::memmem;

use super::MuseaScriptMetadata;

/// Check if a tag has an attribute using the byte scanner.
#[inline]
pub(super) fn has_attribute(tag: &[u8], attr: &[u8]) -> bool {
    memmem::find(tag, attr).is_some()
}

pub(super) fn define_art_rule_info(source: &str) -> MuseaScriptMetadata {
    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(source, Default::default()) else {
        return MuseaScriptMetadata::default();
    };
    let Some(script_setup) = descriptor.script_setup.as_ref() else {
        return MuseaScriptMetadata::default();
    };

    let parsed = vize_croquis::script_parser::parse_script_setup(script_setup.content.as_ref());
    let Some(art) = parsed.macros.define_art() else {
        return MuseaScriptMetadata::default();
    };

    MuseaScriptMetadata {
        has_title: art.title.is_some() || !art.component_name.is_empty(),
        has_component: art.component_source.is_some(),
    }
}

/// Extract the value of the name attribute from a tag.
#[inline]
pub(super) fn extract_name_attr_bytes(tag: &[u8]) -> Option<&[u8]> {
    let name_pos = memmem::find(tag, b"name=")?;
    let after_eq = &tag[name_pos + 5..];

    let mut i = 0;
    while i < after_eq.len() && after_eq[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= after_eq.len() {
        return None;
    }

    let quote = after_eq[i];
    if quote != b'"' && quote != b'\'' {
        return None;
    }

    let after_quote = &after_eq[i + 1..];
    let end_quote = memchr::memchr(quote, after_quote)?;
    Some(&after_quote[..end_quote])
}

#[inline]
pub(super) fn is_whitespace_only(bytes: &[u8]) -> bool {
    bytes.iter().all(|byte| byte.is_ascii_whitespace())
}
