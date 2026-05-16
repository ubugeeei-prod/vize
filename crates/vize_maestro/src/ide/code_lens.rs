//! Code lens provider.
//!
//! Provides code lenses for:
//! - Script setup bindings (usage count)
//! - Component references
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]
//! - Event handler references

use tower_lsp::lsp_types::{CodeLens, Command, Position, Range, Url};

/// Code lens service.
pub struct CodeLensService;

impl CodeLensService {
    /// Get code lenses for a document.
    pub fn get_lenses(content: &str, uri: &Url) -> Vec<CodeLens> {
        let mut lenses = Vec::new();

        let options = vize_atelier_sfc::SfcParseOptions {
            filename: uri.path().to_string().into(),
            ..Default::default()
        };

        let Ok(descriptor) = vize_atelier_sfc::parse_sfc(content, options) else {
            return lenses;
        };

        // Add lenses for script setup bindings
        if let Some(ref script_setup) = descriptor.script_setup {
            Self::collect_binding_lenses(
                &script_setup.content,
                script_setup.loc.start_line as u32,
                &descriptor,
                &mut lenses,
            );
        }

        // Add lenses for regular script
        if let Some(ref script) = descriptor.script {
            Self::collect_binding_lenses(
                &script.content,
                script.loc.start_line as u32,
                &descriptor,
                &mut lenses,
            );
        }

        lenses
    }

    /// Collect code lenses for bindings.
    fn collect_binding_lenses(
        script: &str,
        base_line: u32,
        descriptor: &vize_atelier_sfc::SfcDescriptor,
        lenses: &mut Vec<CodeLens>,
    ) {
        // Find const/let/function declarations
        let declarations = Self::find_declarations(script);

        for (name, line, _col) in declarations {
            // Count references in template
            let template_refs = descriptor
                .template
                .as_ref()
                .map(|t| Self::count_identifier_occurrences(&t.content, &name))
                .unwrap_or(0);

            // Count references in styles (v-bind)
            let style_refs: usize = descriptor
                .styles
                .iter()
                .map(|s| Self::count_vbind_occurrences(&s.content, &name))
                .sum();

            let total_refs = template_refs + style_refs;

            if total_refs > 0 {
                lenses.push(CodeLens {
                    range: Range {
                        start: Position {
                            line: base_line + line - 1,
                            character: 0,
                        },
                        end: Position {
                            line: base_line + line - 1,
                            character: 0,
                        },
                    },
                    #[allow(clippy::disallowed_macros)]
                    command: Some(Command {
                        title: format!(
                            "{} reference{}",
                            total_refs,
                            if total_refs == 1 { "" } else { "s" }
                        ),
                        command: "vize.findReferences".to_string(),
                        arguments: None,
                    }),
                    data: None,
                });
            }
        }
    }

    /// Find declarations in script.
    fn find_declarations(script: &str) -> Vec<(String, u32, u32)> {
        let mut declarations = Vec::new();

        let lines: Vec<&str> = script.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let line_num = (line_idx + 1) as u32;
            let trimmed = line.trim_start();

            // const name = ...
            if let Some(rest) = trimmed.strip_prefix("const ") {
                if let Some(name) = Self::extract_first_identifier(rest) {
                    let col = line.find(&name).unwrap_or(0) as u32;
                    declarations.push((name, line_num, col));
                }
            }
            // let name = ...
            else if let Some(rest) = trimmed.strip_prefix("let ") {
                if let Some(name) = Self::extract_first_identifier(rest) {
                    let col = line.find(&name).unwrap_or(0) as u32;
                    declarations.push((name, line_num, col));
                }
            }
            // function name(...) { ... }
            else if let Some(rest) = trimmed.strip_prefix("function ") {
                if let Some(name) = Self::extract_first_identifier(rest) {
                    let col = line.find(&name).unwrap_or(0) as u32;
                    declarations.push((name, line_num, col));
                }
            }
            // async function name(...) { ... }
            else if let Some(rest) = trimmed.strip_prefix("async function ") {
                if let Some(name) = Self::extract_first_identifier(rest) {
                    let col = line.find(&name).unwrap_or(0) as u32;
                    declarations.push((name, line_num, col));
                }
            }
        }

        declarations
    }

    /// Extract first identifier from a string.
    fn extract_first_identifier(s: &str) -> Option<String> {
        let s = s.trim_start();
        if s.is_empty() {
            return None;
        }

        let bytes = s.as_bytes();
        let first = bytes[0] as char;

        // Handle destructuring: const { a, b } = ...
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

    /// Count occurrences of an identifier in text.
    fn count_identifier_occurrences(text: &str, word: &str) -> usize {
        let bytes = text.as_bytes();
        let word_len = word.len();
        let mut count = 0;
        let mut pos = 0;

        while let Some(found) = text[pos..].find(word) {
            let abs_pos = pos + found;

            // Check word boundaries
            let before_ok = abs_pos == 0 || !Self::is_ident_char(bytes[abs_pos - 1] as char);
            let after_ok = abs_pos + word_len >= bytes.len()
                || !Self::is_ident_char(bytes[abs_pos + word_len] as char);

            if before_ok && after_ok {
                count += 1;
            }

            pos = abs_pos + 1;
        }

        count
    }

    /// Count v-bind() occurrences in CSS.
    fn count_vbind_occurrences(css: &str, word: &str) -> usize {
        let pattern = "v-bind(";
        let mut count = 0;
        let mut pos = 0;

        while let Some(start) = css[pos..].find(pattern) {
            let abs_start = pos + start + pattern.len();

            if let Some(end) = css[abs_start..].find(')') {
                let content = css[abs_start..abs_start + end].trim();
                let var_name = content.trim_matches(|c| c == '"' || c == '\'');

                if var_name == word {
                    count += 1;
                }

                pos = abs_start + end + 1;
            } else {
                break;
            }
        }

        count
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
    use super::CodeLensService;

    #[test]
    fn test_find_declarations() {
        let script = r#"
const count = ref(0)
let message = "hello"
function handleClick() {}
async function fetchData() {}
"#;
        let decls = CodeLensService::find_declarations(script);
        assert_eq!(decls.len(), 4);
        assert_eq!(decls[0].0, "count");
        assert_eq!(decls[1].0, "message");
        assert_eq!(decls[2].0, "handleClick");
        assert_eq!(decls[3].0, "fetchData");
    }

    #[test]
    fn test_count_identifier_occurrences() {
        let text = "count + count * 2 + countUp()";
        assert_eq!(
            CodeLensService::count_identifier_occurrences(text, "count"),
            2
        );
    }

    #[test]
    fn test_count_vbind_occurrences() {
        let css = ".container { color: v-bind(textColor); width: v-bind(textColor); }";
        assert_eq!(
            CodeLensService::count_vbind_occurrences(css, "textColor"),
            2
        );
    }

    #[test]
    fn test_extract_first_identifier() {
        assert_eq!(
            CodeLensService::extract_first_identifier("count = 0"),
            Some("count".to_string())
        );
        assert_eq!(
            CodeLensService::extract_first_identifier("{ a, b } = obj"),
            None
        );
        assert_eq!(
            CodeLensService::extract_first_identifier("[a, b] = arr"),
            None
        );
    }
}
