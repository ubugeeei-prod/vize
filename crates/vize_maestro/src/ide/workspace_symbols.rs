//! Workspace symbols provider.
//!
//! Provides workspace-wide symbol search for:
//! - Vue components (from file names)
//! - Script bindings (functions, variables, classes)
//! - CSS classes and IDs
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use tower_lsp::lsp_types::{Location, Position, Range, SymbolInformation, SymbolKind, Url};

use crate::server::ServerState;
use vize_carton::cstr;

/// Workspace symbols service.
pub struct WorkspaceSymbolsService;

impl WorkspaceSymbolsService {
    /// Search for symbols matching a query.
    pub fn search(state: &ServerState, query: &str) -> Vec<SymbolInformation> {
        let mut symbols = Vec::new();
        let query_lower = query.to_lowercase();

        // Search in all open documents
        for entry in state.documents.iter() {
            let uri = entry.key();
            let doc = entry.value();
            let content = doc.text();

            // Only process .vue files
            if !uri.path().ends_with(".vue") {
                continue;
            }

            Self::collect_symbols_from_document(uri, &content, &query_lower, &mut symbols);
        }

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

        // Limit results
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
        if let Some(component_name) = Self::extract_component_name(uri) {
            if component_name.to_lowercase().contains(query) {
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
                if let Some((name, kind)) = Self::parse_declaration(rest) {
                    if name.to_lowercase().contains(query) {
                        symbols.push(Self::create_symbol(
                            name,
                            kind,
                            uri.clone(),
                            line_num - 1,
                            container,
                        ));
                    }
                }
            }
            // let name = ...
            else if let Some(rest) = trimmed.strip_prefix("let ") {
                if let Some((name, kind)) = Self::parse_declaration(rest) {
                    if name.to_lowercase().contains(query) {
                        symbols.push(Self::create_symbol(
                            name,
                            kind,
                            uri.clone(),
                            line_num - 1,
                            container,
                        ));
                    }
                }
            }
            // function name(...) { ... }
            else if let Some(rest) = trimmed.strip_prefix("function ") {
                if let Some(name) = Self::extract_identifier(rest) {
                    if name.to_lowercase().contains(query) {
                        symbols.push(Self::create_symbol(
                            name,
                            SymbolKind::FUNCTION,
                            uri.clone(),
                            line_num - 1,
                            container,
                        ));
                    }
                }
            }
            // async function name(...) { ... }
            else if let Some(rest) = trimmed.strip_prefix("async function ") {
                if let Some(name) = Self::extract_identifier(rest) {
                    if name.to_lowercase().contains(query) {
                        symbols.push(Self::create_symbol(
                            name,
                            SymbolKind::FUNCTION,
                            uri.clone(),
                            line_num - 1,
                            container,
                        ));
                    }
                }
            }
            // class Name { ... }
            else if let Some(rest) = trimmed.strip_prefix("class ") {
                if let Some(name) = Self::extract_identifier(rest) {
                    if name.to_lowercase().contains(query) {
                        symbols.push(Self::create_symbol(
                            name,
                            SymbolKind::CLASS,
                            uri.clone(),
                            line_num - 1,
                            container,
                        ));
                    }
                }
            }
            // interface Name { ... }
            else if let Some(rest) = trimmed.strip_prefix("interface ") {
                if let Some(name) = Self::extract_identifier(rest) {
                    if name.to_lowercase().contains(query) {
                        symbols.push(Self::create_symbol(
                            name,
                            SymbolKind::INTERFACE,
                            uri.clone(),
                            line_num - 1,
                            container,
                        ));
                    }
                }
            }
            // type Name = ...
            else if let Some(rest) = trimmed.strip_prefix("type ") {
                if let Some(name) = Self::extract_identifier(rest) {
                    if name.to_lowercase().contains(query) {
                        symbols.push(Self::create_symbol(
                            name,
                            SymbolKind::TYPE_PARAMETER,
                            uri.clone(),
                            line_num - 1,
                            container,
                        ));
                    }
                }
            }
            // enum Name { ... }
            else if let Some(rest) = trimmed.strip_prefix("enum ") {
                if let Some(name) = Self::extract_identifier(rest) {
                    if name.to_lowercase().contains(query) {
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

#[cfg(test)]
mod tests {
    use super::WorkspaceSymbolsService;
    use tower_lsp::lsp_types::SymbolKind;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(
            WorkspaceSymbolsService::to_pascal_case("hello-world"),
            "HelloWorld"
        );
        assert_eq!(
            WorkspaceSymbolsService::to_pascal_case("my_component"),
            "MyComponent"
        );
        assert_eq!(WorkspaceSymbolsService::to_pascal_case("Button"), "Button");
    }

    #[test]
    fn test_extract_identifier() {
        assert_eq!(
            WorkspaceSymbolsService::extract_identifier("count = 0"),
            Some("count".to_string())
        );
        assert_eq!(
            WorkspaceSymbolsService::extract_identifier("MyClass extends Base"),
            Some("MyClass".to_string())
        );
        assert_eq!(
            WorkspaceSymbolsService::extract_identifier("{ a, b } = obj"),
            None
        );
    }

    #[test]
    fn test_extract_css_classes() {
        let classes = WorkspaceSymbolsService::extract_css_classes(".container .item-active { }");
        assert_eq!(classes, vec!["container", "item-active"]);
    }

    #[test]
    fn test_extract_css_ids() {
        let ids = WorkspaceSymbolsService::extract_css_ids("#app #main-content { }");
        assert_eq!(ids, vec!["app", "main-content"]);
    }

    #[test]
    fn test_parse_declaration() {
        let (name, kind) = WorkspaceSymbolsService::parse_declaration("count = ref(0)").unwrap();
        assert_eq!(name, "count");
        assert_eq!(kind, SymbolKind::VARIABLE);

        let (name, kind) =
            WorkspaceSymbolsService::parse_declaration("handleClick = () => {}").unwrap();
        assert_eq!(name, "handleClick");
        assert_eq!(kind, SymbolKind::FUNCTION);
    }
}
