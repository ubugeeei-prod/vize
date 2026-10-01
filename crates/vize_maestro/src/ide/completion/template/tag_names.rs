//! Tag-name completions use authored markup ranges and component scope.
use crate::ide::{IdeContext, offset_to_position};
use std::collections::BTreeMap;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionTextEdit, InsertTextFormat, Position, Range,
    TextEdit,
};
use vize_croquis::naming::hyphenate;
use vize_l0::dom_tag_config::{HTML_TAGS, SVG_TAGS};
use vize_relief::BindingType;

struct TagNameContext<'a> {
    start: usize,
    end: usize,
    prefix: &'a str,
}

fn context<'a>(ctx: &'a IdeContext<'_>) -> Option<TagNameContext<'a>> {
    if crate::ide::is_in_vue_template_expression(&ctx.content, ctx.offset) {
        return None;
    }
    let descriptor = ctx.descriptor()?;
    let template = descriptor.template.as_ref()?;
    let mut offset = template.loc.start;
    let mut opening = None;
    let mut quote = None;
    while offset < ctx.offset {
        let rest = ctx.content.get(offset..)?;
        if quote.is_none() && rest.starts_with("<!--") {
            let end = offset + rest.find("-->")? + 3;
            if end > ctx.offset {
                return None;
            }
            offset = end;
            continue;
        }
        let ch = rest.chars().next()?;
        if let Some(delimiter) = quote {
            if ch == delimiter {
                quote = None;
            }
        } else {
            match ch {
                '<' => opening = Some(offset),
                '>' => opening = None,
                '\'' | '"' if opening.is_some() => quote = Some(ch),
                _ => {}
            }
        }
        offset += ch.len_utf8();
    }
    if quote.is_some() {
        return None;
    }
    let mut start = opening? + 1;
    if ctx.content.as_bytes().get(start) == Some(&b'/') {
        start += 1;
    }
    let prefix = ctx.content.get(start..ctx.offset)?;
    if !prefix.chars().all(is_tag_name_char) {
        return None;
    }
    let end = ctx.offset
        + ctx
            .content
            .get(ctx.offset..)?
            .chars()
            .take_while(|ch| is_tag_name_char(*ch))
            .map(char::len_utf8)
            .sum::<usize>();
    Some(TagNameContext { start, end, prefix })
}

fn is_tag_name_char(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, '-' | '_' | ':' | '.')
}

pub(super) fn complete(
    ctx: &IdeContext<'_>,
    globals: &[vize_l0::String],
) -> Option<Vec<CompletionItem>> {
    let tag = context(ctx)?;
    let mut names = BTreeMap::<vize_l0::String, (CompletionItemKind, &'static str)>::new();
    for name in HTML_TAGS.iter().chain(SVG_TAGS.iter()) {
        names.insert(
            (*name).into(),
            (CompletionItemKind::KEYWORD, "Native element"),
        );
    }
    for item in super::components::builtin_component_completions() {
        names.insert(
            item.label.into(),
            (CompletionItemKind::CLASS, "Vue component"),
        );
    }
    if ctx.state.lsp_features().legacy_vue2 {
        for item in super::components::legacy_vue2_component_completions() {
            names.insert(
                item.label.into(),
                (CompletionItemKind::CLASS, "Vue component"),
            );
        }
    }
    if let Some((croquis, _)) = crate::ide::template_scope::analyze(ctx) {
        if croquis.bindings.is_script_setup {
            for (name, kind) in &croquis.bindings.bindings {
                if matches!(kind, BindingType::Props | BindingType::LiteralConst) {
                    continue;
                }
                if name.chars().next().is_some_and(char::is_uppercase)
                    || crate::ide::definition::component_import::resolve_component_file(ctx, name)
                        .is_some()
                {
                    component(&mut names, name, "Component in script setup");
                }
            }
        }
        for registration in &croquis.component_registrations {
            component(&mut names, &registration.name, "Registered component");
        }
    }
    for name in globals {
        component(&mut names, name, "Global component");
    }
    let (start_line, start_character) = offset_to_position(&ctx.content, tag.start);
    let (end_line, end_character) = offset_to_position(&ctx.content, tag.end);
    let range = Range::new(
        Position::new(start_line, start_character),
        Position::new(end_line, end_character),
    );
    Some(
        names
            .into_iter()
            .filter(|(name, _)| name.starts_with(tag.prefix))
            .map(|(name, (kind, detail))| CompletionItem {
                label: name.as_str().into(),
                kind: Some(kind),
                detail: Some(detail.into()),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range,
                    new_text: name.as_str().into(),
                })),
                sort_text: Some(name.as_str().into()),
                ..Default::default()
            })
            .collect(),
    )
}

fn component(
    names: &mut BTreeMap<vize_l0::String, (CompletionItemKind, &'static str)>,
    name: &str,
    detail: &'static str,
) {
    names.insert(name.into(), (CompletionItemKind::CLASS, detail));
    names.insert(
        hyphenate(name).as_str().into(),
        (CompletionItemKind::CLASS, detail),
    );
}

#[cfg(feature = "native")]
pub(super) async fn complete_with_globals(ctx: &IdeContext<'_>) -> Option<Vec<CompletionItem>> {
    context(ctx)?;
    let globals = ctx.state.global_component_tag_names().await;
    complete(ctx, &globals)
}
