use tower_lsp::lsp_types::{Hover, HoverContents, MarkedString};

pub(super) fn hover_has_unknown_reactive_type(hover: &Hover) -> bool {
    let value = match &hover.contents {
        HoverContents::Markup(markup) => markup.value.as_str(),
        HoverContents::Scalar(value) => return marked_string_has_unknown_reactive_type(value),
        HoverContents::Array(values) => {
            return values.iter().any(marked_string_has_unknown_reactive_type);
        }
    };
    value.contains("Ref<unknown>") || value.contains("ComputedRef<unknown>")
}

fn marked_string_has_unknown_reactive_type(value: &MarkedString) -> bool {
    let value = match value {
        MarkedString::String(value) => value.as_str(),
        MarkedString::LanguageString(value) => value.value.as_str(),
    };
    value.contains("Ref<unknown>") || value.contains("ComputedRef<unknown>")
}
