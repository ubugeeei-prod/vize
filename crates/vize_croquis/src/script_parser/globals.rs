//! Static global-name tables and the global scope hierarchy setup.
//!
//! Materializes the browser/Node/Vue global name lists once and installs the
//! `~universal → ~vue → ~mod` scope chain that every parse entry point seeds.

use std::sync::LazyLock;

use crate::scope::{JsGlobalScopeData, JsRuntime, ParamNames, ScopeChain, VueGlobalScopeData};
use vize_carton::CompactString;

/// Browser-only globals (WHATWG Living Standard + HTML timers).
///
/// Immutable, identical for every file. Kept as a static `&[&str]` so the
/// `CompactString` list (see `BROWSER_GLOBAL_NAMES`) is materialized once
/// instead of re-running the `smallvec!`/`const_new` construction per file.
static BROWSER_GLOBALS: &[&str] = &[
    "alert",
    "Audio",
    "cancelAnimationFrame",
    "cancelIdleCallback",
    "CanvasRenderingContext2D",
    "clearInterval",
    "clearTimeout",
    "close",
    "confirm",
    "customElements",
    "document",
    "Document",
    "DocumentFragment",
    "Element",
    "FocusEvent",
    "getComputedStyle",
    "getSelection",
    "history",
    "HTMLElement",
    "Image",
    "indexedDB",
    "InputEvent",
    "IntersectionObserver",
    "KeyboardEvent",
    "localStorage",
    "location",
    "matchMedia",
    "MediaQueryList",
    "MouseEvent",
    "MutationObserver",
    "navigator",
    "Node",
    "NodeList",
    "open",
    "PerformanceObserver",
    "PointerEvent",
    "print",
    "prompt",
    "queueMicrotask",
    "requestAnimationFrame",
    "requestIdleCallback",
    "ResizeObserver",
    "screen",
    "self",
    "sessionStorage",
    "setInterval",
    "setTimeout",
    "ShadowRoot",
    "TouchEvent",
    "WebGL2RenderingContext",
    "WebGLRenderingContext",
    "WebSocket",
    "window",
    "XMLHttpRequest",
];

/// Server-only globals (WinterCG extensions, ESM-based).
static NODE_GLOBALS: &[&str] = &["Buffer", "clearImmediate", "process", "setImmediate"];

/// Vue globals ($refs, $emit, $slots, $attrs, $el, etc.).
static VUE_GLOBALS: &[&str] = &[
    "$attrs",
    "$data",
    "$el",
    "$emit",
    "$forceUpdate",
    "$nextTick",
    "$options",
    "$parent",
    "$props",
    "$refs",
    "$root",
    "$slots",
    "$watch",
];

/// `ParamNames` (the owned form stored in each global scope) built once from the
/// static name lists. Each `enter_*_scope` consumes an owned list, so we clone
/// per file — but the `CompactString` values (all short enough to stay inline)
/// are constructed only once here rather than per file.
static BROWSER_GLOBAL_NAMES: LazyLock<ParamNames> =
    LazyLock::new(|| build_global_names(BROWSER_GLOBALS));
static NODE_GLOBAL_NAMES: LazyLock<ParamNames> = LazyLock::new(|| build_global_names(NODE_GLOBALS));
static VUE_GLOBAL_NAMES: LazyLock<ParamNames> = LazyLock::new(|| build_global_names(VUE_GLOBALS));

/// Materialize a `&'static [&str]` into the owned `ParamNames` form.
fn build_global_names(names: &'static [&'static str]) -> ParamNames {
    names
        .iter()
        .map(|&name| CompactString::const_new(name))
        .collect()
}

/// Setup global scopes hierarchy:
/// - ~universal (JS globals) - root, @0:0 (meta)
/// - ~vue (Vue globals) - parent: ~universal, @0:0 (meta)
/// - ~mod (module = SFC) - parent: ~universal, covers entire source
pub(super) fn setup_global_scopes(scopes: &mut ScopeChain, source_len: u32) {
    // Root is already ~js (JsGlobalUniversal) with common globals
    // Current scope is root (~js)

    // !client - Browser-only globals (WHATWG Living Standard + HTML timers)
    // Used as parent for onMounted, onUnmounted, etc.
    scopes.enter_js_global_scope(
        JsGlobalScopeData {
            runtime: JsRuntime::Browser,
            globals: BROWSER_GLOBAL_NAMES.clone(),
        },
        0,
        0,
    );
    scopes.exit_scope(); // Back to ~univ

    // #server - Server-only globals (WinterCG extensions, ESM-based)
    // Reserved for future SSR/Server Components support
    scopes.enter_js_global_scope(
        JsGlobalScopeData {
            runtime: JsRuntime::Node,
            globals: NODE_GLOBAL_NAMES.clone(),
        },
        0,
        0,
    );
    scopes.exit_scope(); // Back to ~univ

    // ~vue - Vue globals (parent: ~univ, meta scope)
    scopes.enter_vue_global_scope(
        VueGlobalScopeData {
            globals: VUE_GLOBAL_NAMES.clone(),
        },
        0,
        0,
    );
    scopes.exit_scope(); // Back to ~univ

    // ~mod - module scope (parent: ~js, covers entire SFC)
    scopes.enter_module_scope(0, source_len);
    // Stay in module scope - setup/plain will be created as children
}
