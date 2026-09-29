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
use oxc_allocator::Allocator;
use oxc_ast::ast::TSType;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_l0::String;
use vize_relief::BindingType;
use vize_relief::{ElementNode, ExpressionNode, InterpolationNode, RootNode};

/// Browser-only global names that are NOT available in SSR
const BROWSER_GLOBALS: &[&str] = &[
    // Window object and related
    "window",
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

/// Globals that are available in both browser and SSR runtimes.
const UNIVERSAL_GLOBALS: &[&str] = &["globalThis", "self"];

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
        !UNIVERSAL_GLOBALS.contains(&name) && BROWSER_GLOBALS.contains(&name)
    }

    /// Check if a name is a browser-only global using croquis analysis
    #[inline]
    fn is_browser_global_binding(ctx: &LintContext<'_>, name: &str) -> bool {
        if UNIVERSAL_GLOBALS.contains(&name) {
            return false;
        }

        if let Some(binding_type) = ctx.get_binding_type(name) {
            matches!(binding_type, BindingType::JsGlobalBrowser)
        } else {
            // Fall back to static list if analysis is not available
            Self::is_browser_global_static(name)
        }
    }

    /// A script or template binding shadows a browser global of the same name.
    ///
    /// Scope lookup finds ambient browser globals (`open`, `close`) before the
    /// script binding map, so a destructured prop is not "undefined" just
    /// because the global scope also has that name.
    fn is_local_binding(ctx: &LintContext<'_>, name: &str) -> bool {
        if ctx.is_v_for_var(name) || ctx.has_script_binding(name) {
            return true;
        }
        match ctx.get_binding_type(name) {
            Some(BindingType::JsGlobalBrowser) | None => false,
            Some(_) => true,
        }
    }

    /// Extract identifiers from an expression string.
    ///
    /// This method is aware of JavaScript syntax to avoid false positives:
    /// - Skips content inside string literals ('...', "...", `...`)
    /// - Skips content inside comments and regex literals (`/window/`)
    /// - Skips property access after `.` (e.g., `obj.top` → only `obj`)
    /// - Skips object property keys (e.g., `{ top: 0 }` → skips `top`)
    /// - Skips direct `typeof window` guards that are safe in SSR
    fn extract_identifiers(expr: &str) -> Vec<(&str, usize)> {
        let mut identifiers = Vec::new();
        let bytes = expr.as_bytes();
        let len = bytes.len();
        let at = |index: usize| bytes.get(index).copied();
        let mut i = 0;
        // Track whether the previous token was a `.` (property access)
        let mut after_dot = false;
        let mut after_typeof = false;
        let mut can_start_regex = true;

        while let Some(b) = at(i) {
            // Skip comments without changing the surrounding expression state.
            if b == b'/' && at(i + 1) == Some(b'/') {
                i += 2;
                while at(i).is_some_and(|c| !matches!(c, b'\n' | b'\r')) {
                    i += 1;
                }
                continue;
            }
            if b == b'/' && at(i + 1) == Some(b'*') {
                i += 2;
                while i + 1 < len {
                    if at(i) == Some(b'*') && at(i + 1) == Some(b'/') {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
                continue;
            }

            if b == b'/' && can_start_regex {
                i += 1;
                let mut in_character_class = false;
                while let Some(c) = at(i) {
                    if c == b'\\' {
                        i += 2;
                        continue;
                    }
                    if c == b'[' {
                        in_character_class = true;
                        i += 1;
                        continue;
                    }
                    if c == b']' {
                        in_character_class = false;
                        i += 1;
                        continue;
                    }
                    if c == b'/' && !in_character_class {
                        i += 1;
                        while at(i).is_some_and(|c| c.is_ascii_alphabetic()) {
                            i += 1;
                        }
                        break;
                    }
                    i += 1;
                }
                after_dot = false;
                after_typeof = false;
                can_start_regex = false;
                continue;
            }

            // Skip string literals
            if b == b'\'' || b == b'"' || b == b'`' {
                after_typeof = false;
                let quote = b;
                i += 1;
                while let Some(c) = at(i) {
                    if c == b'\\' {
                        i += 2; // skip escaped character
                        continue;
                    }
                    if c == quote {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                after_dot = false;
                can_start_regex = false;
                continue;
            }

            // Track dot for property access
            if b == b'.' {
                after_dot = true;
                can_start_regex = false;
                i += 1;
                continue;
            }

            // Identifier start
            if b.is_ascii_alphabetic() || b == b'_' || b == b'$' {
                let start = i;
                i += 1;
                while at(i).is_some_and(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'$') {
                    i += 1;
                }
                let ident = expr.get(start..i).unwrap_or_default();

                // Skip if it's a property access (after `.`)
                if after_dot {
                    after_dot = false;
                    after_typeof = false;
                    can_start_regex = false;
                    continue;
                }

                // Skip if it's an object property key (identifier followed by `:`)
                // Look ahead past whitespace for `:`
                let mut j = i;
                while at(j).is_some_and(|c| c.is_ascii_whitespace()) {
                    j += 1;
                }
                if at(j) == Some(b':') && at(j + 1) != Some(b':') {
                    // This is an object key like `{ top: 0 }`, skip it
                    after_dot = false;
                    after_typeof = false;
                    can_start_regex = true;
                    continue;
                }

                if ident == "typeof" {
                    after_typeof = true;
                    after_dot = false;
                    can_start_regex = true;
                    continue;
                }

                if after_typeof {
                    after_typeof = false;
                    let mut next = i;
                    while at(next).is_some_and(|c| c.is_ascii_whitespace()) {
                        next += 1;
                    }
                    if !matches!(at(next), Some(b'.' | b'[')) {
                        after_dot = false;
                        can_start_regex = false;
                        continue;
                    }
                }

                identifiers.push((ident, start));
                after_dot = false;
                can_start_regex = false;
                continue;
            }

            // Skip digits (number literals)
            if b.is_ascii_digit() {
                after_typeof = false;
                i += 1;
                while at(i).is_some_and(|c| c.is_ascii_alphanumeric() || c == b'.') {
                    i += 1;
                }
                after_dot = false;
                can_start_regex = false;
                continue;
            }

            // Any other character
            if !b.is_ascii_whitespace() {
                after_dot = false;
                if !matches!(b, b'(' | b')') {
                    after_typeof = false;
                }
                can_start_regex = matches!(
                    b,
                    b'(' | b'['
                        | b'{'
                        | b','
                        | b':'
                        | b';'
                        | b'?'
                        | b'='
                        | b'!'
                        | b'&'
                        | b'|'
                        | b'+'
                        | b'-'
                        | b'*'
                        | b'%'
                        | b'~'
                        | b'^'
                        | b'<'
                        | b'>'
                );
            }
            i += 1;
        }

        identifiers
    }

    fn runtime_identifiers(expr: &str) -> Vec<&str> {
        let identifiers = Self::extract_identifiers(expr);
        if !identifiers
            .iter()
            .any(|(name, _)| Self::is_browser_global_static(name))
        {
            return identifiers.into_iter().map(|(name, _)| name).collect();
        }

        let type_ranges = Self::type_ranges(expr);
        identifiers
            .into_iter()
            .filter(|(_, offset)| {
                !type_ranges
                    .iter()
                    .any(|(start, end)| *offset >= *start && *offset < *end)
            })
            .map(|(name, _)| name)
            .collect()
    }

    fn type_ranges(expr: &str) -> Vec<(usize, usize)> {
        const PREFIX: &str = "const __vize_ssr_expr = (";
        let mut source = String::with_capacity(PREFIX.len() + expr.len() + 2);
        source.push_str(PREFIX);
        source.push_str(expr);
        source.push_str(");");

        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source.as_str(), SourceType::ts()).parse();
        if parsed.panicked || !parsed.diagnostics.is_empty() {
            return Vec::new();
        }

        struct TypeRanges(Vec<(usize, usize)>);
        impl<'a> Visit<'a> for TypeRanges {
            fn visit_ts_type(&mut self, ty: &TSType<'a>) {
                let span = ty.span();
                self.0.push((span.start as usize, span.end as usize));
            }
        }

        let mut ranges = TypeRanges(Vec::new());
        ranges.visit_program(&parsed.program);
        ranges
            .0
            .into_iter()
            .filter_map(|(start, end)| {
                let offset = PREFIX.len();
                (start >= offset).then_some((start - offset, end.saturating_sub(offset)))
            })
            .collect()
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
            ExpressionNode::Simple(s) => s.content,
            ExpressionNode::Compound(_) => return, // Skip compound expressions for now
        };
        let identifiers = Self::runtime_identifiers(content);

        for ident in identifiers {
            if Self::is_local_binding(ctx, ident) {
                continue;
            }

            if Self::is_browser_global_binding(ctx, ident) {
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
        directive: &vize_relief::DirectiveNode<'a>,
    ) {
        // Only run if SSR mode is enabled
        if !ctx.is_ssr_enabled() {
            return;
        }

        // Check directive expressions
        if let Some(exp) = &directive.exp {
            let content = match exp {
                ExpressionNode::Simple(s) => s.content,
                ExpressionNode::Compound(_) => return, // Skip compound expressions
            };
            let identifiers = Self::runtime_identifiers(content);

            for ident in identifiers {
                if Self::is_local_binding(ctx, ident) {
                    continue;
                }

                if Self::is_browser_global_binding(ctx, ident) {
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
mod tests;
