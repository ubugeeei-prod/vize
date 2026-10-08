//! Prefix filtering precedes item/Markdown allocation; documentation is lazy.

use super::{
    context::{self, CssContext},
    data, documentation,
    resolve::Selected,
    values, vue,
};
use crate::ide::{IdeContext, markup::markdown_documentation};
use tower_lsp::lsp_types::{CompletionItem, CompletionTextEdit, InsertTextFormat, Range, TextEdit};

pub(crate) fn complete(ctx: &IdeContext<'_>, index: usize) -> Vec<CompletionItem> {
    let lazy = ctx.state.supports_completion_documentation_resolve();
    let mut items = vue_completions(lazy);
    let Some((content, base)) = super::region(ctx, index) else {
        return items;
    };
    let (candidates, span, has_colon) = match context::at(content, ctx.offset - base) {
        CssContext::Property {
            prefix,
            span,
            has_colon,
        } => {
            let edit_range = super::range(ctx, base, span);
            for entry in data::properties()
                .iter()
                .filter(|entry| super::starts_with(entry.name, prefix))
            {
                push(
                    &mut items,
                    Selected::Entry("property", entry),
                    edit_range,
                    has_colon,
                    lazy,
                );
            }
            return items;
        }
        CssContext::Value {
            property,
            prefix,
            span,
        } => {
            if let Some(property) = data::property(property) {
                let edit_range = super::range(ctx, base, span);
                for value in property
                    .values
                    .iter()
                    .filter(|value| super::starts_with(value.name, prefix))
                {
                    push(
                        &mut items,
                        Selected::Value(property, value),
                        edit_range,
                        false,
                        lazy,
                    );
                }
                for value in values::WIDE.iter().filter(|value| {
                    super::starts_with(value.name, prefix)
                        && !property.values.iter().any(|owned| owned.name == value.name)
                }) {
                    push(
                        &mut items,
                        Selected::Wide(property, value),
                        edit_range,
                        false,
                        lazy,
                    );
                }
                if values::accepts_colors(property) {
                    for value in data::colors().iter().filter(|value| {
                        super::starts_with(value.name, prefix)
                            && !property.values.iter().any(|owned| owned.name == value.name)
                    }) {
                        push(
                            &mut items,
                            Selected::Color(property, value),
                            edit_range,
                            false,
                            lazy,
                        );
                    }
                }
                for value in values::FUNCTIONS.iter().filter(|value| {
                    super::starts_with(value.name, prefix)
                        && (value.name != "calc" || values::accepts_calc(property))
                }) {
                    push(
                        &mut items,
                        Selected::Function(property, value),
                        edit_range,
                        false,
                        lazy,
                    );
                }
            }
            return items;
        }
        CssContext::Pseudo { prefix, span } => {
            let edit_range = super::range(ctx, base, span);
            for entry in data::pseudo_classes()
                .iter()
                .chain(data::pseudo_elements())
                .filter(|entry| super::starts_with(entry.name, prefix))
            {
                push(
                    &mut items,
                    Selected::Entry("pseudo", entry),
                    edit_range,
                    false,
                    lazy,
                );
            }
            return items;
        }
        CssContext::AtRule { span, .. } => (data::at_rules(), span, false),
        CssContext::Unknown => return items,
    };
    let edit_range = super::range(ctx, base, span);
    let prefix = content.get(span.0..ctx.offset - base).unwrap_or_default();
    for entry in candidates
        .iter()
        .filter(|entry| super::starts_with(entry.name, prefix))
    {
        push(
            &mut items,
            Selected::Entry("at-rule", entry),
            edit_range,
            has_colon,
            lazy,
        );
    }
    items
}

pub(crate) fn vue_completions(lazy: bool) -> Vec<CompletionItem> {
    vue::FEATURES
        .iter()
        .map(|feature| {
            let selected = Selected::Vue(feature);
            let mut item = CompletionItem {
                label: feature.name.to_owned(),
                kind: Some(selected.kind()),
                detail: Some(["Vue CSS: ", feature.signature].concat()),
                insert_text: Some(feature.snippet.to_owned()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..CompletionItem::default()
            };
            attach(&mut item, selected, lazy);
            item
        })
        .collect()
}

fn push(
    items: &mut Vec<CompletionItem>,
    selected: Selected,
    edit_range: Range,
    has_colon: bool,
    lazy: bool,
) {
    let label = selected.label();
    let (text, format, detail) = match selected {
        Selected::Entry("property", _) => (
            if has_colon {
                label.to_owned()
            } else {
                [label, ": "].concat()
            },
            InsertTextFormat::PLAIN_TEXT,
            "CSS property",
        ),
        Selected::Entry("pseudo", _) => (
            label.to_owned(),
            InsertTextFormat::PLAIN_TEXT,
            "CSS selector",
        ),
        Selected::Entry(_, _) => (
            label.to_owned(),
            InsertTextFormat::PLAIN_TEXT,
            "CSS at-rule",
        ),
        Selected::Function(_, value) => (
            [value.name, "($1)"].concat(),
            InsertTextFormat::SNIPPET,
            "CSS value function",
        ),
        _ => (label.to_owned(), InsertTextFormat::PLAIN_TEXT, "CSS value"),
    };
    let mut item = CompletionItem {
        label: label.to_owned(),
        kind: Some(selected.kind()),
        detail: Some(detail.to_owned()),
        text_edit: Some(CompletionTextEdit::Edit(TextEdit {
            range: edit_range,
            new_text: text,
        })),
        insert_text_format: Some(format),
        ..CompletionItem::default()
    };
    if let Selected::Entry("property", entry) = selected
        && let Some(relevance) = entry.relevance
    {
        // Rank only new catalog properties; the four authored Vue items are untouched.
        let rank = 100u16.saturating_sub(relevance);
        let key = rank.to_string();
        item.sort_text = Some([if rank < 10 { "0" } else { "" }, key.as_str(), label].concat());
    }
    attach(&mut item, selected, lazy);
    items.push(item);
}

fn attach(item: &mut CompletionItem, selected: Selected, lazy: bool) {
    if lazy {
        item.data = Some(selected.data());
    } else {
        item.documentation = Some(markdown_documentation(documentation::markdown(selected)));
    }
}
