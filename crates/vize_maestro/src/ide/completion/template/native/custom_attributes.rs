//! Contextual data and ARIA attributes without expanding the default item bank.

use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};

use crate::ide::completion::items;
use crate::ide::{IdeContext, pascal_to_kebab};

use super::super::tag_context::OpenTagContext;

pub(in crate::ide::completion::template) fn completions(
    ctx: &IdeContext<'_>,
    tag: &OpenTagContext,
    component: &[CompletionItem],
) -> Vec<CompletionItem> {
    let prefix = attribute_name(&tag.current_token);
    if prefix.is_empty() {
        return Vec::new();
    }
    let data_prefix = "data-".starts_with(prefix) || prefix.starts_with("data-");
    let aria_prefix = "aria-".starts_with(prefix) || prefix.starts_with("aria-");
    if !data_prefix && !aria_prefix {
        return Vec::new();
    }
    let authored = authored_name(ctx, tag);
    let authored_attribute = attribute_name(authored);
    let assigned_aria = ARIA_ATTRIBUTES.contains(&authored_attribute)
        && ctx
            .content
            .get(tag.current_token_start + authored.len()..)
            .is_some_and(|rest| rest.trim_start().starts_with('='));
    let mut candidates = if data_prefix {
        vec![data_attribute(ctx, tag, prefix, authored)]
    } else {
        ARIA_ATTRIBUTES
            .iter()
            // The existing common bank already owns this complete candidate.
            .filter(|name| {
                **name != "aria-label"
                    && name.starts_with(prefix)
                    && (!assigned_aria || **name == authored_attribute)
            })
            .map(|name| attribute(ctx, tag, prefix, authored, name, "ARIA attribute"))
            .collect()
    };
    for prop in component {
        if candidates.is_empty() {
            break;
        }
        if prop.kind == Some(CompletionItemKind::PROPERTY) {
            // Normalize each declared name once, including bound camel-case
            // props, rather than once per contextual ARIA candidate.
            let kebab = pascal_to_kebab(&prop.label);
            candidates
                .retain(|attribute| attribute.label != prop.label && attribute.label != kebab);
        }
    }
    candidates
}

fn data_attribute(
    ctx: &IdeContext<'_>,
    tag: &OpenTagContext,
    prefix: &str,
    raw_name: &str,
) -> CompletionItem {
    let name = attribute_name(raw_name);
    if name.starts_with("data-") && name.len() > "data-".len() {
        attribute(ctx, tag, prefix, raw_name, name, "Custom data attribute")
    } else {
        items::attr_item("data-*", "Custom data attribute", "data-${1:name}=\"$2\"")
    }
}

fn attribute(
    ctx: &IdeContext<'_>,
    tag: &OpenTagContext,
    prefix: &str,
    raw_name: &str,
    name: &str,
    detail: &str,
) -> CompletionItem {
    let mut item = items::attr_item(name, detail, &format!("{name}=\"$1\""));
    let token_end = tag.current_token_start + raw_name.len();
    if attribute_name(raw_name) == name
        && (ctx.offset < token_end
            || ctx
                .content
                .get(token_end..)
                .is_some_and(|s| s.trim_start().starts_with('=')))
    {
        // The shared opening-tag editor replaces the typed prefix only. Keep
        // an existing suffix and assignment intact when this is the already
        // authored attribute, including the original `dat|a-role="x"` case.
        item.insert_text = Some(prefix.to_string());
    }
    item
}

fn attribute_name(token: &str) -> &str {
    token
        .strip_prefix("v-bind:")
        .or_else(|| token.strip_prefix(':'))
        .unwrap_or(token)
}

fn authored_name<'a>(ctx: &'a IdeContext<'_>, tag: &OpenTagContext) -> &'a str {
    ctx.content
        .get(tag.current_token_start..)
        .unwrap_or_default()
        .split(|ch: char| {
            ch.is_whitespace() || matches!(ch, '=' | '>' | '/' | '(' | ')' | ',' | '"' | '\'')
        })
        .next()
        .unwrap_or_default()
}

// Match the current repository's accepted ARIA attribute vocabulary, including
// its retained deprecated and draft entries, without a linter dependency.
const ARIA_ATTRIBUTES: &[&str] = &[
    "aria-atomic",
    "aria-busy",
    "aria-controls",
    "aria-current",
    "aria-describedby",
    "aria-description",
    "aria-details",
    "aria-disabled",
    "aria-dropeffect",
    "aria-errormessage",
    "aria-flowto",
    "aria-grabbed",
    "aria-haspopup",
    "aria-hidden",
    "aria-invalid",
    "aria-keyshortcuts",
    "aria-label",
    "aria-labelledby",
    "aria-live",
    "aria-owns",
    "aria-relevant",
    "aria-roledescription",
    "aria-autocomplete",
    "aria-checked",
    "aria-expanded",
    "aria-level",
    "aria-modal",
    "aria-multiline",
    "aria-multiselectable",
    "aria-orientation",
    "aria-placeholder",
    "aria-pressed",
    "aria-readonly",
    "aria-required",
    "aria-selected",
    "aria-sort",
    "aria-valuemax",
    "aria-valuemin",
    "aria-valuenow",
    "aria-valuetext",
    "aria-activedescendant",
    "aria-colcount",
    "aria-colindex",
    "aria-colindextext",
    "aria-colspan",
    "aria-posinset",
    "aria-rowcount",
    "aria-rowindex",
    "aria-rowindextext",
    "aria-rowspan",
    "aria-setsize",
    "aria-braillelabel",
    "aria-brailleroledescription",
];
