//! Rule: no-browser-globals-in-ssr
//!
//! Warns when browser-only globals (window, document, navigator, etc.) are
//! accessed in SSR context (universal code that runs on server).
//!
//! ## Why is this bad?
//! In SSR (Server-Side Rendering), code runs on both server and client.
//! Browser-only globals like `window`, `document`, `navigator`, `localStorage`
//! are not available in Node.js/Deno/Bun environments and will cause errors.
//!
//! ## How to fix?
//! Move browser-only code to client-only lifecycle hooks:
//! - `onMounted` / `mounted`
//! - `onUpdated` / `updated`
//! - `onBeforeMount` / `beforeMount`
//! - `onBeforeUnmount` / `beforeUnmount`
//! - `onUnmounted` / `unmounted`
//! - `onActivated` / `activated`
//! - `onDeactivated` / `deactivated`
//!
//! ## Example
//!
//! Bad:
//! ```vue
//! <script setup>
//! // This will error in SSR!
//! const width = window.innerWidth;
//! </script>
//! ```
//!
//! Good:
//! ```vue
//! <script setup>
//! import { ref, onMounted } from 'vue';
//!
//! const width = ref(0);
//!
//! onMounted(() => {
//!   // Safe - only runs on client
//!   width.value = window.innerWidth;
//! });
//! </script>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::{ElementNode, ExpressionNode, InterpolationNode, RootNode};
use vize_relief::BindingType;

/// Browser-only global names that are NOT available in SSR
const BROWSER_GLOBALS: &[&str] = &[
    // Window object and related
    "window",
    "self",
    "globalThis", // In browsers only (also exists in Node.js 12+, but with different behavior)
    // Document object
    "document",
    // Navigator
    "navigator",
    // Location/History
    "location",
    "history",
    // Storage
    "localStorage",
    "sessionStorage",
    "indexedDB",
    // Timers (exist in Node.js but may behave differently)
    // "setTimeout", "setInterval", "requestAnimationFrame", // These are often polyfilled
    // Web APIs
    "requestAnimationFrame",
    "cancelAnimationFrame",
    "requestIdleCallback",
    "cancelIdleCallback",
    "ResizeObserver",
    "IntersectionObserver",
    "MutationObserver",
    "PerformanceObserver",
    // DOM
    "HTMLElement",
    "Element",
    "Node",
    "Event",
    "CustomEvent",
    "MouseEvent",
    "KeyboardEvent",
    "TouchEvent",
    "DragEvent",
    // Media
    "Audio",
    "Image",
    "MediaRecorder",
    "MediaSource",
    "MediaStream",
    // Canvas/WebGL
    "CanvasRenderingContext2D",
    "WebGLRenderingContext",
    "WebGL2RenderingContext",
    // Geolocation
    "geolocation",
    // Screen
    "screen",
    "innerWidth",
    "innerHeight",
    "outerWidth",
    "outerHeight",
    "scrollX",
    "scrollY",
    "pageXOffset",
    "pageYOffset",
    // Clipboard
    "clipboard",
    // Speech
    "speechSynthesis",
    "SpeechRecognition",
    // Notification
    "Notification",
    // WebSocket (exists in Node.js but may need import)
    // "WebSocket",
    // Worker
    "Worker",
    "SharedWorker",
    "ServiceWorker",
    // Crypto (exists in Node.js but differently)
    // "crypto", // Node.js has crypto module
    // Performance (exists in Node.js but differently)
    // "performance",
    // Fetch (polyfilled in Node.js 18+)
    // "fetch",
    // Alert/Confirm/Prompt
    "alert",
    "confirm",
    "prompt",
    // Open/Close
    "open",
    "close",
    "print",
    // Frame related
    "frames",
    "parent",
    "top",
    "opener",
    // CSS
    "CSS",
    "CSSStyleSheet",
    "getComputedStyle",
    "matchMedia",
];

static META: RuleMeta = RuleMeta {
    name: "ssr/no-browser-globals-in-ssr",
    description: "Disallow browser-only globals in SSR context",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

pub struct NoBrowserGlobalsInSsr;

impl NoBrowserGlobalsInSsr {
    /// Check if a name is a browser-only global (using static list)
    #[inline]
    fn is_browser_global_static(name: &str) -> bool {
        BROWSER_GLOBALS.contains(&name)
    }

    /// Check if a name is a browser-only global using croquis analysis
    #[inline]
    fn is_browser_global_binding(ctx: &LintContext<'_>, name: &str) -> bool {
        if let Some(binding_type) = ctx.get_binding_type(name) {
            matches!(binding_type, BindingType::JsGlobalBrowser)
        } else {
            // Fall back to static list if analysis is not available
            Self::is_browser_global_static(name)
        }
    }

    /// Extract identifiers from an expression string.
    ///
    /// This method is aware of JavaScript syntax to avoid false positives:
    /// - Skips content inside string literals ('...', "...", `...`)
    /// - Skips property access after `.` (e.g., `obj.top` → only `obj`)
    /// - Skips object property keys (e.g., `{ top: 0 }` → skips `top`)
    fn extract_identifiers(expr: &str) -> Vec<&str> {
        let mut identifiers = Vec::new();
        let bytes = expr.as_bytes();
        let len = bytes.len();
        let mut i = 0;
        // Track whether the previous token was a `.` (property access)
        let mut after_dot = false;

        while i < len {
            let b = bytes[i];

            // Skip string literals
            if b == b'\'' || b == b'"' || b == b'`' {
                let quote = b;
                i += 1;
                while i < len {
                    if bytes[i] == b'\\' {
                        i += 2; // skip escaped character
                        continue;
                    }
                    if bytes[i] == quote {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                after_dot = false;
                continue;
            }

            // Track dot for property access
            if b == b'.' {
                after_dot = true;
                i += 1;
                continue;
            }

            // Identifier start
            if b.is_ascii_alphabetic() || b == b'_' || b == b'$' {
                let start = i;
                i += 1;
                while i < len
                    && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'$')
                {
                    i += 1;
                }
                let ident = &expr[start..i];

                // Skip if it's a property access (after `.`)
                if after_dot {
                    after_dot = false;
                    continue;
                }

                // Skip if it's an object property key (identifier followed by `:`)
                // Look ahead past whitespace for `:`
                let mut j = i;
                while j < len && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < len && bytes[j] == b':' && (j + 1 >= len || bytes[j + 1] != b':') {
                    // This is an object key like `{ top: 0 }`, skip it
                    after_dot = false;
                    continue;
                }

                identifiers.push(ident);
                after_dot = false;
                continue;
            }

            // Skip digits (number literals)
            if b.is_ascii_digit() {
                i += 1;
                while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'.') {
                    i += 1;
                }
                after_dot = false;
                continue;
            }

            // Any other character
            if !b.is_ascii_whitespace() {
                after_dot = false;
            }
            i += 1;
        }

        identifiers
    }
}

impl Rule for NoBrowserGlobalsInSsr {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, _ctx: &mut LintContext<'a>, _root: &RootNode<'a>) {
        // Template-level checking is done via check_interpolation
    }

    fn check_interpolation<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        interpolation: &InterpolationNode<'a>,
    ) {
        // Only run if SSR mode is enabled
        if !ctx.is_ssr_enabled() {
            return;
        }

        let content = match &interpolation.content {
            ExpressionNode::Simple(s) => s.content.as_str(),
            ExpressionNode::Compound(_) => return, // Skip compound expressions for now
        };
        let identifiers = Self::extract_identifiers(content);

        for ident in identifiers {
            // Skip if it's defined as a local variable (from v-for, etc.)
            if ctx.is_variable_defined(ident) {
                continue;
            }

            // Check using croquis analysis or fall back to static list
            if Self::is_browser_global_binding(ctx, ident) || Self::is_browser_global_static(ident)
            {
                ctx.warn_with_help(
                    ctx.t_fmt("ssr/no-browser-globals-in-ssr.message", &[("name", ident)]),
                    &interpolation.loc,
                    ctx.t("ssr/no-browser-globals-in-ssr.help"),
                );
            }
        }
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &vize_relief::ast::DirectiveNode<'a>,
    ) {
        // Only run if SSR mode is enabled
        if !ctx.is_ssr_enabled() {
            return;
        }

        // Check directive expressions
        if let Some(exp) = &directive.exp {
            let content = match exp {
                ExpressionNode::Simple(s) => s.content.as_str(),
                ExpressionNode::Compound(_) => return, // Skip compound expressions
            };
            let identifiers = Self::extract_identifiers(content);

            for ident in identifiers {
                // Skip if it's defined as a local variable
                if ctx.is_variable_defined(ident) {
                    continue;
                }

                // Check using croquis analysis or fall back to static list
                if Self::is_browser_global_binding(ctx, ident)
                    || Self::is_browser_global_static(ident)
                {
                    ctx.warn_with_help(
                        ctx.t_fmt("ssr/no-browser-globals-in-ssr.message", &[("name", ident)]),
                        &directive.loc,
                        ctx.t("ssr/no-browser-globals-in-ssr.help"),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoBrowserGlobalsInSsr;
    use crate::context::{LintContext, SsrMode};
    use crate::rule::{Rule, RuleRegistry};
    use crate::Linter;

    fn lint_with_ssr(source: &str) -> Vec<String> {
        let mut registry = RuleRegistry::new();
        registry.add(Box::new(NoBrowserGlobalsInSsr));
        let _linter = Linter::with_registry(registry);

        // Create allocator and context
        use vize_carton::Allocator;
        let allocator = Allocator::with_capacity(1024);
        let mut ctx = LintContext::with_locale(
            &allocator,
            source,
            "test.vue",
            crate::Linter::default().locale(),
        );
        ctx.set_ssr_mode(SsrMode::Enabled);

        let parser = vize_armature::Parser::new(allocator.as_bump(), source);
        let (root, _) = parser.parse();

        let rules: Vec<Box<dyn Rule>> = vec![Box::new(NoBrowserGlobalsInSsr)];
        let mut visitor = crate::visitor::LintVisitor::new(&mut ctx, &rules);
        visitor.visit_root(&root);

        ctx.into_diagnostics()
            .into_iter()
            .map(|d| d.message.to_string())
            .collect()
    }

    #[test]
    fn test_detects_window_in_interpolation() {
        let result = lint_with_ssr("<div>{{ window.innerWidth }}</div>");
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_detects_document_in_interpolation() {
        let result = lint_with_ssr("<div>{{ document.title }}</div>");
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_detects_navigator_in_directive() {
        let result = lint_with_ssr("<div :class=\"navigator.userAgent\"></div>");
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_allows_local_variable() {
        // If 'window' is a local variable (e.g., from v-for), it should be allowed
        let result = lint_with_ssr("<div v-for=\"window in windows\">{{ window }}</div>");
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_detects_localstorage() {
        let result = lint_with_ssr("<div>{{ localStorage.getItem('key') }}</div>");
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_ignores_css_property_names_in_style_object() {
        // { top: 0 } - 'top' is an object key, not a reference to window.top
        let result =
            lint_with_ssr(r#"<div :style="{ position: 'absolute', top: 0, left: 0 }"></div>"#);
        assert!(
            result.is_empty(),
            "Should not flag CSS property names in style objects, got: {:?}",
            result
        );
    }

    #[test]
    fn test_ignores_string_literal_values() {
        // 'window' is a string literal, not a reference to the window global
        let result = lint_with_ssr(r#"<div :class="'window'"></div>"#);
        assert!(
            result.is_empty(),
            "Should not flag string literals, got: {:?}",
            result
        );
    }

    #[test]
    fn test_ignores_property_access() {
        // obj.top - 'top' is a property access, not a reference to window.top
        let result = lint_with_ssr(r#"<div>{{ obj.top }}</div>"#);
        // Only 'obj' should be checked, not 'top'
        assert!(
            result.is_empty(),
            "Should not flag property accesses, got: {:?}",
            result
        );
    }

    #[test]
    fn test_detects_actual_global_in_style_value() {
        // { top: window.scrollY } - 'window' is a real global reference
        let result = lint_with_ssr(r#"<div :style="{ top: window.scrollY + 'px' }"></div>"#);
        insta::assert_debug_snapshot!(result);
    }
}
