//! Workspace-wide symbol search for Vue components, script bindings, and styles.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

#[cfg(feature = "native")]
mod disk;
#[cfg(test)]
mod tests;

use tower_lsp::lsp_types::{Location, Position, Range, SymbolInformation, SymbolKind, Url};

use crate::server::ServerState;
use vize_s0::cstr;

/// Workspace symbols service.
pub struct WorkspaceSymbolsService;

impl WorkspaceSymbolsService {
    /// Search for symbols matching a query.
    pub fn search(state: &ServerState, query: &str) -> Vec<SymbolInformation> {
        let mut symbols = Vec::new();
        let query_lower = query.to_lowercase();

        // Search in all open documents. Snapshot the text before parsing so a
        // workspace-wide request never keeps a DashMap document guard alive
        // while doing SFC work (#3952).
        for (uri, content) in state.documents.vue_texts() {
            Self::collect_symbols_from_document(&uri, &content, &query_lower, &mut symbols);
        }

        #[cfg(feature = "native")]
        disk::collect(state, &query_lower, &mut symbols);

        // Sort by relevance (exact match first, then prefix match, then contains)
        symbols.sort_by(|a, b| {
            let a_name = a.name.to_lowercase();
            let b_name = b.name.to_lowercase();

            let a_exact = a_name == query_lower;
            let b_exact = b_name == query_lower;

            if a_exact != b_exact {
                return b_exact.cmp(&a_exact);
            }

            let a_prefix = a_name.starts_with(&query_lower);
            let b_prefix = b_name.starts_with(&query_lower);

            if a_prefix != b_prefix {
                return b_prefix.cmp(&a_prefix);
            }

            a_name.cmp(&b_name)
        });

        symbols.truncate(100);

        symbols
    }

    /// Collect symbols from a single document.
    #[allow(deprecated)] // SymbolInformation.deprecated is deprecated in favor of tags
    fn collect_symbols_from_document(
        uri: &Url,
        content: &str,
        query: &str,
        symbols: &mut Vec<SymbolInformation>,
    ) {
        let options = vize_atelier_sfc::SfcParseOptions {
            filename: uri.path().to_string().into(),
            ..Default::default()
        };

        let Ok(descriptor) = vize_atelier_sfc::parse_sfc(content, options) else {
            return;
        };

        // Extract component name from file path
        if let Some(component_name) = Self::extract_component_name(uri)
            && component_name.to_lowercase().contains(query)
        {
            symbols.push(SymbolInformation {
                name: component_name,
                kind: SymbolKind::CLASS,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: uri.clone(),
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: 0,
                            character: 0,
                        },
                    },
                },
                container_name: None,
            });
        }

        // Collect from script setup
        if let Some(ref script_setup) = descriptor.script_setup {
            Self::collect_script_symbols(
                uri,
                &script_setup.content,
                script_setup.loc.start_line as u32,
                query,
                Some("script setup"),
                symbols,
            );
        }

        // Collect from script
        if let Some(ref script) = descriptor.script {
            Self::collect_script_symbols(
                uri,
                &script.content,
                script.loc.start_line as u32,
                query,
                Some("script"),
                symbols,
            );
        }

        // Collect from styles
        for (idx, style) in descriptor.styles.iter().enumerate() {
            Self::collect_style_symbols(
                uri,
                &style.content,
                style.loc.start_line as u32,
                query,
                Some(&cstr!("style[{idx}]")),
                symbols,
            );
        }

        // Vue-specific symbols (emits, slots, provide/inject keys, …) so
        // workspace symbol search surfaces component contracts, not just
        // script bindings. Foundation for #697 — currently covers emits and
        // slot names; provide/inject and template refs follow the same shape.
        Self::collect_vue_specific_symbols(uri, &descriptor, query, symbols);
    }

    /// Surface Vue-specific entities (emits, slots) discovered by Croquis.
    #[allow(deprecated)] // SymbolInformation.deprecated is deprecated in favor of tags
    fn collect_vue_specific_symbols(
        uri: &Url,
        descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
        query: &str,
        symbols: &mut Vec<SymbolInformation>,
    ) {
        let Some(ref script_setup) = descriptor.script_setup else {
            return;
        };

        let mut analyzer = vize_croquis::Drawer::with_options(vize_croquis::DrawerOptions {
            analyze_script: true,
            ..Default::default()
        });
        analyzer.analyze_script_setup(&script_setup.content);
        let croquis = analyzer.finish();

        let container = Self::extract_component_name(uri);

        // Fallback for Croquis symbols that do not yet carry a declaration
        // span, on top of the line-0-character-0 placeholder from #715.
        let script_setup_position =
            Self::offset_to_position(descriptor.source.as_ref(), script_setup.loc.start);
        let placeholder_range = Range {
            start: script_setup_position,
            end: script_setup_position,
        };

        // Emits declared via defineEmits<{...}>(). The macro tracker keeps the
        // raw names; expose each as an EVENT symbol so `@symbol` searches
        // discover "update:modelValue", "submit", etc.
        for emit in croquis.macros.emits() {
            let name = emit.name.as_str();
            if !query.is_empty() && !name.to_lowercase().contains(query) {
                continue;
            }
            let range = Self::macro_declaration_range(
                descriptor.source.as_ref(),
                script_setup.content.as_ref(),
                script_setup.loc.start,
                croquis.macros.emit_declaration(name),
                placeholder_range,
            );
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: SymbolKind::EVENT,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: uri.clone(),
                    range,
                },
                container_name: container.clone(),
            });
        }

        // Slots declared via defineSlots<{...}>(). Expose each as an
        // INTERFACE symbol — slots define the parent-child contract, much
        // like a TypeScript interface.
        for slot in croquis.macros.slots() {
            let name = slot.name.as_str();
            if !query.is_empty() && !name.to_lowercase().contains(query) {
                continue;
            }
            symbols.push(SymbolInformation {
                name: name.to_string(),
                kind: SymbolKind::INTERFACE,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: uri.clone(),
                    range: placeholder_range,
                },
                container_name: container.clone(),
            });
        }
    }

    fn macro_declaration_range(
        source: &str,
        script: &str,
        script_start: usize,
        declaration: Option<(u32, u32)>,
        fallback: Range,
    ) -> Range {
        let Some((start, end)) = declaration else {
            return fallback;
        };
        let start = start as usize;
        let end = end as usize;
        if start >= end || script.get(start..end).is_none() {
            return fallback;
        }

        let (start, end) = Self::trim_paired_quotes(script, start, end);
        let (Some(source_start), Some(source_end)) = (
            script_start.checked_add(start),
            script_start.checked_add(end),
        ) else {
            return fallback;
        };
        if source_start > source_end || source.get(source_start..source_end).is_none() {
            return fallback;
        }

        Range {
            start: Self::offset_to_position(source, source_start),
            end: Self::offset_to_position(source, source_end),
        }
    }

    fn trim_paired_quotes(source: &str, start: usize, end: usize) -> (usize, usize) {
        let Some(raw) = source.get(start..end) else {
            return (start, end);
        };
        let bytes = raw.as_bytes();
        if bytes.len() < 2 {
            return (start, end);
        }
        let Some(last) = bytes.last() else {
            return (start, end);
        };
        if matches!(bytes[0], b'\'' | b'"') && *last == bytes[0] {
            (start + 1, end - 1)
        } else {
            (start, end)
        }
    }

    /// Convert a byte offset in `source` to an LSP `Position`. Simple line/
    /// column calculator — workspace symbols don't go through the heavier
    /// position mapping used by diagnostics.
    fn offset_to_position(source: &str, offset: usize) -> Position {
        let bounded = offset.min(source.len());
        let prefix = &source[..bounded];
        let line = prefix.matches('\n').count() as u32;
        let last_nl = prefix.rfind('\n').map(|p| p + 1).unwrap_or(0);
        let character = (bounded - last_nl) as u32;
        Position { line, character }
    }

    /// Extract component name from URI.
    fn extract_component_name(uri: &Url) -> Option<String> {
        let path = uri.path();
        let file_name = path.rsplit('/').next()?;

        // Remove .vue extension
        let name = file_name.strip_suffix(".vue")?;

        // Convert to PascalCase
        Some(Self::to_pascal_case(name))
    }

    /// Convert string to PascalCase.
    fn to_pascal_case(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let mut capitalize_next = true;

        for c in s.chars() {
            if c == '-' || c == '_' || c == '.' {
                capitalize_next = true;
            } else if capitalize_next {
                result.push(c.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                result.push(c);
            }
        }

        result
    }

    /// Collect symbols from script content.
    fn collect_script_symbols(
        uri: &Url,
        script: &str,
        base_line: u32,
        query: &str,
        container: Option<&str>,
        symbols: &mut Vec<SymbolInformation>,
    ) {
        let lines: Vec<&str> = script.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let line_num = base_line + line_idx as u32;
            let trimmed = line.trim_start();

            // const name = ...
            if let Some(rest) = trimmed.strip_prefix("const ") {
                if let Some((name, kind)) = Self::parse_declaration(rest)
                    && name.to_lowercase().contains(query)
                {
                    symbols.push(Self::create_symbol(
                        name,
                        kind,
                        uri.clone(),
                        line_num - 1,
                        container,
                    ));
                }
            }
            // let name = ...
            else if let Some(rest) = trimmed.strip_prefix("let ") {
                if let Some((name, kind)) = Self::parse_declaration(rest)
                    && name.to_lowercase().contains(query)
                {
                    symbols.push(Self::create_symbol(
                        name,
                        kind,
                        uri.clone(),
                        line_num - 1,
                        container,
                    ));
                }
            }
            // function name(...) { ... }
            else if let Some(rest) = trimmed.strip_prefix("function ") {
                if let Some(name) = Self::extract_identifier(rest)
                    && name.to_lowercase().contains(query)
                {
                    symbols.push(Self::create_symbol(
                        name,
                        SymbolKind::FUNCTION,
                        uri.clone(),
                        line_num - 1,
                        container,
                    ));
                }
            }
            // async function name(...) { ... }
            else if let Some(rest) = trimmed.strip_prefix("async function ") {
                if let Some(name) = Self::extract_identifier(rest)
                    && name.to_lowercase().contains(query)
                {
                    symbols.push(Self::create_symbol(
                        name,
                        SymbolKind::FUNCTION,
                        uri.clone(),
                        line_num - 1,
                        container,
                    ));
                }
            }
            // class Name { ... }
            else if let Some(rest) = trimmed.strip_prefix("class ") {
                if let Some(name) = Self::extract_identifier(rest)
                    && name.to_lowercase().contains(query)
                {
                    symbols.push(Self::create_symbol(
                        name,
                        SymbolKind::CLASS,
                        uri.clone(),
                        line_num - 1,
                        container,
                    ));
                }
            }
            // interface Name { ... }
            else if let Some(rest) = trimmed.strip_prefix("interface ") {
                if let Some(name) = Self::extract_identifier(rest)
                    && name.to_lowercase().contains(query)
                {
                    symbols.push(Self::create_symbol(
                        name,
                        SymbolKind::INTERFACE,
                        uri.clone(),
                        line_num - 1,
                        container,
                    ));
                }
            }
            // type Name = ...
            else if let Some(rest) = trimmed.strip_prefix("type ") {
                if let Some(name) = Self::extract_identifier(rest)
                    && name.to_lowercase().contains(query)
                {
                    symbols.push(Self::create_symbol(
                        name,
                        SymbolKind::TYPE_PARAMETER,
                        uri.clone(),
                        line_num - 1,
                        container,
                    ));
                }
            }
            // enum Name { ... }
            else if let Some(rest) = trimmed.strip_prefix("enum ")
                && let Some(name) = Self::extract_identifier(rest)
                && name.to_lowercase().contains(query)
            {
                symbols.push(Self::create_symbol(
                    name,
                    SymbolKind::ENUM,
                    uri.clone(),
                    line_num - 1,
                    container,
                ));
            }
        }
    }

    /// Collect symbols from style content.
    fn collect_style_symbols(
        uri: &Url,
        style: &str,
        base_line: u32,
        query: &str,
        container: Option<&str>,
        symbols: &mut Vec<SymbolInformation>,
    ) {
        let lines: Vec<&str> = style.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let line_num = base_line + line_idx as u32;
            let trimmed = line.trim();

            // CSS class selectors
            for class in Self::extract_css_classes(trimmed) {
                if class.to_lowercase().contains(query) {
                    #[allow(clippy::disallowed_macros)]
                    symbols.push(Self::create_symbol(
                        format!(".{}", class),
                        SymbolKind::STRING,
                        uri.clone(),
                        line_num - 1,
                        container,
                    ));
                }
            }

            // CSS ID selectors
            for id in Self::extract_css_ids(trimmed) {
                if id.to_lowercase().contains(query) {
                    #[allow(clippy::disallowed_macros)]
                    symbols.push(Self::create_symbol(
                        format!("#{}", id),
                        SymbolKind::STRING,
                        uri.clone(),
                        line_num - 1,
                        container,
                    ));
                }
            }
        }
    }

    /// Parse a declaration and return name and kind.
    fn parse_declaration(s: &str) -> Option<(String, SymbolKind)> {
        let name = Self::extract_identifier(s)?;

        // Determine kind based on initialization
        let kind = if s.contains("ref(") || s.contains("computed(") || s.contains("reactive(") {
            SymbolKind::VARIABLE
        } else if s.contains("=>") || s.contains("function") {
            SymbolKind::FUNCTION
        } else {
            SymbolKind::CONSTANT
        };

        Some((name, kind))
    }

    /// Extract identifier from string.
    fn extract_identifier(s: &str) -> Option<String> {
        let s = s.trim_start();
        if s.is_empty() {
            return None;
        }

        let bytes = s.as_bytes();
        let first = bytes[0] as char;

        // Skip destructuring
        if first == '{' || first == '[' {
            return None;
        }

        if !Self::is_ident_start(first) {
            return None;
        }

        let mut end = 1;
        while end < bytes.len() && Self::is_ident_char(bytes[end] as char) {
            end += 1;
        }

        Some(s[..end].to_string())
    }

    /// Extract CSS class names from a selector line.
    fn extract_css_classes(line: &str) -> Vec<String> {
        let mut classes = Vec::new();
        let mut pos = 0;

        while let Some(dot_pos) = line[pos..].find('.') {
            let abs_pos = pos + dot_pos + 1;
            if abs_pos < line.len() {
                let rest = &line[abs_pos..];
                let end = rest
                    .find(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
                    .unwrap_or(rest.len());

                if end > 0 {
                    classes.push(rest[..end].to_string());
                }

                pos = abs_pos + end;
            } else {
                break;
            }
        }

        classes
    }

    /// Extract CSS ID names from a selector line.
    fn extract_css_ids(line: &str) -> Vec<String> {
        let mut ids = Vec::new();
        let mut pos = 0;

        while let Some(hash_pos) = line[pos..].find('#') {
            let abs_pos = pos + hash_pos + 1;
            if abs_pos < line.len() {
                let rest = &line[abs_pos..];
                let end = rest
                    .find(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
                    .unwrap_or(rest.len());

                if end > 0 {
                    ids.push(rest[..end].to_string());
                }

                pos = abs_pos + end;
            } else {
                break;
            }
        }

        ids
    }

    /// Create a symbol information entry.
    #[allow(deprecated)]
    fn create_symbol(
        name: String,
        kind: SymbolKind,
        uri: Url,
        line: u32,
        container: Option<&str>,
    ) -> SymbolInformation {
        SymbolInformation {
            name,
            kind,
            tags: None,
            deprecated: None,
            location: Location {
                uri,
                range: Range {
                    start: Position { line, character: 0 },
                    end: Position { line, character: 0 },
                },
            },
            container_name: container.map(|s| s.to_string()),
        }
    }

    fn is_ident_start(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_' || c == '$'
    }

    fn is_ident_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || c == '$'
    }
}
