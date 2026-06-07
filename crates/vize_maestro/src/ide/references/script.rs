//! Script and style reference finding.
//!
//! Finds references in script blocks (both setup and regular),
//! style v-bind() expressions, and definition locations.

use tower_lsp::lsp_types::{Location, Position, Range};

use super::{IdeContext, ReferencesService};
use vize_carton::cstr;

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

    fn location_from_sfc_offset(ctx: &IdeContext, offset: usize, word: &str) -> Location {
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
    pub(super) fn find_references_in_script(ctx: &IdeContext, word: &str) -> Vec<Location> {
        let mut locations = Vec::new();

        let options = vize_atelier_sfc::SfcParseOptions::default();
        let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&ctx.content, options) else {
            return locations;
        };

        // Check script setup
        if let Some(ref script_setup) = descriptor.script_setup {
            let script_content = script_setup.content.as_ref();
            let script_start_line = script_setup.loc.start_line as u32;

            let refs = Self::find_identifier_references_in_script(script_content, word);
            for (line, character) in refs {
                locations.push(Location {
                    uri: ctx.uri.clone(),
                    range: Range {
                        start: Position {
                            line: script_start_line + line - 1,
                            character,
                        },
                        end: Position {
                            line: script_start_line + line - 1,
                            character: character + word.len() as u32,
                        },
                    },
                });
            }
        }

        // Check regular script
        if let Some(ref script) = descriptor.script {
            let script_content = script.content.as_ref();
            let script_start_line = script.loc.start_line as u32;

            let refs = Self::find_identifier_references_in_script(script_content, word);
            for (line, character) in refs {
                locations.push(Location {
                    uri: ctx.uri.clone(),
                    range: Range {
                        start: Position {
                            line: script_start_line + line - 1,
                            character,
                        },
                        end: Position {
                            line: script_start_line + line - 1,
                            character: character + word.len() as u32,
                        },
                    },
                });
            }
        }

        locations
    }

    /// Find references to a symbol in style blocks (v-bind).
    pub(super) fn find_references_in_style(ctx: &IdeContext, word: &str) -> Vec<Location> {
        let mut locations = Vec::new();

        let options = vize_atelier_sfc::SfcParseOptions::default();
        let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&ctx.content, options) else {
            return locations;
        };

        for style in &descriptor.styles {
            let style_content = style.content.as_ref();
            let style_start_line = style.loc.start_line as u32;

            // Find v-bind() references
            let refs = Self::find_vbind_references_in_style(style_content, word);
            for (line, character) in refs {
                locations.push(Location {
                    uri: ctx.uri.clone(),
                    range: Range {
                        start: Position {
                            line: style_start_line + line - 1,
                            character,
                        },
                        end: Position {
                            line: style_start_line + line - 1,
                            character: character + word.len() as u32,
                        },
                    },
                });
            }
        }

        locations
    }

    /// Find identifier references in script content.
    pub(super) fn find_identifier_references_in_script(
        content: &str,
        word: &str,
    ) -> Vec<(u32, u32)> {
        let mut refs = Vec::new();

        for (line_idx, line) in content.lines().enumerate() {
            let positions = Self::find_word_occurrences(line, word);

            for pos in positions {
                refs.push((
                    line_idx as u32 + 1,
                    crate::ide::offset_to_position(line, pos).1,
                ));
            }
        }

        refs
    }

    /// Find v-bind references in style content.
    pub(super) fn find_vbind_references_in_style(content: &str, word: &str) -> Vec<(u32, u32)> {
        let mut refs = Vec::new();

        for (line_idx, line) in content.lines().enumerate() {
            let mut search_start = 0;
            while let Some(relative_vbind_pos) = line[search_start..].find("v-bind(") {
                let vbind_pos = search_start + relative_vbind_pos;
                let after_vbind = &line[vbind_pos + 7..];
                if let Some(close_paren) = after_vbind.find(')') {
                    let binding_name = after_vbind[..close_paren].trim();
                    if binding_name == word {
                        let binding_offset =
                            vbind_pos + 7 + (binding_name.len() - binding_name.trim_start().len());
                        refs.push((
                            line_idx as u32 + 1,
                            crate::ide::offset_to_position(line, binding_offset).1,
                        ));
                    }
                    search_start = vbind_pos + 7 + close_paren + 1;
                } else {
                    break;
                }
            }
        }

        refs
    }

    /// Find a binding definition in script content.
    pub(super) fn find_binding_in_script(content: &str, name: &str) -> Option<usize> {
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
