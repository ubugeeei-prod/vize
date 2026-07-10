//! SFC resource-path rewriting for file renames.

use std::path::Path;

use tower_lsp::lsp_types::TextEdit;

use super::manual::{RenameTarget, offset_range, rewrite_relative_specifier};
use crate::server::ServerState;

pub(super) fn collect_vue_resource_edits(
    state: &ServerState,
    path: &Path,
    future_path: &Path,
    source: &str,
    rename_targets: &[RenameTarget],
) -> Vec<TextEdit> {
    let options = vize_atelier_sfc::SfcParseOptions {
        filename: path.to_string_lossy().to_string().into(),
        ..Default::default()
    };
    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(source, options) else {
        return Vec::new();
    };
    let Some(current_dir) = path.parent() else {
        return Vec::new();
    };
    let Some(future_dir) = future_path.parent() else {
        return Vec::new();
    };

    let mut edits = Vec::new();
    let mut push_edit = |specifier: &str, start: usize, end: usize| {
        let Some(new_text) =
            rewrite_relative_specifier(state, current_dir, future_dir, specifier, rename_targets)
        else {
            return;
        };
        if new_text == specifier {
            return;
        }
        if let Some(range) = offset_range(source, start, end) {
            edits.push(TextEdit { range, new_text });
        }
    };

    if let Some(template) = descriptor.template.as_ref()
        && let Some(src) = template.src.as_deref()
        && let Some((start, end)) = find_src_attr_range(source, template.loc.tag_start)
    {
        push_edit(src, start, end);
    }
    if let Some(script) = descriptor.script.as_ref()
        && let Some(src) = script.src.as_deref()
        && let Some((start, end)) = find_src_attr_range(source, script.loc.tag_start)
    {
        push_edit(src, start, end);
    }
    if let Some(script_setup) = descriptor.script_setup.as_ref()
        && let Some(src) = script_setup.src.as_deref()
        && let Some((start, end)) = find_src_attr_range(source, script_setup.loc.tag_start)
    {
        push_edit(src, start, end);
    }
    for style in &descriptor.styles {
        if let Some(src) = style.src.as_deref()
            && let Some((start, end)) = find_src_attr_range(source, style.loc.tag_start)
        {
            push_edit(src, start, end);
        }
        collect_css_imports(style.content.as_ref(), style.loc.start, &mut push_edit);
    }

    edits
}

fn collect_css_imports(
    css: &str,
    base_offset: usize,
    push_edit: &mut impl FnMut(&str, usize, usize),
) {
    let mut pos = 0;
    while let Some(import_pos) = css[pos..].find("@import") {
        let start = pos + import_pos;
        let after_import = &css[start + 7..];
        let trimmed = after_import.trim_start();
        let ws_len = after_import.len() - trimmed.len();

        if let Some(url_content) = trimmed.strip_prefix("url(") {
            let inner = url_content.trim_start();
            let inner_ws = url_content.len() - inner.len();
            if let Some((specifier, rel_start, rel_end)) = extract_css_specifier(inner) {
                let abs_start = base_offset + start + 7 + ws_len + 4 + inner_ws + rel_start;
                let abs_end = base_offset + start + 7 + ws_len + 4 + inner_ws + rel_end;
                push_edit(specifier, abs_start, abs_end);
            }
        } else if let Some((specifier, rel_start, rel_end)) = extract_css_specifier(trimmed) {
            let abs_start = base_offset + start + 7 + ws_len + rel_start;
            let abs_end = base_offset + start + 7 + ws_len + rel_end;
            push_edit(specifier, abs_start, abs_end);
        }

        pos = start + 8;
    }
}

fn extract_css_specifier(text: &str) -> Option<(&str, usize, usize)> {
    let bytes = text.as_bytes();
    let first = *bytes.first()?;
    if first == b'"' || first == b'\'' {
        let quote = first as char;
        let end = text[1..].find(quote)? + 1;
        return Some((&text[1..end], 1, end));
    }

    let end = text.find([')', ';', '\n']).unwrap_or(text.len());
    let specifier = text[..end].trim_end();
    if specifier.is_empty() {
        None
    } else {
        Some((specifier, 0, specifier.len()))
    }
}

fn find_src_attr_range(content: &str, tag_start: usize) -> Option<(usize, usize)> {
    let tag = content.get(tag_start..)?.split_once('>')?.0;
    let src_pos = tag.find("src=")?;
    let after_src = &tag[src_pos + 4..];
    let quote = after_src.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let value_start = src_pos + 5;
    let value_end = after_src[1..].find(quote)?;
    Some((tag_start + value_start, tag_start + value_start + value_end))
}

#[cfg(test)]
#[allow(clippy::disallowed_macros)]
mod tests {
    use super::{collect_css_imports, find_src_attr_range};

    #[test]
    fn css_import_ranges_cover_only_the_specifier_text() {
        let css =
            "@import \"./base.css\";\n@import url('./theme.css');\n@import url(../reset.css);\n";
        let base_offset = 11;
        let mut imports = Vec::new();

        collect_css_imports(css, base_offset, &mut |specifier, start, end| {
            imports.push((
                specifier.to_string(),
                css[start - base_offset..end - base_offset].to_string(),
            ));
        });

        assert_eq!(
            imports,
            [
                ("./base.css".to_string(), "./base.css".to_string()),
                ("./theme.css".to_string(), "./theme.css".to_string()),
                ("../reset.css".to_string(), "../reset.css".to_string()),
            ]
        );
    }

    #[test]
    fn src_attribute_ranges_cover_quoted_values_only() {
        let source = "<template lang=\"pug\" src='./partial.pug'></template>\n<script setup src=\"./entry.ts\"></script>";
        let template_range = find_src_attr_range(source, 0).unwrap();
        let script_range = find_src_attr_range(source, source.find("<script").unwrap()).unwrap();

        assert_eq!(&source[template_range.0..template_range.1], "./partial.pug");
        assert_eq!(&source[script_range.0..script_range.1], "./entry.ts");
        assert_eq!(
            find_src_attr_range("<style src=./style.css></style>", 0),
            None
        );
    }
}
