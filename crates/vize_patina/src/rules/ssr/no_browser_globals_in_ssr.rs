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
use vize_relief::BindingType;

mod script_layout;
mod script_reads;
mod script_symbols;
mod template_identifiers;
mod type_ranges;
mod typeof_guard;
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
}

impl Rule for NoBrowserGlobalsInSsr {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        script_reads::check(ctx);
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
mod script_scope_tests;
#[cfg(test)]
mod script_tests;
#[cfg(test)]
mod tests;
