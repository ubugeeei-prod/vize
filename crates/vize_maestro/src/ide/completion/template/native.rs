//! Native HTML attribute completions for template opening tags.

use tower_lsp::lsp_types::CompletionItem;

use crate::ide::completion::items;
use crate::ide::{IdeContext, is_component_tag};

use super::tag_context::{is_prop_completion_prefix, opening_tag_context_at_offset};

pub(super) fn native_element_attribute_completions(ctx: &IdeContext) -> Vec<CompletionItem> {
    let Some(tag_ctx) = opening_tag_context_at_offset(&ctx.content, ctx.offset) else {
        return Vec::new();
    };
    if tag_ctx.inside_attribute_value
        || is_component_tag(&tag_ctx.tag_name)
        || !is_prop_completion_prefix(&tag_ctx.current_token)
    {
        return Vec::new();
    }

    let mut items = common_attribute_completions();
    items.extend(tag_attribute_completions(&tag_ctx.tag_name));
    items
}

fn common_attribute_completions() -> Vec<CompletionItem> {
    [
        ("id", "HTML id", "id=\"$1\""),
        ("class", "CSS class", "class=\"$1\""),
        ("style", "Inline style", "style=\"$1\""),
        ("title", "Tooltip title", "title=\"$1\""),
        ("role", "ARIA role", "role=\"$1\""),
        ("tabindex", "Tab order", "tabindex=\"$1\""),
        ("aria-label", "Accessible label", "aria-label=\"$1\""),
        ("ref", "Template ref", "ref=\"$1\""),
        ("key", "VNode key", ":key=\"$1\""),
    ]
    .into_iter()
    .map(|(label, detail, snippet)| items::attr_item(label, detail, snippet))
    .collect()
}

fn tag_attribute_completions(tag_name: &str) -> Vec<CompletionItem> {
    match tag_name {
        "button" => attrs(&[
            ("type", "Button type", "type=\"$1\""),
            ("disabled", "Disabled state", "disabled"),
            ("name", "Form control name", "name=\"$1\""),
            ("value", "Submitted value", "value=\"$1\""),
            ("autofocus", "Autofocus", "autofocus"),
            ("form", "Associated form id", "form=\"$1\""),
        ]),
        "input" => attrs(&[
            ("type", "Input type", "type=\"$1\""),
            ("value", "Input value", "value=\"$1\""),
            ("placeholder", "Placeholder text", "placeholder=\"$1\""),
            ("name", "Form control name", "name=\"$1\""),
            ("disabled", "Disabled state", "disabled"),
            ("checked", "Checked state", "checked"),
            ("required", "Required input", "required"),
            ("autocomplete", "Autocomplete hint", "autocomplete=\"$1\""),
        ]),
        "a" => attrs(&[
            ("href", "Link target", "href=\"$1\""),
            ("target", "Browsing context", "target=\"$1\""),
            ("rel", "Link relationship", "rel=\"$1\""),
            ("download", "Download target", "download"),
        ]),
        "form" => attrs(&[
            ("action", "Submit URL", "action=\"$1\""),
            ("method", "Submit method", "method=\"$1\""),
            ("novalidate", "Skip validation", "novalidate"),
        ]),
        "img" => attrs(&[
            ("src", "Image source", "src=\"$1\""),
            ("alt", "Alternative text", "alt=\"$1\""),
            ("width", "Image width", "width=\"$1\""),
            ("height", "Image height", "height=\"$1\""),
            ("loading", "Loading strategy", "loading=\"$1\""),
        ]),
        "label" => attrs(&[("for", "Associated control id", "for=\"$1\"")]),
        "textarea" => attrs(&[
            ("name", "Form control name", "name=\"$1\""),
            ("placeholder", "Placeholder text", "placeholder=\"$1\""),
            ("disabled", "Disabled state", "disabled"),
            ("required", "Required input", "required"),
            ("rows", "Visible rows", "rows=\"$1\""),
            ("cols", "Visible columns", "cols=\"$1\""),
        ]),
        "select" => attrs(&[
            ("name", "Form control name", "name=\"$1\""),
            ("disabled", "Disabled state", "disabled"),
            ("required", "Required input", "required"),
            ("multiple", "Multiple selection", "multiple"),
        ]),
        _ => Vec::new(),
    }
}

fn attrs(values: &[(&str, &str, &str)]) -> Vec<CompletionItem> {
    values
        .iter()
        .map(|(label, detail, snippet)| items::attr_item(label, detail, snippet))
        .collect()
}
