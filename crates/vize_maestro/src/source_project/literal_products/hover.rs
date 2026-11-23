//! The typed editor signature for a binding with a single local identity.
use crate::ide::{hover::HoverService, markup};
use tower_lsp::lsp_types::{Hover, HoverContents, Range};
use vize_canon::literal_assignments::BindingType;
use vize_l0::{Span, cstr};
use vize_l1::container::vue::ScriptRole;

pub(crate) fn typed_binding_hover(
    source: &str,
    script: Span,
    role: ScriptRole,
    item: BindingType,
    range: Range,
) -> Option<Hover> {
    let name = item.binding.slice(source);
    if let Some(answer) = HoverService::literal_binding_override(name, role == ScriptRole::Setup) {
        return Some(answer);
    }
    // An ordinary exported binding obtains its signature from the virtual
    // module. Documentation and merged/escaped identities need symbol lookup.
    let body = script.slice(source);
    let offset = item.binding.start.checked_sub(script.start)? as usize;
    if (item.exported && role == ScriptRole::Ordinary)
        || body.get(..offset)?.contains("/**")
        || body.contains("\\u")
        || body.match_indices(name).any(|(at, _)| {
            at != offset
                && !body
                    .get(..at)
                    .and_then(|s| s.chars().next_back())
                    .is_some_and(oxc_syntax::identifier::is_identifier_part)
                && !body
                    .get(at + name.len()..)
                    .and_then(|s| s.chars().next())
                    .is_some_and(oxc_syntax::identifier::is_identifier_part)
        })
    {
        return None;
    }
    let signature = cstr!("{} {name}: {}", item.kind.as_str(), item.declared.keyword());
    Some(Hover {
        contents: HoverContents::Markup(markup::markdown_content(markup::code_block(
            "typescript",
            &signature,
        ))),
        range: Some(range),
    })
}
