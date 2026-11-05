//! The standard global scopes every file seeds, built once per process.
//!
//! Each table is the map the per-file insert loop built, inserted in the same
//! order, so lookups and iteration answer exactly as before. A scope borrows
//! its table until the file writes to it (see [`super::Scope`]).

use std::sync::LazyLock;

use vize_carton::{CompactString, FxHashMap};
use vize_relief::BindingType;

use super::ScopeChain;
use super::types::{
    JsGlobalScopeData, JsRuntime, ParamNames, ScopeBinding, ScopeData, ScopeKind,
    VueGlobalScopeData,
};

pub(crate) type BindingMap = FxHashMap<CompactString, ScopeBinding>;

/// One standard global scope: its kind, bindings and data.
pub(crate) struct StandardGlobals {
    pub(crate) kind: ScopeKind,
    pub(crate) bindings: BindingMap,
    pub(crate) data: ScopeData,
}

/// ECMAScript standard built-in globals (ECMA-262), the root scope.
pub(crate) const JS_UNIVERSAL_GLOBALS: &[&str] = &[
    "AggregateError",
    "arguments", // Function scope closure
    "Array",
    "ArrayBuffer",
    "AsyncFunction",
    "AsyncGenerator",
    "AsyncGeneratorFunction",
    "AsyncIterator",
    "Atomics",
    "BigInt",
    "BigInt64Array",
    "BigUint64Array",
    "Boolean",
    "console", // Non-standard but universally available
    "DataView",
    "Date",
    "decodeURI",
    "decodeURIComponent",
    "encodeURI",
    "encodeURIComponent",
    "Error",
    "eval",
    "EvalError",
    "Float32Array",
    "Float64Array",
    "Function",
    "Generator",
    "GeneratorFunction",
    "globalThis",
    "Infinity",
    "Int16Array",
    "Int32Array",
    "Int8Array",
    "Intl",
    "isFinite",
    "isNaN",
    "Iterator",
    "JSON",
    "Map",
    "Math",
    "NaN",
    "Number",
    "Object",
    "parseFloat",
    "parseInt",
    "Promise",
    "Proxy",
    "RangeError",
    "ReferenceError",
    "Reflect",
    "RegExp",
    "Set",
    "SharedArrayBuffer",
    "String",
    "Symbol",
    "SyntaxError",
    "this", // Function scope closure
    "TypeError",
    "Uint16Array",
    "Uint32Array",
    "Uint8Array",
    "Uint8ClampedArray",
    "undefined",
    "URIError",
    "WeakMap",
    "WeakSet",
];

/// Browser-only globals (WHATWG Living Standard + HTML timers).
pub(crate) const BROWSER_GLOBALS: &[&str] = &[
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
pub(crate) const NODE_GLOBALS: &[&str] = &["Buffer", "clearImmediate", "process", "setImmediate"];

/// Vue globals ($refs, $emit, $slots, $attrs, $el, etc.).
pub(crate) const VUE_GLOBALS: &[&str] = &[
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

/// `~univ`: the chain root.
pub(crate) static UNIVERSAL: LazyLock<StandardGlobals> = LazyLock::new(|| StandardGlobals {
    kind: ScopeKind::JsGlobalUniversal,
    bindings: bindings(JS_UNIVERSAL_GLOBALS, BindingType::JsGlobalUniversal),
    data: ScopeData::None,
});

/// `!client`: parent of client-only hooks such as `onMounted`.
pub(crate) static BROWSER: LazyLock<StandardGlobals> =
    LazyLock::new(|| js_runtime(JsRuntime::Browser, BROWSER_GLOBALS));

/// `#server`: reserved for SSR / server components.
pub(crate) static NODE: LazyLock<StandardGlobals> =
    LazyLock::new(|| js_runtime(JsRuntime::Node, NODE_GLOBALS));

/// `~vue`: the extra parent of template scopes.
pub(crate) static VUE: LazyLock<StandardGlobals> = LazyLock::new(|| StandardGlobals {
    kind: ScopeKind::VueGlobal,
    bindings: bindings(VUE_GLOBALS, BindingType::VueGlobal),
    data: ScopeData::VueGlobal(VueGlobalScopeData {
        globals: names(VUE_GLOBALS),
    }),
});

fn js_runtime(runtime: JsRuntime, globals: &'static [&'static str]) -> StandardGlobals {
    StandardGlobals {
        kind: runtime.to_scope_kind(),
        bindings: bindings(globals, runtime.to_binding_type()),
        data: ScopeData::JsGlobal(JsGlobalScopeData {
            runtime,
            globals: names(globals),
        }),
    }
}

fn bindings(names: &[&str], binding_type: BindingType) -> BindingMap {
    let mut bindings = BindingMap::default();
    for name in names {
        bindings.insert(CompactString::new(name), ScopeBinding::new(binding_type, 0));
    }
    bindings
}

fn names(names: &[&str]) -> ParamNames {
    names.iter().map(|&name| CompactString::new(name)).collect()
}

/// Seed the global scope hierarchy every parse entry point starts from:
/// - ~universal (JS globals) - root, @0:0 (meta)
/// - !client, #server and ~vue - parent: ~universal, @0:0 (meta)
/// - ~mod (module = SFC) - parent: ~universal, covers entire source
pub(crate) fn setup_global_scopes(scopes: &mut ScopeChain, source_len: u32) {
    // Current scope is the ~univ root. !client parents client-only hooks
    // (onMounted, ...); #server is reserved for SSR / server components.
    for globals in [&BROWSER, &NODE, &VUE] {
        scopes.enter_standard_global_scope(globals);
        scopes.exit_scope();
    }
    // Stay in ~mod: setup/plain scopes are created as its children.
    scopes.enter_module_scope(0, source_len);
}
