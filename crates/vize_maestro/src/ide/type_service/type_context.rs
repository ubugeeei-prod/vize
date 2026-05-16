//! Type context construction.
//!
//! Builds a `vize_canon::TypeContext` from SFC descriptors by extracting
//! bindings from script blocks and adding Vue built-in globals.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use super::TypeService;

impl TypeService {
    /// Build type context from SFC descriptor.
    pub(super) fn build_type_context(
        descriptor: &vize_atelier_sfc::SfcDescriptor,
    ) -> vize_canon::TypeContext {
        let mut ctx = vize_canon::TypeContext::new();

        // Extract bindings from script setup
        if let Some(ref script_setup) = descriptor.script_setup {
            Self::extract_bindings_from_script(&script_setup.content, &mut ctx);
        }

        // Extract bindings from regular script (for Options API)
        if let Some(ref script) = descriptor.script {
            Self::extract_bindings_from_script(&script.content, &mut ctx);
        }

        // Add Vue built-in globals
        Self::add_vue_globals(&mut ctx);

        ctx
    }

    /// Extract bindings from script content.
    fn extract_bindings_from_script(script: &str, ctx: &mut vize_canon::TypeContext) {
        // Use simple pattern matching to extract bindings
        // This is a simplified version - full implementation would use a proper parser

        // Find const/let/var declarations
        for pattern in ["const ", "let ", "var "] {
            let mut pos = 0;
            while let Some(start) = script[pos..].find(pattern) {
                let abs_start = pos + start + pattern.len();
                let remaining = &script[abs_start..];

                // Extract the identifier
                if let Some(ident) = Self::extract_identifier(remaining) {
                    let kind = match pattern.trim() {
                        "const" => vize_canon::BindingKind::Const,
                        "let" => vize_canon::BindingKind::Let,
                        "var" => vize_canon::BindingKind::Var,
                        _ => vize_canon::BindingKind::Const,
                    };

                    // Try to infer type
                    let type_info = Self::infer_binding_type(remaining, &ident);

                    ctx.add_binding(
                        ident.clone(),
                        vize_canon::Binding::new(ident, type_info, kind),
                    );
                }

                pos = abs_start + 1;
            }
        }

        // Find function declarations
        let mut pos = 0;
        while let Some(start) = script[pos..].find("function ") {
            let abs_start = pos + start + 9;
            let remaining = &script[abs_start..];

            if let Some(ident) = Self::extract_identifier(remaining) {
                ctx.add_binding(
                    ident.clone(),
                    vize_canon::Binding::new(
                        ident,
                        vize_canon::TypeInfo::new(
                            "(...args: any[]) => any",
                            vize_canon::TypeKind::Function,
                        ),
                        vize_canon::BindingKind::Function,
                    ),
                );
            }

            pos = abs_start + 1;
        }

        // Find ref(), computed(), reactive() calls
        for (fn_name, kind) in [
            ("ref(", vize_canon::BindingKind::Ref),
            ("computed(", vize_canon::BindingKind::Computed),
            ("reactive(", vize_canon::BindingKind::Reactive),
        ] {
            let mut search_pos = 0;
            while let Some(fn_pos) = script[search_pos..].find(fn_name) {
                let abs_fn_pos = search_pos + fn_pos;

                // Look backwards for the binding name
                if let Some(binding_name) = Self::find_binding_before(script, abs_fn_pos) {
                    let type_info = match kind {
                        vize_canon::BindingKind::Ref => {
                            vize_canon::TypeInfo::new("Ref<unknown>", vize_canon::TypeKind::Ref)
                        }
                        vize_canon::BindingKind::Computed => vize_canon::TypeInfo::new(
                            "ComputedRef<unknown>",
                            vize_canon::TypeKind::Computed,
                        ),
                        vize_canon::BindingKind::Reactive => vize_canon::TypeInfo::new(
                            "Reactive<unknown>",
                            vize_canon::TypeKind::Reactive,
                        ),
                        _ => vize_canon::TypeInfo::unknown(),
                    };

                    ctx.add_binding(
                        binding_name.clone(),
                        vize_canon::Binding::new(binding_name, type_info, kind),
                    );
                }

                search_pos = abs_fn_pos + fn_name.len();
            }
        }
    }

    /// Extract an identifier from the start of a string.
    pub(super) fn extract_identifier(s: &str) -> Option<String> {
        let s = s.trim_start();
        if s.is_empty() {
            return None;
        }

        let bytes = s.as_bytes();
        let first = bytes[0] as char;

        // Must start with letter, underscore, or $
        if !first.is_ascii_alphabetic() && first != '_' && first != '$' {
            return None;
        }

        let mut end = 1;
        while end < bytes.len() {
            let c = bytes[end] as char;
            if !c.is_ascii_alphanumeric() && c != '_' && c != '$' {
                break;
            }
            end += 1;
        }

        Some(s[..end].to_string())
    }

    /// Find the binding name before a function call like ref().
    fn find_binding_before(script: &str, fn_pos: usize) -> Option<String> {
        // Look for pattern like "const name = ref("
        let before = &script[..fn_pos];
        let trimmed = before.trim_end();

        // Should end with "= "
        if !trimmed.ends_with('=') {
            return None;
        }

        let before_eq = trimmed[..trimmed.len() - 1].trim_end();

        // Find the identifier before =
        let mut end = before_eq.len();
        let bytes = before_eq.as_bytes();

        while end > 0 {
            let c = bytes[end - 1] as char;
            if !c.is_ascii_alphanumeric() && c != '_' && c != '$' {
                break;
            }
            end -= 1;
        }

        if end < before_eq.len() {
            Some(before_eq[end..].to_string())
        } else {
            None
        }
    }

    /// Infer type from binding initialization.
    pub(super) fn infer_binding_type(after_ident: &str, _ident: &str) -> vize_canon::TypeInfo {
        let trimmed = after_ident.trim_start();

        // Check for type annotation
        if trimmed.starts_with(':') {
            // Has type annotation - extract it
            if let Some(eq_pos) = trimmed.find('=') {
                let type_str = trimmed[1..eq_pos].trim();
                return vize_canon::TypeInfo::new(type_str, vize_canon::TypeKind::Unknown);
            }
        }

        // Check for = and infer from value
        if let Some(stripped) = trimmed.strip_prefix('=') {
            let value = stripped.trim_start();

            // String literal
            if value.starts_with('"') || value.starts_with('\'') || value.starts_with('`') {
                return vize_canon::TypeInfo::string();
            }

            // Number
            if value
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                return vize_canon::TypeInfo::number();
            }

            // Boolean
            if value.starts_with("true") || value.starts_with("false") {
                return vize_canon::TypeInfo::boolean();
            }

            // ref()
            if value.starts_with("ref(") {
                return vize_canon::TypeInfo::new("Ref<unknown>", vize_canon::TypeKind::Ref);
            }

            // computed()
            if value.starts_with("computed(") {
                return vize_canon::TypeInfo::new(
                    "ComputedRef<unknown>",
                    vize_canon::TypeKind::Computed,
                );
            }

            // reactive()
            if value.starts_with("reactive(") {
                return vize_canon::TypeInfo::new(
                    "Reactive<unknown>",
                    vize_canon::TypeKind::Reactive,
                );
            }

            // Array literal
            if value.starts_with('[') {
                return vize_canon::TypeInfo::new("unknown[]", vize_canon::TypeKind::Array);
            }

            // Object literal
            if value.starts_with('{') {
                return vize_canon::TypeInfo::new("object", vize_canon::TypeKind::Object);
            }
        }

        vize_canon::TypeInfo::unknown()
    }

    /// Add Vue built-in globals to context.
    fn add_vue_globals(ctx: &mut vize_canon::TypeContext) {
        // Template globals
        ctx.globals.insert(
            vize_carton::cstr!("$slots"),
            vize_canon::TypeInfo::new("Slots", vize_canon::TypeKind::Object),
        );
        ctx.globals.insert(
            vize_carton::cstr!("$emit"),
            vize_canon::TypeInfo::new(
                "(event: string, ...args: any[]) => void",
                vize_canon::TypeKind::Function,
            ),
        );
        ctx.globals.insert(
            vize_carton::cstr!("$attrs"),
            vize_canon::TypeInfo::new("Record<string, unknown>", vize_canon::TypeKind::Object),
        );
        ctx.globals.insert(
            vize_carton::cstr!("$refs"),
            vize_canon::TypeInfo::new(
                "Record<string, Element | ComponentPublicInstance | null>",
                vize_canon::TypeKind::Object,
            ),
        );
        ctx.globals.insert(
            vize_carton::cstr!("$el"),
            vize_canon::TypeInfo::new("Element | null", vize_canon::TypeKind::Object),
        );
        ctx.globals.insert(
            vize_carton::cstr!("$parent"),
            vize_canon::TypeInfo::new(
                "ComponentPublicInstance | null",
                vize_canon::TypeKind::Object,
            ),
        );
        ctx.globals.insert(
            vize_carton::cstr!("$root"),
            vize_canon::TypeInfo::new("ComponentPublicInstance", vize_canon::TypeKind::Object),
        );
        ctx.globals.insert(
            vize_carton::cstr!("$data"),
            vize_canon::TypeInfo::new("object", vize_canon::TypeKind::Object),
        );
        ctx.globals.insert(
            vize_carton::cstr!("$options"),
            vize_canon::TypeInfo::new("ComponentOptions", vize_canon::TypeKind::Object),
        );
        ctx.globals.insert(
            vize_carton::cstr!("$props"),
            vize_canon::TypeInfo::new("object", vize_canon::TypeKind::Object),
        );
        ctx.globals.insert(
            vize_carton::cstr!("$watch"),
            vize_canon::TypeInfo::new("WatchStopHandle", vize_canon::TypeKind::Function),
        );
        ctx.globals.insert(
            vize_carton::cstr!("$forceUpdate"),
            vize_canon::TypeInfo::new("() => void", vize_canon::TypeKind::Function),
        );
        ctx.globals.insert(
            vize_carton::cstr!("$nextTick"),
            vize_canon::TypeInfo::new(
                "(callback?: () => void) => Promise<void>",
                vize_canon::TypeKind::Function,
            ),
        );
    }
}
