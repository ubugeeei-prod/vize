//! Script and style reference finding.
//!
//! Finds references in script blocks (both setup and regular),
//! style v-bind() expressions, and definition locations.

#[cfg(feature = "native")]
use tower_lsp::lsp_types::{Location, Position, Range};

#[cfg(feature = "native")]
use super::{IdeContext, ReferencesService};

#[cfg(feature = "native")]
impl ReferencesService {
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
