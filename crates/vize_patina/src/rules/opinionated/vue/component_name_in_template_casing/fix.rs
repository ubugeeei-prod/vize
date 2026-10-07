//! Replace a proven component name in both authored tag delimiters.

use super::ComponentCasing;
use crate::diagnostic::{Fix, TextEdit};
use crate::rules::opinionated::vue::authored_element::{closing, opening};
use vize_croquis::naming::to_pascal_case;
use vize_l0::String;
use vize_relief::{ElementNode, Namespace};

pub(super) fn element_fix(
    source: &str,
    element: &ElementNode<'_>,
    casing: ComponentCasing,
    help: &str,
) -> Option<Fix> {
    let tag = element.tag;
    // Member expressions/custom namespace spellings are not ordinary Vue casing.
    if element.ns != Namespace::Html
        || vize_l0::is_math_ml_tag(tag)
        || !tag.as_bytes().first()?.is_ascii_alphabetic()
        || !tag.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        || tag.ends_with('-')
        || tag.contains("--")
    {
        return None;
    }
    let replacement: String = match casing {
        ComponentCasing::PascalCase => to_pascal_case(tag),
        ComponentCasing::KebabCase => {
            let mut result = String::with_capacity(tag.len());
            for (index, byte) in tag.bytes().enumerate() {
                if byte.is_ascii_uppercase() {
                    if index > 0 && !result.ends_with('-') {
                        result.push('-');
                    }
                    result.push(byte.to_ascii_lowercase() as char);
                } else {
                    result.push(byte as char);
                }
            }
            result
        }
    };
    let authored = opening(source, element)?;
    let name_start = element.loc.span.start.checked_add(1)?;
    let name_end = name_start.checked_add(u32::try_from(tag.len()).ok()?)?;
    let mut edits = vec![TextEdit::replace(name_start, name_end, replacement.clone())];
    if !authored.trim_end_matches('>').trim_end().ends_with('/') {
        let close = closing(source, element)?;
        let name_start = u32::try_from(close.start).ok()?.checked_add(2)?;
        edits.push(TextEdit::replace(
            name_start,
            name_start + tag.len() as u32,
            replacement,
        ));
    }
    Some(Fix::with_edits(help, edits))
}
