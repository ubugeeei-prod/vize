//! Document link provider.
//!
//! Provides clickable links for:
//! - Import statements in script blocks
//! - src attributes on script/style/template blocks
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]
//! - CSS @import statements

use std::path::Path;

use tower_lsp::lsp_types::{DocumentLink, Position, Range, Url};

use super::offset_to_position;

mod imports;

/// Document link service.
pub struct DocumentLinkService;

impl DocumentLinkService {
    /// Get document links for a file.
    pub fn get_links(content: &str, uri: &Url) -> Vec<DocumentLink> {
        let mut links = Vec::new();

        let options = vize_atelier_sfc::SfcParseOptions {
            filename: uri.path().to_string().into(),
            ..Default::default()
        };

        let Ok(descriptor) = vize_atelier_sfc::parse_sfc(content, options) else {
            return links;
        };

        let base_path = uri.to_file_path().ok();

        if let Some(ref script_setup) = descriptor.script_setup {
            if let Some(ref src) = script_setup.src
                && let Some((start, end)) =
                    Self::find_src_attr_range(content, script_setup.loc.tag_start)
                && let Some(target) = Self::resolve_path(src, base_path.as_deref())
            {
                links.push(Self::create_link(content, start, end, target));
            }

            imports::collect_import_links(
                &script_setup.content,
                script_setup.loc.start,
                content,
                base_path.as_deref(),
                &mut links,
            );
            Self::collect_define_art_source_links(content, uri, &mut links);
        }

        if let Some(ref script) = descriptor.script {
            if let Some(ref src) = script.src
                && let Some((start, end)) = Self::find_src_attr_range(content, script.loc.tag_start)
                && let Some(target) = Self::resolve_path(src, base_path.as_deref())
            {
                links.push(Self::create_link(content, start, end, target));
            }

            imports::collect_import_links(
                &script.content,
                script.loc.start,
                content,
                base_path.as_deref(),
                &mut links,
            );
        }

        if let Some(ref template) = descriptor.template
            && let Some(ref src) = template.src
            && let Some((start, end)) = Self::find_src_attr_range(content, template.loc.tag_start)
            && let Some(target) = Self::resolve_path(src, base_path.as_deref())
        {
            links.push(Self::create_link(content, start, end, target));
        }

        for style in &descriptor.styles {
            if let Some(ref src) = style.src
                && let Some((start, end)) = Self::find_src_attr_range(content, style.loc.tag_start)
                && let Some(target) = Self::resolve_path(src, base_path.as_deref())
            {
                links.push(Self::create_link(content, start, end, target));
            }

            Self::collect_css_import_links(
                &style.content,
                style.loc.start,
                content,
                base_path.as_deref(),
                &mut links,
            );
        }

        links
    }

    fn collect_define_art_source_links(content: &str, uri: &Url, links: &mut Vec<DocumentLink>) {
        for source in crate::ide::musea::define_art_sources(content, uri) {
            let Some(target) = crate::ide::musea::resolve_define_art_source(uri, &source.source)
            else {
                continue;
            };
            let Ok(target) = Url::from_file_path(target) else {
                continue;
            };
            links.push(Self::create_link(
                content,
                source.value_start,
                source.value_end,
                target,
            ));
        }
    }

    /// Collect CSS @import links.
    fn collect_css_import_links(
        css: &str,
        base_offset: usize,
        full_content: &str,
        base_path: Option<&Path>,
        links: &mut Vec<DocumentLink>,
    ) {
        // Match: @import "path" or @import 'path' or @import url("path")
        let mut pos = 0;

        while let Some(import_pos) = css[pos..].find("@import") {
            let start = pos + import_pos;
            let after_import = &css[start + 7..];
            let trimmed = after_import.trim_start();
            let ws_len = after_import.len() - trimmed.len();

            // @import url("path") or @import url('path')
            if let Some(url_content) = trimmed.strip_prefix("url(") {
                if let Some((path, s, e)) =
                    imports::extract_string_literal(url_content.trim_start())
                {
                    let inner_ws = url_content.len() - url_content.trim_start().len();
                    if (path.starts_with('.') || path.starts_with('/'))
                        && let Some(target) = Self::resolve_path(&path, base_path)
                    {
                        let abs_start = base_offset + start + 7 + ws_len + 4 + inner_ws + s;
                        let abs_end = base_offset + start + 7 + ws_len + 4 + inner_ws + e;
                        links.push(Self::create_link(full_content, abs_start, abs_end, target));
                    }
                }
            }
            // @import "path" or @import 'path'
            else if let Some((path, s, e)) = imports::extract_string_literal(trimmed)
                && (path.starts_with('.') || path.starts_with('/'))
                && let Some(target) = Self::resolve_path(&path, base_path)
            {
                let abs_start = base_offset + start + 7 + ws_len + s;
                let abs_end = base_offset + start + 7 + ws_len + e;
                links.push(Self::create_link(full_content, abs_start, abs_end, target));
            }

            pos = start + 8;
        }
    }

    /// Find src attribute range in the opening tag.
    fn find_src_attr_range(content: &str, tag_start: usize) -> Option<(usize, usize)> {
        let tag = content.get(tag_start..)?.split_once('>')?.0;

        // Find src="..." or src='...'
        let src_pos = tag.find("src=")?;
        let after_src = &tag[src_pos + 4..];

        let quote = after_src.chars().next()?;
        if quote != '"' && quote != '\'' {
            return None;
        }

        let value_start = src_pos + 5; // src=" plus quote
        let value_end = after_src[1..].find(quote)?;

        Some((tag_start + value_start, tag_start + value_start + value_end))
    }

    /// Resolve a relative path to an absolute URL.
    fn resolve_path(path: &str, base_path: Option<&Path>) -> Option<Url> {
        let base = base_path?;
        let parent = base.parent()?;

        // Clean up the path
        let clean_path = path.trim_matches(|c| c == '"' || c == '\'');

        let resolved = if let Some(stripped) = clean_path.strip_prefix('/') {
            // Absolute path from project root - try to find it
            // For now, treat as relative to current file's directory
            parent.join(stripped)
        } else {
            parent.join(clean_path)
        };

        // Try common extensions if file doesn't exist
        let candidates = [
            resolved.clone(),
            resolved.with_extension("ts"),
            resolved.with_extension("js"),
            resolved.with_extension("vue"),
            resolved.with_extension("tsx"),
            resolved.with_extension("jsx"),
            resolved.with_extension("mts"),
            resolved.with_extension("cts"),
            resolved.with_extension("mjs"),
            resolved.with_extension("cjs"),
            resolved.join("index.ts"),
            resolved.join("index.js"),
            resolved.join("index.vue"),
            resolved.join("index.mts"),
            resolved.join("index.cts"),
            resolved.join("index.mjs"),
            resolved.join("index.cjs"),
        ];

        for candidate in &candidates {
            if candidate.exists() {
                return Url::from_file_path(candidate.canonicalize().ok()?).ok();
            }
        }

        // Return original path even if it doesn't exist (user might create it)
        Url::from_file_path(&resolved).ok()
    }

    /// Create a document link.
    fn create_link(content: &str, start: usize, end: usize, target: Url) -> DocumentLink {
        let (start_line, start_char) = offset_to_position(content, start);
        let (end_line, end_char) = offset_to_position(content, end);

        DocumentLink {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_char,
                },
                end: Position {
                    line: end_line,
                    character: end_char,
                },
            },
            target: Some(target),
            tooltip: None,
            data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::DocumentLinkService;
    use tower_lsp::lsp_types::Url;

    #[test]
    fn test_define_art_source_link() {
        let dir = tempfile::tempdir().unwrap();
        let component_path = dir.path().join("Button.vue");
        let art_path = dir.path().join("Button.art.vue");
        fs::write(&component_path, "<template />").unwrap();

        let source = r#"<script setup lang="ts">
defineArt("./Button.vue", {
  title: "Button",
});
</script>

<art>
  <variant name="Default"><Button /></variant>
</art>
"#;
        fs::write(&art_path, source).unwrap();
        let uri = Url::from_file_path(&art_path).unwrap();

        let links = DocumentLinkService::get_links(source, &uri);

        assert!(links.iter().any(|link| {
            link.target
                .as_ref()
                .and_then(|target| target.to_file_path().ok())
                .is_some_and(|target| target == component_path.canonicalize().unwrap())
        }));
    }
}
