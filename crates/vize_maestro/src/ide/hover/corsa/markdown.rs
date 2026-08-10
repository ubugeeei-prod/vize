//! Markdown shaping for hovers answered through the Corsa bridge.

use super::HoverService;

impl HoverService {
    /// Wrap TypeScript type information in a code block for proper markdown rendering.
    pub(in crate::ide::hover) fn wrap_type_info_in_codeblock(text: &str) -> String {
        let text = text.trim();
        // If already wrapped in code block, return as-is
        if text.starts_with("```") {
            return text.to_string();
        }
        // Check if this looks like TypeScript type info
        // Common patterns: (const), (let), (var), (function), (method), (property), type, interface, etc.
        let looks_like_type_info = text.starts_with('(')
            || text.starts_with("type ")
            || text.starts_with("interface ")
            || text.starts_with("class ")
            || text.starts_with("enum ")
            || text.starts_with("function ")
            || text.starts_with("const ")
            || text.starts_with("let ")
            || text.starts_with("var ")
            || text.starts_with("import ")
            || text.contains(": ")
            || text.contains("=>")
            || text.contains(" | ")
            || text.contains(" & ");

        if looks_like_type_info {
            #[allow(clippy::disallowed_macros)]
            {
                format!("```typescript\n{}\n```", text)
            }
        } else {
            text.to_string()
        }
    }

    /// The hover body is the signature itself. The former
    /// `**TypeScript quick info**` / `_Resolved through Vize virtual
    /// TypeScript_` preamble named an implementation detail no user acts on
    /// and pushed the signature below the fold in small popups (#3894);
    /// Volar and tsserver open with the code block.
    pub(super) fn decorate_corsa_hover_markdown(value: &str) -> String {
        value.trim().to_string()
    }
}
