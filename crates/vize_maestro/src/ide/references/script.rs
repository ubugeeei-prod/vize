//! Script and style reference finding.
//!
//! Finds references in script blocks (both setup and regular),
//! style v-bind() expressions, and definition locations.

use tower_lsp::lsp_types::{Location, Position, Range};

use super::{IdeContext, ReferencesService};
use vize_s0::cstr;

impl ReferencesService {
    /// Find the definition location of a symbol.
    pub(super) fn find_definition_location(ctx: &IdeContext, word: &str) -> Option<Location> {
        let options = vize_atelier_sfc::SfcParseOptions::default();
        let descriptor = vize_atelier_sfc::parse_sfc(&ctx.content, options).ok()?;

        if ctx.state.options_api_enabled()
            && let Some(location) =
                crate::ide::definition::script::find_analyzed_binding_location(ctx, word)
        {
            return Some(location);
        }

        if let Some(ref script_setup) = descriptor.script_setup
            && let Some(loc) = Self::find_binding_in_script(&script_setup.content, word)
        {
            return Some(Self::location_from_sfc_offset(
                ctx,
                script_setup.loc.start + loc,
                word,
            ));
        }

        if let Some(ref script) = descriptor.script
            && let Some(loc) = Self::find_binding_in_script(&script.content, word)
        {
            return Some(Self::location_from_sfc_offset(
                ctx,
                script.loc.start + loc,
                word,
            ));
        }

        None
    }

    pub(super) fn location_from_sfc_offset(
        ctx: &IdeContext,
        offset: usize,
        word: &str,
    ) -> Location {
        let (line, character) = crate::ide::offset_to_position(&ctx.content, offset);

        Location {
            uri: ctx.uri.clone(),
            range: Range {
                start: Position { line, character },
                end: Position {
                    line,
                    character: character + word.encode_utf16().count() as u32,
                },
            },
        }
    }

    /// Find references to a symbol in the script block.
    ///
    /// Occurrences are collected as byte offsets inside the block content and
    /// rebased onto the authored SFC through `loc.start`, the mapping `rename`
    /// already uses. Deriving a position from block-relative *line numbers*
    /// instead placed every script hit one line below the authored one and
    /// could overrun the end of the line it landed on (#3325).
    pub(super) fn find_references_in_script(ctx: &IdeContext, word: &str) -> Vec<Location> {
        let mut locations = Vec::new();

        let options = vize_atelier_sfc::SfcParseOptions::default();
        let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&ctx.content, options) else {
            return locations;
        };

        let blocks = [descriptor.script_setup.as_ref(), descriptor.script.as_ref()];
        for block in blocks.into_iter().flatten() {
            for offset in Self::find_identifier_references_in_script(&block.content, word) {
                locations.push(Self::location_from_sfc_offset(
                    ctx,
                    block.loc.start + offset,
                    word,
                ));
            }
        }

        locations
    }

    /// Find references to a symbol in style blocks (v-bind).
    pub(in crate::ide) fn find_references_in_style(ctx: &IdeContext, word: &str) -> Vec<Location> {
        let mut locations = Vec::new();

        let options = vize_atelier_sfc::SfcParseOptions::default();
        let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&ctx.content, options) else {
            return locations;
        };

        for style in &descriptor.styles {
            for offset in Self::find_vbind_references_in_style(&style.content, word) {
                locations.push(Self::location_from_sfc_offset(
                    ctx,
                    style.loc.start + offset,
                    word,
                ));
            }
        }

        locations
    }

    /// Byte offsets, relative to the block content, of every standalone
    /// occurrence of `word` in script code.
    pub(super) fn find_identifier_references_in_script(content: &str, word: &str) -> Vec<usize> {
        Self::find_word_occurrences(content, word)
    }

    /// Byte offsets, relative to the block content, of every `v-bind()`
    /// argument that names `word`.
    pub(super) fn find_vbind_references_in_style(content: &str, word: &str) -> Vec<usize> {
        let mut refs = Vec::new();
        let mut line_start = 0usize;

        for line in content.split_inclusive('\n') {
            let mut search_start = 0;
            while let Some(relative_vbind_pos) = line[search_start..].find("v-bind(") {
                let vbind_pos = search_start + relative_vbind_pos;
                let argument = &line[vbind_pos + 7..];
                let Some(close_paren) = argument.find(')') else {
                    break;
                };
                let raw = &argument[..close_paren];
                if raw.trim() == word {
                    let leading = raw.len() - raw.trim_start().len();
                    refs.push(line_start + vbind_pos + 7 + leading);
                }
                search_start = vbind_pos + 7 + close_paren + 1;
            }
            line_start += line.len();
        }

        refs
    }

    /// Find a binding definition in script content.
    pub(super) fn find_binding_in_script(content: &str, name: &str) -> Option<usize> {
        if let Some(offset) = Self::find_import_binding(content, name) {
            return Some(offset);
        }
        let content_start = Self::skip_virtual_header(content);
        let search_content = &content[content_start..];

        let patterns = [
            cstr!("const {name} "),
            cstr!("const {name}="),
            cstr!("let {name} "),
            cstr!("let {name}="),
            cstr!("var {name} "),
            cstr!("var {name}="),
            cstr!("function {name}("),
            cstr!("function {name} ("),
        ];

        for pattern in &patterns {
            if let Some(pos) = search_content.find(pattern.as_str()) {
                let name_offset = pattern.find(name).unwrap_or(0);
                return Some(content_start + pos + name_offset);
            }
        }

        // Check destructuring
        let destructure_patterns = [
            cstr!("{{ {name}"),
            cstr!("{{ {name}, "),
            cstr!("{{ {name} }}"),
            cstr!(", {name} }}"),
            cstr!(", {name}, "),
        ];

        for pattern in &destructure_patterns {
            if let Some(pos) = search_content.find(pattern.as_str()) {
                let name_offset = pattern.find(name).unwrap_or(0);
                return Some(content_start + pos + name_offset);
            }
        }

        None
    }

    fn find_import_binding(content: &str, name: &str) -> Option<usize> {
        use oxc_ast::ast::{ImportDeclarationSpecifier, Statement};

        let allocator = oxc_allocator::Allocator::default();
        let parsed = oxc_parser::Parser::new(
            &allocator,
            content,
            oxc_span::SourceType::ts().with_module(true),
        )
        .parse();
        let parsed = if parsed.panicked {
            oxc_parser::Parser::new(
                &allocator,
                content,
                oxc_span::SourceType::tsx().with_module(true),
            )
            .parse()
        } else {
            parsed
        };
        if parsed.panicked {
            return None;
        }

        parsed.program.body.iter().find_map(|statement| {
            let Statement::ImportDeclaration(import) = statement else {
                return None;
            };
            import.specifiers.as_ref()?.iter().find_map(|specifier| {
                let local = match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier) => &specifier.local,
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                        &specifier.local
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        &specifier.local
                    }
                };
                (local.name == name).then_some(local.span.start as usize)
            })
        })
    }

    /// Skip virtual code header.
    fn skip_virtual_header(content: &str) -> usize {
        let mut offset = 0;
        for line in content.lines() {
            if line.starts_with("//") || line.trim().is_empty() {
                offset += line.len() + 1;
            } else {
                break;
            }
        }
        offset
    }
}

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::Url;

    use crate::ide::{IdeContext, ReferencesService};
    use crate::server::ServerState;

    /// Authored 0-based lines: 0 `<script setup>`, 3 `const visitCount`,
    /// 4 `const doubled`, 5 `</script>`, 8 the `@click` handler.
    const SOURCE: &str = r#"<script setup lang="ts">
import { computed, ref } from 'vue'

const visitCount = ref(0)
const doubled = computed(() => visitCount.value * 2)
</script>

<template>
  <button @click="visitCount++">bump</button>
</template>
"#;

    fn reference_spans(source: &str, cursor_text: &str) -> Vec<(u32, u32, u32)> {
        spans(source, cursor_text, true)
    }

    fn spans(source: &str, cursor_text: &str, include_declaration: bool) -> Vec<(u32, u32, u32)> {
        let state = ServerState::new();
        let uri = Url::parse("file:///App.vue").unwrap();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        let offset = source.find(cursor_text).unwrap();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();

        ReferencesService::references(&ctx, include_declaration)
            .unwrap()
            .into_iter()
            .map(|location| {
                assert_eq!(location.uri, uri);
                (
                    location.range.start.line,
                    location.range.start.character,
                    location.range.end.character,
                )
            })
            .collect()
    }

    /// #3325: script hits were rebased from block-relative line numbers, so
    /// they answered one line below the authored occurrence — `5:31-41` even
    /// overran the 9-character `</script>` line while the real use at `4:31-41`
    /// went missing.
    #[test]
    fn script_references_land_on_authored_ranges() {
        assert_eq!(
            reference_spans(SOURCE, "visitCount = ref"),
            [(3, 6, 16), (4, 31, 41), (8, 18, 28)],
        );
    }

    /// The block scan reports the declaration site too, so once it maps to the
    /// authored span `includeDeclaration: false` has to drop it by range.
    #[test]
    fn excluding_the_declaration_drops_only_the_declaration_span() {
        assert_eq!(
            spans(SOURCE, "visitCount = ref", false),
            [(4, 31, 41), (8, 18, 28)],
        );
    }

    #[test]
    fn excluding_declarations_recognizes_imported_local_bindings() {
        let source = "<script setup>\nimport { shared } from './Child.vue'\nconst local = shared\n</script>\n<template>{{ shared }}</template>\n";

        assert_eq!(
            spans(source, "shared } from", false),
            [(2, 14, 20), (4, 13, 19)],
        );
    }

    /// A style `v-bind()` argument maps through the same block offset rebase.
    #[test]
    fn style_vbind_references_land_on_authored_ranges() {
        let source = "<script setup>\nconst textColor = 'red'\n</script>\n\n<style>\n.a {\n  color: v-bind(textColor);\n}\n</style>\n";

        assert_eq!(
            reference_spans(source, "textColor = "),
            [(1, 6, 15), (6, 16, 25)],
        );
    }
}
