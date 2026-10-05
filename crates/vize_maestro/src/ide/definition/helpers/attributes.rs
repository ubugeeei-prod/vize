//! Authored attribute lookup shared by definition and hover routes.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "definition helpers hand std `String` values to the lsp_types-based definition service"
)]

use super::{IdeContext, find_tag_name_span};

/// Get the attribute name and component name at the cursor position.
pub(crate) fn get_attribute_and_component_at_offset(
    ctx: &IdeContext<'_>,
) -> Option<(String, String)> {
    get_attribute_with_source_span_at_offset(ctx).map(|(name, tag, _)| (name, tag))
}

type AuthoredAttribute = (String, String, Option<(usize, usize)>);

/// Model attributes retain their separate model query route.
#[cfg(feature = "native")]
pub(crate) fn get_non_model_attribute_at_offset(ctx: &IdeContext<'_>) -> Option<(String, String)> {
    attribute_with_source_span_at_offset(ctx, false).map(|(name, tag, _)| (name, tag))
}

pub(crate) fn get_attribute_with_source_span_at_offset(
    ctx: &IdeContext<'_>,
) -> Option<AuthoredAttribute> {
    attribute_with_source_span_at_offset(ctx, true)
}

fn attribute_with_source_span_at_offset(
    ctx: &IdeContext<'_>,
    include_model: bool,
) -> Option<AuthoredAttribute> {
    let content = &ctx.content;
    let cursor = ctx.offset.min(content.len());
    let (tag_start, tag_end, name_start, name_end) = find_tag_name_span(content, cursor)?;
    let bytes = content.as_bytes();

    if bytes.get(tag_start + 1) == Some(&b'/') {
        return None;
    }

    let tag_name = content.get(name_start..name_end)?;
    let mut pos = name_end;
    // Byte at `i` while still inside the tag; `None` past `tag_end`.
    let at = |i: usize| {
        if i < tag_end {
            bytes.get(i).copied()
        } else {
            None
        }
    };

    while pos < tag_end {
        while at(pos).is_some_and(|b| b.is_ascii_whitespace()) {
            pos += 1;
        }

        let Some(first) = at(pos) else { break };
        if first == b'/' {
            break;
        }

        let attr_start = pos;
        while at(pos).is_some_and(|b| !b.is_ascii_whitespace() && b != b'=' && b != b'/') {
            pos += 1;
        }
        let attr_end = pos;

        if attr_start == attr_end {
            break;
        }

        let cursor_on_attr_name = cursor >= attr_start && cursor <= attr_end;
        let raw_attr_name = content.get(attr_start..attr_end)?;

        while at(pos).is_some_and(|b| b.is_ascii_whitespace()) {
            pos += 1;
        }

        if at(pos) == Some(b'=') {
            pos += 1;
            while at(pos).is_some_and(|b| b.is_ascii_whitespace()) {
                pos += 1;
            }

            if let Some(quote @ (b'"' | b'\'')) = at(pos) {
                pos += 1;
                while at(pos).is_some_and(|b| b != quote) {
                    pos += 1;
                }
                if pos < tag_end {
                    pos += 1;
                }
            } else {
                while at(pos).is_some_and(|b| !b.is_ascii_whitespace() && b != b'>') {
                    pos += 1;
                }
            }
        }

        if !cursor_on_attr_name {
            continue;
        }

        let mut attr_name = raw_attr_name;
        if let Some(model_prop_name) =
            super::component_model::prop_name_from_v_model_attribute(raw_attr_name)
        {
            return include_model.then(|| (model_prop_name, tag_name.to_string(), None));
        } else if let Some(stripped) = attr_name.strip_prefix(':') {
            attr_name = stripped;
        } else if let Some(stripped) = attr_name.strip_prefix("v-bind:") {
            attr_name = stripped;
        } else if attr_name.starts_with('@')
            || attr_name.starts_with("v-on:")
            || attr_name.starts_with("v-")
        {
            return None;
        }

        if attr_name.is_empty() {
            return None;
        }

        let source_span = (attr_end - attr_name.len(), attr_end);
        return Some((
            attr_name.to_string(),
            tag_name.to_string(),
            Some(source_span),
        ));
    }

    None
}
