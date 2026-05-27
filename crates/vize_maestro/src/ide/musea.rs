//! Musea-specific editor helpers.

use std::fs;
use std::path::{Path, PathBuf};

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionTextEdit, Documentation, MarkupContent,
    MarkupKind, Position, Range, TextEdit, Url,
};

use crate::ide::{IdeContext, offset_to_position};

#[derive(Debug, Clone)]
pub(crate) struct DefineArtSource {
    pub source: String,
    pub component_name: String,
    pub literal_start: usize,
    pub literal_end: usize,
    pub value_start: usize,
    pub value_end: usize,
}

pub(crate) fn define_art_sources(content: &str, uri: &Url) -> Vec<DefineArtSource> {
    let options = vize_atelier_sfc::SfcParseOptions {
        filename: uri.path().to_string().into(),
        ..Default::default()
    };
    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(content, options) else {
        return Vec::new();
    };
    let Some(script_setup) = descriptor.script_setup.as_ref() else {
        return Vec::new();
    };

    let parsed = vize_croquis::script_parser::parse_script_setup(script_setup.content.as_ref());
    let Some(art) = parsed.macros.define_art() else {
        return Vec::new();
    };
    let Some(source) = art.component_source.as_ref() else {
        return Vec::new();
    };
    let Some((literal_start, literal_end)) = art.component_source_span else {
        return Vec::new();
    };
    let Some((value_start, value_end)) = art.component_source_value_span else {
        return Vec::new();
    };

    vec![DefineArtSource {
        source: source.to_string(),
        component_name: art.component_name.to_string(),
        literal_start: script_setup.loc.start + literal_start as usize,
        literal_end: script_setup.loc.start + literal_end as usize,
        value_start: script_setup.loc.start + value_start as usize,
        value_end: script_setup.loc.start + value_end as usize,
    }]
}

pub(crate) fn define_art_source_at_offset(
    content: &str,
    uri: &Url,
    offset: usize,
) -> Option<DefineArtSource> {
    define_art_sources(content, uri).into_iter().find(|source| {
        source.literal_start <= offset
            && offset <= source.literal_end
            && source.value_start <= offset
            && offset <= source.value_end
    })
}

pub(crate) fn resolve_define_art_source(uri: &Url, source: &str) -> Option<PathBuf> {
    if !should_check_define_art_source(source) {
        return None;
    }

    let current_path = uri.to_file_path().ok()?;
    let current_dir = current_path.parent()?;
    let candidate = if source.starts_with('/') {
        PathBuf::from(source)
    } else {
        current_dir.join(source)
    };

    resolve_existing_path(&candidate)
}

pub(crate) fn should_check_define_art_source(source: &str) -> bool {
    source.starts_with("./") || source.starts_with("../") || source.starts_with('/')
}

pub(crate) fn range_for_offsets(content: &str, start: usize, end: usize) -> Range {
    let (start_line, start_character) = offset_to_position(content, start.min(content.len()));
    let (end_line, end_character) = offset_to_position(content, end.min(content.len()));
    Range {
        start: Position {
            line: start_line,
            character: start_character,
        },
        end: Position {
            line: end_line,
            character: end_character,
        },
    }
}

pub(crate) fn define_art_source_completions(ctx: &IdeContext<'_>) -> Option<Vec<CompletionItem>> {
    let source = define_art_source_at_offset(&ctx.content, ctx.uri, ctx.offset)?;
    let current_path = ctx.uri.to_file_path().ok()?;
    let base_dir = current_path.parent()?;
    let typed = ctx.content.get(source.value_start..ctx.offset)?;
    let mut items = component_source_path_completions(
        &ctx.content,
        base_dir,
        typed,
        source.value_start,
        ctx.offset,
    );

    if items.is_empty() {
        None
    } else {
        Some({
            items.sort_by(|a, b| a.label.cmp(&b.label));
            items
        })
    }
}

fn component_source_path_completions(
    content: &str,
    base_dir: &Path,
    typed: &str,
    replace_start: usize,
    replace_end: usize,
) -> Vec<CompletionItem> {
    let normalized = typed.replace('\\', "/");
    let replace_range = range_for_offsets(content, replace_start, replace_end);

    let mut items = Vec::new();
    if normalized.is_empty() || normalized == "." {
        items.push(path_completion_item(
            "./",
            CompletionItemKind::FOLDER,
            replace_range,
            "Current directory",
        ));
        items.push(path_completion_item(
            "../",
            CompletionItemKind::FOLDER,
            replace_range,
            "Parent directory",
        ));
    }

    let (directory_prefix, filename_prefix) =
        split_path_completion_prefix(&normalized).unwrap_or(("", normalized.as_str()));
    let lookup_dir = if directory_prefix.starts_with('/') {
        PathBuf::from(directory_prefix)
    } else if directory_prefix.is_empty() {
        base_dir.to_path_buf()
    } else {
        base_dir.join(directory_prefix)
    };

    let Ok(entries) = fs::read_dir(lookup_dir) else {
        return items;
    };

    let insert_prefix = if directory_prefix.is_empty() && !normalized.starts_with('.') {
        "./"
    } else {
        directory_prefix
    };

    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || !name.starts_with(filename_prefix) {
            continue;
        }

        if file_type.is_dir() {
            let mut new_text = String::with_capacity(insert_prefix.len() + name.len() + 1);
            new_text.push_str(insert_prefix);
            new_text.push_str(&name);
            new_text.push('/');
            items.push(path_completion_item(
                &new_text,
                CompletionItemKind::FOLDER,
                replace_range,
                "Component directory",
            ));
        } else if file_type.is_file()
            && entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "vue")
        {
            let mut new_text = String::with_capacity(insert_prefix.len() + name.len());
            new_text.push_str(insert_prefix);
            new_text.push_str(&name);
            items.push(path_completion_item(
                &new_text,
                CompletionItemKind::FILE,
                replace_range,
                "Vue component",
            ));
        }
    }

    items
}

fn split_path_completion_prefix(path: &str) -> Option<(&str, &str)> {
    let slash = path.rfind('/')?;
    Some((&path[..slash + 1], &path[slash + 1..]))
}

fn path_completion_item(
    new_text: &str,
    kind: CompletionItemKind,
    range: Range,
    detail: &str,
) -> CompletionItem {
    let mut documentation = String::from("Use `");
    documentation.push_str(new_text);
    documentation.push_str("` as the `defineArt` component source.");

    CompletionItem {
        label: new_text.to_string(),
        kind: Some(kind),
        detail: Some(detail.to_string()),
        text_edit: Some(CompletionTextEdit::Edit(TextEdit {
            range,
            new_text: new_text.to_string(),
        })),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: documentation,
        })),
        ..Default::default()
    }
}

fn resolve_existing_path(path: &Path) -> Option<PathBuf> {
    if path.exists() {
        return Some(path.canonicalize().unwrap_or_else(|_| path.to_path_buf()));
    }

    if path.extension().is_none() {
        let vue = path.with_extension("vue");
        if vue.exists() {
            return Some(vue.canonicalize().unwrap_or(vue));
        }
    }

    let index_vue = path.join("index.vue");
    if index_vue.exists() {
        return Some(index_vue.canonicalize().unwrap_or(index_vue));
    }

    None
}
