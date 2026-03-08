//! Helper functions and constants for virtual TypeScript generation.
//!
//! Contains utility functions for type declarations, event type mapping,
//! identifier conversion, and template context generation.

use super::types::VirtualTsOptions;
use vize_carton::append;
use vize_carton::String;

/// Vue compiler macros - these are defined inside setup scope, NOT globally.
/// This ensures they're only valid within <script setup>.
/// Parameters and type parameters are prefixed with _ to avoid "unused" warnings.
pub(crate) const VUE_SETUP_COMPILER_MACROS: &str = r#"  // Compiler macros (only valid in setup scope, not global)
  // Emit type helper: converts { event: [args] } to callable emit function
  type __EmitFn<T> = T extends (...args: any[]) => any ? T : (<K extends keyof T>(event: K, ...args: T[K] extends any[] ? T[K] : any[]) => void);
  // Vue ref type aliases (resolved from node_modules/vue)
  type __Ref<T> = import('vue').Ref<T>;
  type __ShallowRef<T> = import('vue').ShallowRef<T>;
  function defineProps<_T = unknown>(_props?: any): _T { void _props; return undefined as unknown as _T; }
  function defineEmits<_T = unknown>(): __EmitFn<_T> { return (() => {}) as any; }
  function defineEmits<_T extends readonly string[]>(_events: _T): (event: _T[number], ...args: any[]) => void { void _events; return (() => {}) as any; }
  function defineEmits<_T extends Record<string, any>>(_events: _T): (event: keyof _T, ...args: any[]) => void { void _events; return (() => {}) as any; }
  function defineExpose<_T = unknown>(_exposed?: _T): void { void _exposed; }
  function defineModel<_T = unknown>(): __Ref<_T | undefined> { return undefined as unknown as __Ref<_T | undefined>; }
  function defineModel<_T = unknown>(_options: any): __Ref<_T> { void _options; return undefined as unknown as __Ref<_T>; }
  function defineModel<_T = unknown>(_name: string, _options?: any): __Ref<_T> { void _name; void _options; return undefined as unknown as __Ref<_T>; }
  function defineSlots<_T = unknown>(): _T { return undefined as unknown as _T; }
  function withDefaults<_T = unknown, _D = unknown>(_props: _T, _defaults: _D): _T & _D { void _props; void _defaults; return undefined as unknown as _T & _D; }
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
    readonly client: boolean;
    readonly server: boolean;
    readonly dev: boolean;
    readonly prod: boolean;
    readonly ssr: boolean;
  }
}
"#;

/// Generate Vue template context declarations dynamically.
///
/// Derives `$`-prefixed globals from `ComponentPublicInstance` so that
/// type resolution is delegated to tsgo via Vue's type system
/// (including `ComponentCustomProperties` augmentations from plugins).
pub(crate) fn generate_template_context(options: &VirtualTsOptions) -> String {
    let mut ctx = String::default();

    let needs_global_helper =
        !options.template_globals.is_empty() || !options.css_modules.is_empty();

    // Instance type + conditional accessor helper
    ctx.push_str("    // Vue template context (delegates to ComponentPublicInstance)\n");
    ctx.push_str("    type __Ctx = $Vue['ComponentPublicInstance'];\n");
    if needs_global_helper {
        ctx.push_str("    type __Global<K extends string, F = unknown> = K extends keyof __Ctx ? __Ctx[K] : F;\n");
    }
    ctx.push_str("    const __ctx = undefined as unknown as __Ctx;\n");

    // Core Vue globals (always present on ComponentPublicInstance)
    ctx.push_str("    const $attrs = __ctx.$attrs;\n");
    ctx.push_str("    const $slots = __ctx.$slots;\n");
    ctx.push_str("    const $refs = __ctx.$refs;\n");
    ctx.push_str("    const $emit = __ctx.$emit;\n");

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

/// Strip TypeScript `as Type` assertion from a v-for source expression.
/// Returns (source_expression, Option<type_annotation>).
/// e.g., "(expr) as OptionSponsor[]" -> ("(expr)", Some("OptionSponsor[]"))
pub(crate) fn strip_as_assertion(source: &str) -> (&str, Option<&str>) {
    // Look for ` as ` in the source, but be careful with nested expressions.
    // We scan from the end to find the last top-level ` as `.
    let trimmed = source.trim();

    // Simple approach: find the last ` as ` that is not inside parentheses
    let mut paren_depth = 0i32;
    let bytes = trimmed.as_bytes();
    let mut last_as_pos = None;

    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => paren_depth += 1,
            b')' => paren_depth -= 1,
            b' ' if paren_depth == 0 => {
                // Check for " as "
                if i + 4 <= bytes.len() && &bytes[i..i + 4] == b" as " {
                    last_as_pos = Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }

    if let Some(pos) = last_as_pos {
        let expr = trimmed[..pos].trim();
        let type_ann = trimmed[pos + 4..].trim();
        if !type_ann.is_empty() {
            return (expr, Some(type_ann));
        }
    }

    (trimmed, None)
}

/// Get the TypeScript event type for a DOM event name.
/// Returns the specific event interface (MouseEvent, KeyboardEvent, etc.)
pub(crate) fn get_dom_event_type(event_name: &str) -> &'static str {
    match event_name {
        // Mouse events
        "click" | "dblclick" | "mousedown" | "mouseup" | "mousemove" | "mouseenter"
        | "mouseleave" | "mouseover" | "mouseout" | "contextmenu" => "MouseEvent",

        // Pointer events
        "pointerdown" | "pointerup" | "pointermove" | "pointerenter" | "pointerleave"
        | "pointerover" | "pointerout" | "pointercancel" | "gotpointercapture"
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

        // Default fallback
        _ => "Event",
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
/// Replaces invalid characters (like ':') with underscores.
/// Examples: "update:title" -> "update_title", "my-event" -> "my_event"
pub(crate) fn to_safe_identifier(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
