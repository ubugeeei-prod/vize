//! Hover information provider.
//!
//! Provides contextual hover information for:
//! - Template expressions and bindings
//! - Vue directives
//! - Script bindings and imports
//! - CSS properties and Vue-specific selectors
//! - TypeScript type information from croquis analysis
//! - Real type information from tsgo (when available)
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

mod script;
mod template;
#[cfg(feature = "native")]
mod tsgo;

#[cfg(feature = "native")]
use std::sync::Arc;

use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Range};
use vize_relief::BindingType;

#[cfg(feature = "native")]
use vize_canon::TsgoBridge;

use super::IdeContext;
use crate::virtual_code::{ArtCursorPosition, BlockType};

/// Hover service for providing contextual information.
pub struct HoverService;

impl HoverService {
    /// Get hover information for the given context.
    pub fn hover(ctx: &IdeContext) -> Option<Hover> {
        match ctx.block_type? {
            BlockType::Template => Self::hover_template(ctx),
            BlockType::Script => Self::hover_script(ctx, false),
            BlockType::ScriptSetup => Self::hover_script(ctx, true),
            BlockType::Style(index) => Self::hover_style(ctx, index),
            BlockType::Art(ArtCursorPosition::VariantTemplate(_)) => Self::hover_template(ctx),
            BlockType::Art(_) => None,
        }
    }

    /// Get hover information with tsgo support (async version).
    ///
    /// This method first tries to get type information from tsgo,
    /// then falls back to the synchronous analysis.
    #[cfg(feature = "native")]
    pub async fn hover_with_tsgo(
        ctx: &IdeContext<'_>,
        tsgo_bridge: Option<Arc<TsgoBridge>>,
    ) -> Option<Hover> {
        match ctx.block_type? {
            BlockType::Template => Self::hover_template_with_tsgo(ctx, tsgo_bridge).await,
            BlockType::Script => Self::hover_script_with_tsgo(ctx, false, tsgo_bridge).await,
            BlockType::ScriptSetup => Self::hover_script_with_tsgo(ctx, true, tsgo_bridge).await,
            BlockType::Style(index) => Self::hover_style(ctx, index),
            BlockType::Art(ArtCursorPosition::VariantTemplate(ref info)) => {
                Self::hover_art_variant_with_tsgo(ctx, info, tsgo_bridge).await
            }
            BlockType::Art(_) => None,
        }
    }

    // =========================================================================
    // Style hover
    // =========================================================================

    /// Get hover for style context.
    fn hover_style(ctx: &IdeContext, _index: usize) -> Option<Hover> {
        let word = Self::get_word_at_offset(&ctx.content, ctx.offset);

        if word.is_empty() {
            return None;
        }

        // Check for Vue-specific CSS features
        if let Some(hover) = Self::hover_vue_css(&word) {
            return Some(hover);
        }

        None
    }

    /// Get hover for Vue CSS features.
    fn hover_vue_css(word: &str) -> Option<Hover> {
        let (title, description) = match word {
            "v-bind" => (
                "v-bind() in CSS",
                "Link CSS values to dynamic component state. The value will be compiled into a hashed CSS custom property.",
            ),
            ":deep" => (
                ":deep()",
                "Affects child component styles in scoped CSS. The selector inside `:deep()` will be compiled with the scoped attribute.",
            ),
            ":slotted" => (
                ":slotted()",
                "Target content passed via slots in scoped CSS. Only works inside scoped `<style>` blocks.",
            ),
            ":global" => (
                ":global()",
                "Apply styles globally, escaping the scoped CSS encapsulation.",
            ),
            _ => return None,
        };

        #[allow(clippy::disallowed_macros)]
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!(
                    "**{}**\n\n{}\n\n[Vue SFC CSS Features](https://vuejs.org/api/sfc-css-features.html)",
                    title, description
                ),
            }),
            range: None,
        })
    }

    // =========================================================================
    // Shared utilities
    // =========================================================================

    /// Get the word at a given offset.
    pub(super) fn get_word_at_offset(content: &str, offset: usize) -> String {
        if offset >= content.len() {
            return String::new();
        }

        let bytes = content.as_bytes();

        // If the character at offset is not a word character, return empty
        if !Self::is_word_char(bytes[offset]) {
            return String::new();
        }

        // Find word start
        let mut start = offset;
        while start > 0 {
            let c = bytes[start - 1];
            if !Self::is_word_char(c) {
                break;
            }
            start -= 1;
        }

        // Find word end
        let mut end = offset;
        while end < bytes.len() {
            let c = bytes[end];
            if !Self::is_word_char(c) {
                break;
            }
            end += 1;
        }

        if start == end {
            return String::new();
        }

        String::from_utf8_lossy(&bytes[start..end]).to_string()
    }

    /// Check if a byte is a valid word character.
    #[inline]
    fn is_word_char(c: u8) -> bool {
        c.is_ascii_alphanumeric() || c == b'_' || c == b'-' || c == b'$' || c == b':'
    }

    // =========================================================================
    // Type inference utilities
    // =========================================================================

    /// Infer a more specific type from the script content.
    pub(super) fn infer_type_from_script(
        content: &str,
        name: &str,
        binding_type: BindingType,
    ) -> Option<String> {
        // Patterns to look for initialization
        #[allow(clippy::disallowed_macros)]
        let patterns = [
            format!("const {} = ref(", name),
            format!("const {} = ref<", name),
            format!("let {} = ref(", name),
            format!("const {} = shallowRef(", name),
            format!("const {} = reactive(", name),
            format!("const {} = reactive<", name),
            format!("const {} = computed(", name),
            format!("const {} = computed<", name),
        ];

        for pattern in &patterns {
            if let Some(pos) = content.find(pattern.as_str()) {
                let after_pattern = &content[pos + pattern.len()..];

                // Check if it's a generic type annotation: ref<Type>
                if pattern.ends_with('<') {
                    // Find the closing >
                    if let Some(end) = Self::find_matching_bracket(after_pattern, '<', '>') {
                        let type_arg = &after_pattern[..end];
                        return Some(Self::format_wrapper_type(pattern, type_arg));
                    }
                }

                // Try to infer from the argument
                if let Some(arg_type) = Self::infer_type_from_arg(after_pattern) {
                    return Some(Self::format_wrapper_type(pattern, &arg_type));
                }
            }
        }

        // Check for explicit type annotation: const name: Type = ...
        #[allow(clippy::disallowed_macros)]
        let type_annotation_patterns = [format!("const {}: ", name), format!("let {}: ", name)];

        for pattern in &type_annotation_patterns {
            if let Some(pos) = content.find(pattern.as_str()) {
                let after_pattern = &content[pos + pattern.len()..];
                // Find = or end of type
                if let Some(type_str) = Self::extract_type_annotation(after_pattern) {
                    return Some(type_str);
                }
            }
        }

        // For Props, try to get the actual prop type
        if binding_type == BindingType::Props {
            return Self::infer_prop_type(content, name);
        }

        None
    }

    /// Format the wrapper type (Ref, Reactive, etc.) with the inner type.
    #[allow(clippy::disallowed_macros)]
    fn format_wrapper_type(pattern: &str, inner_type: &str) -> String {
        if pattern.contains("ref(") || pattern.contains("ref<") || pattern.contains("shallowRef(") {
            format!("Ref<{}>", inner_type)
        } else if pattern.contains("reactive(") || pattern.contains("reactive<") {
            format!("Reactive<{}>", inner_type)
        } else if pattern.contains("computed(") || pattern.contains("computed<") {
            format!("ComputedRef<{}>", inner_type)
        } else {
            inner_type.to_string()
        }
    }

    /// Infer type from an argument value (literal or expression).
    fn infer_type_from_arg(arg_str: &str) -> Option<String> {
        let arg_str = arg_str.trim();

        // Number literal
        if arg_str.starts_with(|c: char| c.is_ascii_digit() || c == '-') {
            let num_end = arg_str
                .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != 'e' && c != 'E')
                .unwrap_or(arg_str.len());
            let num_str = &arg_str[..num_end];
            if num_str.contains('.') || num_str.contains('e') || num_str.contains('E') {
                return Some("number".to_string());
            }
            return Some("number".to_string());
        }

        // String literal
        if arg_str.starts_with('"') || arg_str.starts_with('\'') || arg_str.starts_with('`') {
            return Some("string".to_string());
        }

        // Boolean literal
        if arg_str.starts_with("true") || arg_str.starts_with("false") {
            return Some("boolean".to_string());
        }

        // Array literal
        if arg_str.starts_with('[') {
            // Try to infer array element type
            if arg_str.starts_with("[]") {
                return Some("unknown[]".to_string());
            }
            return Some("unknown[]".to_string());
        }

        // Object literal
        if arg_str.starts_with('{') {
            // Could try to infer object structure, but keep it simple for now
            return Some("object".to_string());
        }

        // null/undefined
        if arg_str.starts_with("null") {
            return Some("null".to_string());
        }
        if arg_str.starts_with("undefined") {
            return Some("undefined".to_string());
        }

        None
    }

    /// Extract type annotation from a string like "Type = ..."
    fn extract_type_annotation(s: &str) -> Option<String> {
        let s = s.trim();
        let mut depth = 0;
        let mut end = 0;

        for (i, c) in s.chars().enumerate() {
            match c {
                '<' | '(' | '[' | '{' => depth += 1,
                '>' | ')' | ']' | '}' => depth -= 1,
                '=' if depth == 0 => {
                    end = i;
                    break;
                }
                ';' | '\n' if depth == 0 => {
                    end = i;
                    break;
                }
                _ => {}
            }
            end = i + 1;
        }

        if end > 0 {
            let type_str = s[..end].trim();
            if !type_str.is_empty() {
                return Some(type_str.to_string());
            }
        }

        None
    }

    /// Find matching bracket position.
    fn find_matching_bracket(s: &str, open: char, close: char) -> Option<usize> {
        let mut depth = 1;
        for (i, c) in s.chars().enumerate() {
            if c == open {
                depth += 1;
            } else if c == close {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Infer prop type from defineProps.
    #[allow(clippy::disallowed_macros)]
    fn infer_prop_type(content: &str, prop_name: &str) -> Option<String> {
        // Look for defineProps<{ propName: Type }>
        if let Some(props_start) = content.find("defineProps<") {
            let after = &content[props_start + "defineProps<".len()..];
            if let Some(end) = Self::find_matching_bracket(after, '<', '>') {
                let props_type = &after[..end];
                // Look for the property
                let prop_pattern = format!("{}: ", prop_name);
                if let Some(prop_pos) = props_type.find(prop_pattern.as_str()) {
                    let after_prop = &props_type[prop_pos + prop_pattern.len()..];
                    if let Some(type_str) = Self::extract_prop_type(after_prop) {
                        return Some(type_str);
                    }
                }
                // Also check for optional: propName?: Type
                let opt_pattern = format!("{}?: ", prop_name);
                if let Some(prop_pos) = props_type.find(opt_pattern.as_str()) {
                    let after_prop = &props_type[prop_pos + opt_pattern.len()..];
                    if let Some(type_str) = Self::extract_prop_type(after_prop) {
                        return Some(format!("{} | undefined", type_str));
                    }
                }
            }
        }
        None
    }

    /// Extract a prop type from the remaining string.
    fn extract_prop_type(s: &str) -> Option<String> {
        let s = s.trim();
        let mut depth = 0;
        let mut end = 0;

        for (i, c) in s.chars().enumerate() {
            match c {
                '<' | '(' | '[' | '{' => depth += 1,
                '>' | ')' | ']' | '}' => {
                    if depth == 0 {
                        end = i;
                        break;
                    }
                    depth -= 1;
                }
                ',' | ';' | '\n' if depth == 0 => {
                    end = i;
                    break;
                }
                _ => {}
            }
            end = i + 1;
        }

        if end > 0 {
            let type_str = s[..end].trim();
            if !type_str.is_empty() {
                return Some(type_str.to_string());
            }
        }

        None
    }
}

/// Hover content builder for creating rich hover information.
pub struct HoverBuilder {
    sections: Vec<String>,
}

impl HoverBuilder {
    /// Create a new hover builder.
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
        }
    }

    /// Add a title.
    #[allow(clippy::disallowed_macros)]
    pub fn title(mut self, title: &str) -> Self {
        self.sections.push(format!("**{}**", title));
        self
    }

    /// Add a code block.
    #[allow(clippy::disallowed_macros)]
    pub fn code(mut self, language: &str, code: &str) -> Self {
        self.sections
            .push(format!("```{}\n{}\n```", language, code));
        self
    }

    /// Add a description.
    pub fn description(mut self, text: &str) -> Self {
        self.sections.push(text.to_string());
        self
    }

    /// Add a documentation link.
    #[allow(clippy::disallowed_macros)]
    pub fn link(mut self, text: &str, url: &str) -> Self {
        self.sections.push(format!("[{}]({})", text, url));
        self
    }

    /// Build the hover.
    pub fn build(self) -> Hover {
        Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: self.sections.join("\n\n"),
            }),
            range: None,
        }
    }

    /// Build the hover with a range.
    pub fn build_with_range(self, range: Range) -> Hover {
        Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: self.sections.join("\n\n"),
            }),
            range: Some(range),
        }
    }
}

impl Default for HoverBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{HoverBuilder, HoverContents, HoverService};
    use vize_relief::BindingType;

    #[test]
    fn test_get_word_at_offset() {
        let content = "const message = 'hello'";

        assert_eq!(HoverService::get_word_at_offset(content, 0), "const");
        assert_eq!(HoverService::get_word_at_offset(content, 6), "message");
        assert_eq!(HoverService::get_word_at_offset(content, 5), "");
    }

    #[test]
    fn test_hover_directive() {
        let hover = HoverService::hover_directive("v-if");
        assert!(hover.is_some());

        let hover = HoverService::hover_directive("unknown");
        assert!(hover.is_none());
    }

    #[test]
    fn test_hover_vue_api() {
        let hover = HoverService::hover_vue_api("ref");
        assert!(hover.is_some());

        let hover = HoverService::hover_vue_api("unknown");
        assert!(hover.is_none());
    }

    #[test]
    fn test_hover_builder() {
        let hover = HoverBuilder::new()
            .title("ref")
            .code("typescript", "function ref<T>(value: T): Ref<T>")
            .description("Creates a reactive reference.")
            .link("Documentation", "https://vuejs.org")
            .build();

        if let HoverContents::Markup(content) = hover.contents {
            assert!(content.value.contains("**ref**"));
            assert!(content.value.contains("```typescript"));
        } else {
            panic!("Expected Markup content");
        }
    }

    #[test]
    fn test_binding_type_to_ts_display() {
        assert_eq!(
            HoverService::binding_type_to_ts_display(BindingType::SetupRef),
            "Ref<unknown>"
        );
        assert_eq!(
            HoverService::binding_type_to_ts_display(BindingType::SetupReactiveConst),
            "Reactive<unknown>"
        );
        assert_eq!(
            HoverService::binding_type_to_ts_display(BindingType::Props),
            "Props"
        );
        assert_eq!(
            HoverService::binding_type_to_ts_display(BindingType::SetupConst),
            "const"
        );
    }

    #[test]
    fn test_binding_type_to_description() {
        let desc = HoverService::binding_type_to_description(BindingType::SetupRef);
        assert!(desc.contains("ref()"));
        assert!(desc.contains(".value"));

        let desc = HoverService::binding_type_to_description(BindingType::Props);
        assert!(desc.contains("prop"));
        assert!(desc.contains("Read-only"));
    }
}
