//! SSR code generation.
//!
//! SSR code generation produces JavaScript that uses template literals and `_push()` calls
//! to build HTML strings on the server side.

mod element;
pub(crate) mod helpers;

use crate::options::SsrCompilerOptions;
use vize_atelier_core::ast::{RootNode, RuntimeHelper, TemplateChildNode};
use vize_carton::{camelize, capitalize, cstr, Bump, FxHashSet, String, ToCompactString};

/// SSR codegen result
#[derive(Debug, Default)]
pub struct SsrCodegenResult {
    /// Generated render function code
    pub code: String,
    /// Import preamble
    pub preamble: String,
}

/// A part of a template literal
#[derive(Debug)]
pub(crate) enum TemplatePart {
    /// Static string content
    Static(String),
    /// Dynamic expression
    Dynamic(String),
}

/// SSR codegen context
pub struct SsrCodegenContext<'a> {
    #[allow(dead_code)]
    pub(crate) allocator: &'a Bump,
    pub(crate) options: &'a SsrCompilerOptions,
    /// Output buffer
    pub(crate) code: Vec<u8>,
    /// Indent level
    pub(crate) indent_level: u32,
    /// Used SSR helpers
    pub(crate) ssr_helpers: FxHashSet<RuntimeHelper>,
    /// Used core helpers (from vue)
    pub(crate) core_helpers: FxHashSet<RuntimeHelper>,
    /// Current template literal parts being accumulated
    pub(crate) current_template_parts: Vec<TemplatePart>,
    /// Whether we have an open _push call
    #[allow(dead_code)]
    pub(crate) has_open_push: bool,
    /// Whether currently within a slot scope
    #[allow(dead_code)]
    pub(crate) with_slot_scope_id: bool,
    /// Template-local identifiers from v-for and scoped slots.
    pub(crate) scoped_params: std::vec::Vec<FxHashSet<String>>,
}

impl<'a> SsrCodegenContext<'a> {
    pub fn new(allocator: &'a Bump, options: &'a SsrCompilerOptions) -> Self {
        Self {
            allocator,
            options,
            code: Vec::with_capacity(1024),
            indent_level: 0,
            ssr_helpers: FxHashSet::default(),
            core_helpers: FxHashSet::default(),
            current_template_parts: Vec::new(),
            has_open_push: false,
            with_slot_scope_id: false,
            scoped_params: std::vec::Vec::new(),
        }
    }

    /// Generate SSR code from the AST
    pub fn generate(mut self, root: &RootNode<'a>) -> SsrCodegenResult {
        // Transforms can rewrite setup bindings to `_unref(...)` in SSR
        // expressions. SSR codegen does not walk helper symbols the same way as
        // DOM codegen, so carry the root helper through explicitly.
        if root.helpers.contains(&RuntimeHelper::Unref) {
            self.use_core_helper(RuntimeHelper::Unref);
        }

        // Check if this is a fragment (multiple non-text children)
        let is_fragment = root.children.len() > 1
            && root
                .children
                .iter()
                .any(|c| !matches!(c, TemplateChildNode::Text(_)));

        // Generate function signature
        self.push("function ssrRender(_ctx, _push, _parent, _attrs");
        if self.options.binding_metadata.is_some() {
            self.push(", $props, $setup, $data, $options");
        }
        if self.options.scope_id.is_some() {
            self.push(", _scopeId");
        }
        self.push(") {\n");
        self.indent_level += 1;

        // Inject CSS vars if present
        if let Some(css_vars) = &self.options.ssr_css_vars {
            self.push_indent();
            self.push("const _cssVars = { style: ");
            self.push(css_vars);
            self.push(" }\n");
        }

        // Process children
        self.process_root_children(&root.children, is_fragment, false, false);

        // Flush any remaining template literal
        self.flush_push();

        self.indent_level -= 1;
        self.push("}\n");

        // Build preamble with imports
        let preamble = self.build_preamble();

        SsrCodegenResult {
            // SAFETY: We only push valid UTF-8 strings
            code: unsafe { String::from_utf8_unchecked(self.code) },
            preamble,
        }
    }

    /// Push static string content to the current template literal
    pub(crate) fn push_string_part_static(&mut self, s: &str) {
        if let Some(TemplatePart::Static(last)) = self.current_template_parts.last_mut() {
            last.push_str(s);
        } else {
            self.current_template_parts
                .push(TemplatePart::Static(s.to_compact_string()));
        }
    }

    /// Push dynamic expression to the current template literal
    pub(crate) fn push_string_part_dynamic(&mut self, expr: &str) {
        self.current_template_parts
            .push(TemplatePart::Dynamic(expr.to_compact_string()));
    }

    /// Flush the current template literal as a _push() call
    pub(crate) fn flush_push(&mut self) {
        if self.current_template_parts.is_empty() {
            return;
        }

        // Take ownership of parts to avoid borrow issues
        let parts = std::mem::take(&mut self.current_template_parts);

        self.push_indent();
        self.push("_push(`");

        for part in &parts {
            match part {
                TemplatePart::Static(s) => {
                    // Escape backticks and ${
                    let escaped = s.replace('`', "\\`").replace("${", "\\${");
                    self.push(&escaped);
                }
                TemplatePart::Dynamic(expr) => {
                    self.push("${");
                    self.push(expr);
                    self.push("}");
                }
            }
        }

        self.push("`)\n");
    }

    /// Use an SSR helper
    pub(crate) fn use_ssr_helper(&mut self, helper: RuntimeHelper) {
        self.ssr_helpers.insert(helper);
    }

    /// Use a core helper (from vue)
    pub(crate) fn use_core_helper(&mut self, helper: RuntimeHelper) {
        self.core_helpers.insert(helper);
    }

    pub(crate) fn resolve_component_binding_name(&self, component: &str) -> Option<String> {
        let metadata = self.options.binding_metadata.as_ref()?;

        let resolve_base = |name: &str| {
            if metadata.bindings.contains_key(name) {
                return Some(name.to_compact_string());
            }

            let camel = camelize(name);
            if metadata.bindings.contains_key(camel.as_str()) {
                return Some(camel);
            }

            let pascal = capitalize(camel.as_str());
            if metadata.bindings.contains_key(pascal.as_str()) {
                return Some(pascal);
            }

            None
        };

        if let Some((base, suffix)) = component.split_once('.') {
            let resolved_base = resolve_base(base)?;
            let mut resolved = String::with_capacity(resolved_base.len() + suffix.len() + 1);
            resolved.push_str(resolved_base.as_str());
            resolved.push('.');
            resolved.push_str(suffix);
            return Some(resolved);
        }

        resolve_base(component)
    }

    pub(crate) fn push_scoped_params(&mut self, params: FxHashSet<String>) {
        self.scoped_params.push(params);
    }

    pub(crate) fn pop_scoped_params(&mut self) {
        self.scoped_params.pop();
    }

    pub(crate) fn is_scoped_param(&self, name: &str) -> bool {
        self.scoped_params
            .iter()
            .rev()
            .any(|params| params.contains(name))
    }

    pub(crate) fn strip_ctx_for_scoped_params(&self, content: &str) -> String {
        if self.scoped_params.is_empty() || !content.contains("_ctx.") {
            return content.to_compact_string();
        }

        let mut result = String::with_capacity(content.len());
        let bytes = content.as_bytes();
        let prefix = b"_ctx.";
        let mut index = 0;

        while index < bytes.len() {
            if index + prefix.len() <= bytes.len() && &bytes[index..index + prefix.len()] == prefix
            {
                let start = index + prefix.len();
                let mut end = start;
                while end < bytes.len()
                    && (bytes[end].is_ascii_alphanumeric()
                        || bytes[end] == b'_'
                        || bytes[end] == b'$')
                {
                    end += 1;
                }

                let ident = &content[start..end];
                if !ident.is_empty() && self.is_scoped_param(ident) {
                    result.push_str(ident);
                    index = end;
                } else {
                    result.push_str("_ctx.");
                    index = start;
                }
            } else {
                result.push(bytes[index] as char);
                index += 1;
            }
        }

        result
    }

    /// Push raw code to the buffer
    pub(crate) fn push(&mut self, s: &str) {
        self.code.extend_from_slice(s.as_bytes());
    }

    /// Push indentation
    pub(crate) fn push_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.code.extend_from_slice(b"  ");
        }
    }

    /// Build the preamble with imports
    fn build_preamble(&self) -> String {
        let mut preamble = String::default();

        // SSR helpers from @vue/server-renderer
        if !self.ssr_helpers.is_empty() {
            preamble.push_str("import { ");
            let mut ssr_helpers: Vec<_> = self.ssr_helpers.iter().copied().collect();
            ssr_helpers.sort();
            let helpers: Vec<_> = ssr_helpers
                .iter()
                .map(|h| cstr!("{} as _{}", h.name(), h.name()))
                .collect();
            preamble.push_str(&helpers.join(", "));
            preamble.push_str(" } from \"@vue/server-renderer\"\n");
        }

        // Core helpers from vue
        if !self.core_helpers.is_empty() {
            preamble.push_str("import { ");
            let mut core_helpers: Vec<_> = self.core_helpers.iter().copied().collect();
            core_helpers.sort();
            let helpers: Vec<_> = core_helpers
                .iter()
                .map(|h| cstr!("{} as _{}", h.name(), h.name()))
                .collect();
            preamble.push_str(&helpers.join(", "));
            preamble.push_str(" } from \"vue\"\n");
        }

        preamble
    }
}

#[cfg(test)]
mod tests {
    use super::helpers::{escape_html, escape_html_attr};
    use super::SsrCodegenResult;

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<div>"), "&lt;div&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
        assert_eq!(escape_html("\"hello\""), "&quot;hello&quot;");
    }

    #[test]
    fn test_escape_html_all_special_chars() {
        assert_eq!(escape_html("&"), "&amp;");
        assert_eq!(escape_html("<"), "&lt;");
        assert_eq!(escape_html(">"), "&gt;");
        assert_eq!(escape_html("\""), "&quot;");
        assert_eq!(escape_html("'"), "&#39;");
    }

    #[test]
    fn test_escape_html_no_special() {
        assert_eq!(escape_html("hello world"), "hello world");
        assert_eq!(escape_html("abc123"), "abc123");
    }

    #[test]
    fn test_escape_html_attr() {
        assert_eq!(escape_html_attr("hello\"world"), "hello&quot;world");
        assert_eq!(escape_html_attr("a & b"), "a &amp; b");
    }

    #[test]
    fn test_escape_html_attr_preserves_angle_brackets() {
        // In attribute context, < and > do not need escaping
        assert_eq!(escape_html_attr("<foo>"), "<foo>");
        assert_eq!(escape_html_attr("a > b"), "a > b");
    }

    #[test]
    fn test_escape_html_attr_no_special() {
        assert_eq!(escape_html_attr("hello"), "hello");
    }

    #[test]
    fn test_ssr_codegen_result_default() {
        let result = SsrCodegenResult::default();
        assert!(result.code.is_empty());
        assert!(result.preamble.is_empty());
    }
}
