//! Helper functions and constants for virtual TypeScript generation.
//!
//! Contains utility functions for type declarations, event type mapping,
//! identifier conversion, and template context generation.

use std::ops::Range;

use super::types::VirtualTsOptions;
use vize_carton::String;
use vize_carton::append;
use vize_carton::config::VueVersion;
use vize_croquis::macros::{
    DEFINE_EMITS, DEFINE_EXPOSE, DEFINE_MODEL, DEFINE_PROPS, DEFINE_SLOTS, WITH_DEFAULTS,
};

pub(crate) const USE_TEMPLATE_REF: &str = "useTemplateRef";

/// Names declared by the generated setup-scope helper block.
///
/// This includes Vue compiler macros plus runtime helper shims that are modeled
/// inside `__setup()`. It is intentionally broader than `COMPILER_MACRO_NAMES`.
pub(crate) const SETUP_SCOPE_HELPER_NAMES: &[&str] = &[
    DEFINE_PROPS,
    DEFINE_EMITS,
    DEFINE_EXPOSE,
    DEFINE_MODEL,
    DEFINE_SLOTS,
    WITH_DEFAULTS,
    USE_TEMPLATE_REF,
];

/// Shared type-helper text used both by the per-file embedded preamble and by
/// the hoisted ambient helpers file. Declared as a macro so the exact same
/// bytes can be spliced into both constants at compile time.
macro_rules! vue_type_aliases_text {
    () => {
        r#"type __EmitShape<T> = T extends (...args: any[]) => any ? T : T extends Record<string, any> ? { [K in keyof T]: T[K] extends (...args: infer A) => any ? A : T[K] extends any[] ? T[K] : any[]; } : Record<string, any[]>;
type __EmitArgs<T, K extends keyof T> = T[K] extends any[] ? T[K] : any[];
type __EmitFn<T> = __EmitShape<T> extends (...args: any[]) => any ? __EmitShape<T> : (<K extends keyof __EmitShape<T>>(event: K, ...args: __EmitArgs<__EmitShape<T>, K>) => void);
type __RuntimePropValue<T> = T extends { new (...args: any[]): infer V } ? V : T extends { (): infer V } ? V : never;
type __RuntimePropCtorInner<T> = T extends null | undefined ? never : T extends readonly (infer U)[] ? __RuntimePropCtorInner<U> : T extends { type: infer U } ? __RuntimePropCtorInner<U> : T extends StringConstructor ? string : T extends NumberConstructor ? number : T extends BooleanConstructor ? boolean : T extends ArrayConstructor ? unknown[] : T extends ObjectConstructor ? Record<string, unknown> : T extends DateConstructor ? Date : T extends FunctionConstructor ? (...args: any[]) => any : __RuntimePropValue<T>;
type __RuntimePropCtor<T> = [__RuntimePropCtorInner<T>] extends [never] ? unknown : __RuntimePropCtorInner<T>;
type __RuntimePropResolved<T> = T extends { required: true } ? true : T extends { default: any } ? true : false;
type __RuntimePropShape<T extends Record<string, any>> = { [K in keyof T]: __RuntimePropResolved<T[K]> extends true ? __RuntimePropCtor<T[K]> : __RuntimePropCtor<T[K]> | undefined; };
type __DefaultFactory<T> = (props: any) => T;
type __WithDefaultValue<T> = T | __DefaultFactory<T>;
type __WithDefaultsArgs<T> = { [K in keyof T]?: __WithDefaultValue<T[K]> };
type __WithDefaultsResult<T, D extends __WithDefaultsArgs<T>> = Omit<T, keyof D> & Required<Pick<T, keyof D & keyof T>>;
type __Ref<T> = import('vue').Ref<T>;
type __ShallowRef<T> = import('vue').ShallowRef<T>;
type __VizeKebabCase<S extends string> = S extends `${infer Head}${infer Tail}` ? Head extends Lowercase<Head> ? `${Head}${__VizeKebabCase<Tail>}` : `-${Lowercase<Head>}${__VizeKebabCase<Tail>}` : S;
type __VizeKebabProps<T> = { [K in keyof T & string as __VizeKebabCase<K>]: T[K] };
type __VizeComponentProps<T> = T extends unknown ? T & Partial<__VizeKebabProps<T>> : never;"#
    };
}

macro_rules! v_for_list_decls_text {
    () => {
        r#"declare function __vForList<T>(source: readonly T[] | undefined | null): readonly [item: T, key: number, index: number][];
declare function __vForList(source: number | undefined | null): readonly [item: number, key: number, index: number][];
declare function __vForList(source: string | undefined | null): readonly [item: string, key: number, index: number][];
declare function __vForList<T>(source: Iterable<T> | undefined | null): readonly [item: T, key: number, index: number][];
declare function __vForList<T extends object>(source: T | undefined | null): readonly [item: T[keyof T], key: keyof T, index: number][];"#
    };
}

macro_rules! vue_type_helpers_text {
    () => {
        concat!(vue_type_aliases_text!(), "\n", v_for_list_decls_text!())
    };
}

/// Emit-overload helper text shared between the per-file embedded emission and
/// the hoisted ambient helpers file. Each line ends with `\n`.
///
/// Deliberately excludes `__EmitProps`: that alias is emitted per-file and only
/// for components that actually declare emits, so it stays out of the shared
/// hoisted helper text, exactly as before hoisting.
macro_rules! emit_overload_helpers_text {
    () => {
        concat!(
            "type __VizeOverloadProps<TOverload> = Pick<TOverload, keyof TOverload>;\n",
            "type __VizeOverloadUnionRecursive<TOverload, TPartialOverload = unknown> = TOverload extends (...args: infer TArgs) => infer TReturn ? TPartialOverload extends TOverload ? never : __VizeOverloadUnionRecursive<TPartialOverload & TOverload, TPartialOverload & ((...args: TArgs) => TReturn) & __VizeOverloadProps<TOverload>> | ((...args: TArgs) => TReturn) : never;\n",
            "type __VizeOverloadUnion<TOverload extends (...args: any[]) => any> = Exclude<__VizeOverloadUnionRecursive<(() => never) & TOverload>, TOverload extends () => never ? never : () => never>;\n",
            "type __VizeOverloadParameters<T extends (...args: any[]) => any> = Parameters<__VizeOverloadUnion<T>>;\n",
            "type __VizeIsStringLiteral<T> = T extends string ? string extends T ? false : true : false;\n",
            "type __VizeParametersToFns<T extends any[]> = { [K in T[0]]: __VizeIsStringLiteral<K> extends true ? (...args: T extends [e: infer E, ...args: infer P] ? K extends E ? P : never : never) => any : never };\n",
            "type __EmitOptions<T> = { [K in keyof __EmitShape<T> & string]: (...args: __EmitArgs<__EmitShape<T>, K>) => any } & (__EmitShape<T> extends (...args: any[]) => any ? __VizeParametersToFns<__VizeOverloadParameters<__EmitShape<T>>> : {});\ntype __VizeCamelize<S extends string> = S extends `${infer Head}-${infer Tail}` ? `${Head}${Capitalize<__VizeCamelize<Tail>>}` : S;\ntype __VizeHandlerKey<K extends string> = `on${Capitalize<__VizeCamelize<K>>}`;\n",
        )
    };
}

/// Shared type helpers used by generated virtual modules and setup-scope macros.
pub(crate) const VUE_TYPE_HELPERS: &str = vue_type_helpers_text!();

/// Emit-overload helpers embedded per-file when the shared preamble is not
/// hoisted. In hoisted mode the same text lives in the ambient helpers file.
pub(crate) const EMIT_OVERLOAD_HELPERS: &str = emit_overload_helpers_text!();

/// Per-file `__EmitProps` alias used only by components that declare emits.
pub(crate) const EMIT_PROPS_HELPER: &str = "type __EmitProps<T> = { [K in keyof __EmitOptions<T> & string as __VizeHandlerKey<K>]?: __EmitOptions<T>[K] };\n";

/// Vue setup-scope helpers - these are defined inside setup scope, NOT globally.
/// Compiler macros stay setup-only, while runtime helper shims model Vue APIs.
/// Parameters and type parameters are prefixed with _ to avoid "unused" warnings.
pub(crate) const VUE_SETUP_HELPERS: &str = r#"  // Compiler macros (only valid in setup scope, not global)
  function defineProps<_T = unknown>(): _T;
  function defineProps<const _T extends readonly string[]>(_props: _T): { [K in _T[number]]?: any };
  function defineProps<const _T extends Record<string, any>>(_props: _T): __RuntimePropShape<_T>;
  function defineProps(_props?: any) { void _props; return undefined as any; }
  function defineEmits<_T = unknown>(): __EmitFn<_T>;
  function defineEmits<const _T extends readonly string[]>(_events: _T): (event: _T[number], ...args: any[]) => void;
  function defineEmits<const _T extends Record<string, any>>(_events: _T): __EmitFn<_T>;
  function defineEmits(_events?: any) { void _events; return (() => {}) as any; }
  function defineExpose<_T = unknown>(_exposed?: _T): void { void _exposed; }
  function defineModel<_T = unknown>(): __Ref<_T | undefined>;
  function defineModel<_T = unknown>(_options: any): __Ref<_T>;
  function defineModel<_T = unknown>(_name: string, _options?: any): __Ref<_T>;
  function defineModel(_name_or_options?: any, _options?: any) { void _name_or_options; void _options; return undefined as any; }
  function defineSlots<_T = unknown>(): _T { return undefined as unknown as _T; }
  function withDefaults<_T, _D extends __WithDefaultsArgs<_T>>(_props: _T, _defaults: _D): __WithDefaultsResult<_T, _D> { void _props; void _defaults; return undefined as unknown as __WithDefaultsResult<_T, _D>; }
  function useTemplateRef<_T = any>(_key: string): __ShallowRef<_T | null> { void _key; return undefined as unknown as __ShallowRef<_T | null>; }
  // Mark compiler macros as used
  void defineProps; void defineEmits; void defineExpose; void defineModel; void defineSlots; void withDefaults; void useTemplateRef;"#;

/// ImportMeta augmentation for Vite/Nuxt projects.
/// Uses `/// <reference types="..." />` to pull in existing type definitions
/// from frameworks like Vite, Nuxt, etc. when available.
pub(crate) const IMPORT_META_AUGMENTATION: &str = r#"// ImportMeta augmentation (reference existing framework types)
/// <reference types="vite/client" />
declare global {
  // Extend ImportMeta with Nuxt-specific properties not covered by vite/client
  interface ImportMeta {
    client: boolean;
    server: boolean;
    dev: boolean;
    prod: boolean;
    ssr: boolean;
  }
}
"#;

/// Per-file setup-scope helper block emitted when the shared preamble is
/// hoisted: the macro signatures live once in the ambient helpers file as
/// `__vize_*` globals and are aliased into setup scope here, so compiler
/// macros stay setup-scope-only and still shadow same-named module imports
/// exactly like the embedded `function` declarations did.
pub(crate) const VUE_SETUP_HELPERS_HOISTED: &str = r#"  // Compiler macros (setup-scope only; signatures hoisted to the shared helpers file)
  const defineProps = __vize_defineProps;
  const defineEmits = __vize_defineEmits;
  const defineExpose = __vize_defineExpose;
  const defineModel = __vize_defineModel;
  const defineSlots = __vize_defineSlots;
  const withDefaults = __vize_withDefaults;
  const useTemplateRef = __vize_useTemplateRef;
  // Mark compiler macros as used
  void defineProps; void defineEmits; void defineExpose; void defineModel; void defineSlots; void withDefaults; void useTemplateRef;"#;

/// File name of the shared ambient helpers declaration materialized once per
/// program when the preamble is hoisted out of the generated virtual modules.
pub const SHARED_PREAMBLE_FILE_NAME: &str = "__vize_helpers.d.ts";

/// Content of the shared ambient helpers file.
///
/// This file is a global script (no imports/exports), so every declaration
/// merges into the program's global scope exactly once:
///
/// - the `ImportMeta` augmentation becomes a plain global interface merge,
///   so generated modules stop carrying per-file `declare global` blocks
///   (which made every module a global-scope augmenter and defeated
///   incremental rebuilds);
/// - the generic type helpers ([`VUE_TYPE_HELPERS`] /
///   [`EMIT_OVERLOAD_HELPERS`]) are file-independent and hoist verbatim;
/// - the compiler-macro signatures are declared as `__vize_*` global
///   functions with byte-identical signatures (same overload order, type
///   parameter and parameter names) and aliased into each module's
///   `__setup()` scope, preserving exact overload parity while keeping the
///   macros invalid outside setup scope.
pub const SHARED_PREAMBLE_DTS: &str = concat!(
    "// ============================================\n",
    "// Shared ambient helpers for vize virtual TypeScript\n",
    "// Generated by vize\n",
    "// ============================================\n",
    "// Global script: one copy of these declarations per program replaces the\n",
    "// preamble previously embedded in every generated .vue.ts module.\n",
    "\n",
    "// ImportMeta augmentation (reference existing framework types)\n",
    "/// <reference types=\"vite/client\" />\n",
    "// Extend ImportMeta with Nuxt-specific properties not covered by vite/client\n",
    "interface ImportMeta {\n",
    "  client: boolean;\n",
    "  server: boolean;\n",
    "  dev: boolean;\n",
    "  prod: boolean;\n",
    "  ssr: boolean;\n",
    "}\n\ndeclare namespace JSX { interface IntrinsicAttributes { class?: unknown; style?: unknown; } }\ndeclare module 'vue/jsx-runtime' { export namespace JSX { interface IntrinsicAttributes { class?: unknown; style?: unknown; } } }\n",
    "\n",
    "// Shared type helpers used by generated virtual modules\n",
    vue_type_helpers_text!(),
    "\n\n",
    "// Emit-overload helpers (consumed by the per-file __EmitProps alias)\n",
    emit_overload_helpers_text!(),
    "\n",
    "// Compiler-macro signatures (aliased inside each module's __setup() scope)\n",
    "declare function __vize_defineProps<_T = unknown>(): _T;\n",
    "declare function __vize_defineProps<const _T extends readonly string[]>(_props: _T): { [K in _T[number]]?: any };\n",
    "declare function __vize_defineProps<const _T extends Record<string, any>>(_props: _T): __RuntimePropShape<_T>;\n",
    "declare function __vize_defineEmits<_T = unknown>(): __EmitFn<_T>;\n",
    "declare function __vize_defineEmits<const _T extends readonly string[]>(_events: _T): (event: _T[number], ...args: any[]) => void;\n",
    "declare function __vize_defineEmits<const _T extends Record<string, any>>(_events: _T): __EmitFn<_T>;\n",
    "declare function __vize_defineExpose<_T = unknown>(_exposed?: _T): void;\n",
    "declare function __vize_defineModel<_T = unknown>(): __Ref<_T | undefined>;\n",
    "declare function __vize_defineModel<_T = unknown>(_options: any): __Ref<_T>;\n",
    "declare function __vize_defineModel<_T = unknown>(_name: string, _options?: any): __Ref<_T>;\n",
    "declare function __vize_defineSlots<_T = unknown>(): _T;\n",
    "declare function __vize_withDefaults<_T, _D extends __WithDefaultsArgs<_T>>(_props: _T, _defaults: _D): __WithDefaultsResult<_T, _D>;\n",
    "declare function __vize_useTemplateRef<_T = any>(_key: string): __ShallowRef<_T | null>;\n",
);

/// Helper declarations shipped next to emitted declaration outputs.
///
/// Emitted `.vue.d.ts` files reference the shared helper type aliases by
/// name (e.g. `__EmitFn<Emits>`), so declaration output directories carry one
/// copy of the type aliases, wired up via a `/// <reference path>` from each
/// emitted file. Deliberately excludes the `vite/client` reference, the
/// `ImportMeta` augmentation, and the value-level macro declarations: emitted
/// declarations never reference those, and they must not leak into consumer
/// programs.
pub const DECLARATION_HELPERS_DTS: &str = concat!(
    "// Shared helper types for vize-generated declaration files.\n",
    "// Generated by vize\n",
    vue_type_aliases_text!(),
    "\n",
    emit_overload_helpers_text!(),
);

/// Vue 2-only public-instance members that are absent from Vue 3's
/// `ComponentPublicInstance`.
///
/// In a Vue 2 / 2.7 dialect, template (and `this`) references such as
/// `$listeners`, `$children`, `$scopedSlots`, the `$on`/`$off`/`$once` event
/// emitter, `$set`/`$delete`, and `$createElement`/`_c` are valid but resolve
/// to nothing on the Vue 3 instance type, so Corsa would false-error on them.
/// They are emitted as permissive `any` bindings so v2 templates type-check.
/// Vue 3 output never emits these, so it stays byte-identical.
const VUE2_INSTANCE_MEMBERS: &[&str] = &[
    "$listeners",
    "$children",
    "$scopedSlots",
    "$on",
    "$off",
    "$once",
    "$set",
    "$delete",
    "$createElement",
    "_c",
];

/// Generate Vue template context declarations dynamically.
///
/// Uses Vue's `ComponentPublicInstance` for Vue 3. In legacy Vue 2 mode, emits
/// a structural fallback because Vue 2.6 does not export that Vue 3 helper type.
pub(crate) fn generate_template_context(
    options: &VirtualTsOptions,
    dialect: VueVersion,
    legacy_vue2: bool,
) -> String {
    let mut ctx = String::default();

    let needs_global_helper =
        !options.template_globals.is_empty() || !options.css_modules.is_empty();

    // Instance type + conditional accessor helper
    let vue2_dialect = legacy_vue2 || matches!(dialect, VueVersion::V2 | VueVersion::V2_7);
    if vue2_dialect {
        ctx.push_str("    // Vue template context (Vue 2-compatible structural fallback)\n    type __Ctx = { $attrs: Record<string, unknown>; $slots: Record<string, unknown>; $refs: Record<string, any>; $emit: (...args: any[]) => void; };\n");
    } else {
        ctx.push_str("    // Vue template context (delegates to ComponentPublicInstance)\n    type __Ctx = import('vue').ComponentPublicInstance;\n");
    }
    if needs_global_helper {
        ctx.push_str("    type __Global<K extends string, F = unknown> = K extends keyof __Ctx ? __Ctx[K] : F;\n");
    }
    ctx.push_str("    const __ctx = undefined as unknown as __Ctx;\n");

    // Core Vue globals (always present on ComponentPublicInstance)
    ctx.push_str("    const $attrs = __ctx.$attrs;\n");
    ctx.push_str("    const $slots = __ctx.$slots;\n");
    ctx.push_str("    const $refs = __ctx.$refs;\n");
    ctx.push_str("    const $emit = __ctx.$emit;\n");

    // Vue 2-only instance members (absent from Vue 3's ComponentPublicInstance).
    if vue2_dialect {
        ctx.push_str("    // Vue 2 instance members (not on Vue 3 ComponentPublicInstance)\n");
        for member in VUE2_INSTANCE_MEMBERS {
            append!(ctx, "    const {member} = undefined as any;\n");
        }
    }

    // Plugin globals (resolved via ComponentCustomProperties if augmented,
    // otherwise falls back to the configured type_annotation)
    if !options.template_globals.is_empty() {
        ctx.push_str("    // Plugin globals (via ComponentCustomProperties)\n");
        for global in &options.template_globals {
            append!(
                ctx,
                "    const {}: __Global<'{}', {}> = undefined as any;\n",
                global.name,
                global.name,
                global.type_annotation
            );
        }
    }

    // CSS module globals (resolved via ComponentCustomProperties if augmented,
    // otherwise falls back to Record<string, string>)
    if !options.css_modules.is_empty() {
        ctx.push_str("    // CSS modules (from <style module>)\n");
        for module_name in &options.css_modules {
            append!(
                ctx,
                "    const {module_name}: __Global<'{module_name}', Record<string, string>> = undefined as any;\n"
            );
        }
    }

    // Mark all as used
    ctx.push_str("    void __ctx; void $attrs; void $slots; void $refs; void $emit;\n");
    if vue2_dialect {
        ctx.push_str("    ");
        for member in VUE2_INSTANCE_MEMBERS {
            append!(ctx, "void {member};");
        }
        ctx.push('\n');
    }
    if !options.template_globals.is_empty() {
        ctx.push_str("    ");
        for (i, global) in options.template_globals.iter().enumerate() {
            if i > 0 {
                ctx.push(' ');
            }
            append!(ctx, "void {};", global.name);
        }
        ctx.push('\n');
    }
    if !options.css_modules.is_empty() {
        ctx.push_str("    ");
        for (i, module_name) in options.css_modules.iter().enumerate() {
            if i > 0 {
                ctx.push(' ');
            }
            append!(ctx, "void {module_name};");
        }
        ctx.push('\n');
    }

    ctx
}

/// Get the generated subrange that corresponds to a specific source expression.
///
/// This keeps source maps anchored to the actual expression text instead of
/// any wrapping code we emit around it (`void (...)`, `as Foo`, handler shims).
pub(crate) fn generated_text_range(
    generated_segment: &str,
    mapped_text: &str,
    generated_start: usize,
) -> Range<usize> {
    if mapped_text.is_empty() {
        return generated_start..generated_start + generated_segment.len();
    }

    let relative_start = generated_segment.find(mapped_text).unwrap_or(0);
    let start = generated_start + relative_start;
    start..start + mapped_text.len()
}

/// Get the TypeScript event type for a DOM event name.
/// Returns the specific event interface (MouseEvent, KeyboardEvent, etc.)
pub(crate) fn get_dom_event_type(event_name: &str) -> &'static str {
    match event_name {
        // Mouse events
        "dblclick" | "mousedown" | "mouseup" | "mousemove" | "mouseenter" | "mouseleave"
        | "mouseover" | "mouseout" | "contextmenu" => "MouseEvent",

        // Pointer events
        // `click`/`auxclick` are PointerEvent in current TypeScript DOM maps.
        "click" | "auxclick" | "pointerdown" | "pointerup" | "pointermove" | "pointerenter"
        | "pointerleave" | "pointerover" | "pointerout" | "pointercancel" | "gotpointercapture"
        | "lostpointercapture" => "PointerEvent",

        // Touch events
        "touchstart" | "touchend" | "touchmove" | "touchcancel" => "TouchEvent",

        // Keyboard events
        "keydown" | "keyup" | "keypress" => "KeyboardEvent",

        // Focus events
        "focus" | "blur" | "focusin" | "focusout" => "FocusEvent",

        // Input events
        "input" | "beforeinput" => "InputEvent",

        // Composition events
        "compositionstart" | "compositionend" | "compositionupdate" => "CompositionEvent",

        // Form events
        "submit" => "SubmitEvent",
        "change" => "Event",
        "reset" => "Event",

        // Drag events
        "drag" | "dragstart" | "dragend" | "dragenter" | "dragleave" | "dragover" | "drop" => {
            "DragEvent"
        }

        // Clipboard events
        "cut" | "copy" | "paste" => "ClipboardEvent",

        // Wheel events
        "wheel" => "WheelEvent",

        // Animation events
        "animationstart" | "animationend" | "animationiteration" | "animationcancel" => {
            "AnimationEvent"
        }

        // Transition events
        "transitionstart" | "transitionend" | "transitionrun" | "transitioncancel" => {
            "TransitionEvent"
        }

        // UI events
        "scroll" | "resize" => "Event",

        // Media events
        "play" | "pause" | "ended" | "loadeddata" | "loadedmetadata" | "timeupdate"
        | "volumechange" | "waiting" | "seeking" | "seeked" | "ratechange" | "durationchange"
        | "canplay" | "canplaythrough" | "playing" | "progress" | "stalled" | "suspend"
        | "emptied" | "abort" => "Event",

        // Error/Load events
        "error" => "ErrorEvent",
        "load" => "Event",

        // Selection events
        "select" | "selectionchange" | "selectstart" => "Event",

        // Modern UI events that were absent — without an entry, `@toggle` /
        // `@beforetoggle` etc. fell back to the bare `Event` interface and
        // the user lost the specific payload members. See #688.
        "toggle" | "beforetoggle" => "ToggleEvent",
        "formdata" => "FormDataEvent",
        "popstate" => "PopStateEvent",
        "hashchange" => "HashChangeEvent",
        "message" => "MessageEvent",
        "storage" => "StorageEvent",
        "online" | "offline" => "Event",
        "securitypolicyviolation" => "SecurityPolicyViolationEvent",

        // Default fallback
        _ => "Event",
    }
}

#[cfg(test)]
mod event_type_tests {
    use super::get_dom_event_type;

    #[test]
    fn maps_legacy_dom_events() {
        assert_eq!(get_dom_event_type("click"), "PointerEvent");
        assert_eq!(get_dom_event_type("auxclick"), "PointerEvent");
        assert_eq!(get_dom_event_type("dblclick"), "MouseEvent");
        assert_eq!(get_dom_event_type("keydown"), "KeyboardEvent");
        assert_eq!(get_dom_event_type("submit"), "SubmitEvent");
    }

    #[test]
    fn maps_modern_dom_events() {
        // These fell back to `Event` before #688 — now they get the specific
        // interface so `e.newState` / `e.formData` etc. complete.
        assert_eq!(get_dom_event_type("toggle"), "ToggleEvent");
        assert_eq!(get_dom_event_type("beforetoggle"), "ToggleEvent");
        assert_eq!(get_dom_event_type("formdata"), "FormDataEvent");
    }

    #[test]
    fn unknown_events_fall_back_to_event() {
        assert_eq!(get_dom_event_type("totally-made-up"), "Event");
    }
}

/// Convert kebab-case or PascalCase prop name to camelCase.
/// Vue normalizes prop names to camelCase internally.
/// Examples: "my-prop" -> "myProp", "MyProp" -> "myProp"
pub(crate) fn to_camel_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = false;
    let mut first = true;

    for c in s.chars() {
        if c == '-' || c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else if first {
            // First character should be lowercase
            result.push(c.to_ascii_lowercase());
            first = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Sanitize a string to be a valid TypeScript identifier.
/// Replaces invalid characters (like ':') with underscores and prefixes
/// reserved words.
/// Examples: "update:title" -> "update_title", "my-event" -> "my_event"
pub(crate) fn to_safe_identifier(s: &str) -> String {
    let mut result = to_safe_identifier_fragment(s);

    if !result
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '$')
    {
        result.insert(0, '_');
    }
    if is_reserved_identifier(result.as_str()) {
        result.insert(0, '_');
    }

    result
}

/// Sanitize a string for use inside a generated identifier that already has a
/// safe prefix (for example `_slot_{name}`).
pub(crate) fn to_safe_identifier_fragment(s: &str) -> String {
    let mut result = String::with_capacity(s.len().max(1));

    for c in s.chars() {
        if c.is_ascii_alphanumeric() || c == '_' || c == '$' {
            result.push(c);
        } else {
            result.push('_');
        }
    }

    if result.is_empty() {
        result.push('_');
    }

    result
}

#[inline]
pub(crate) fn is_reserved_identifier(s: &str) -> bool {
    matches!(
        s,
        "await"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "new"
            | "null"
            | "return"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
            | "let"
            | "static"
            | "implements"
            | "interface"
            | "package"
            | "private"
            | "protected"
            | "public"
            | "as"
            | "from"
            | "of"
    )
}
