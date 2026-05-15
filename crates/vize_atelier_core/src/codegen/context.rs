//! Code generation context and result types.

use crate::ast::RuntimeHelper;
use crate::options::CodegenOptions;

use super::helpers::default_helper_alias;
use vize_carton::camelize;
use vize_carton::capitalize;
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::ToCompactString;

/// Code generation context using byte buffer for performance
pub struct CodegenContext {
    /// Generated code buffer (bytes)
    pub(super) code: Vec<u8>,
    /// Current indentation level
    pub(super) indent_level: u32,
    /// Whether we're in SSR mode
    #[allow(dead_code)]
    pub(super) ssr: bool,
    /// Helper function alias map
    pub(super) helper_alias: fn(RuntimeHelper) -> &'static str,
    /// Runtime global name
    pub(super) runtime_global_name: String,
    /// Runtime module name
    pub(super) runtime_module_name: String,
    /// Options
    pub(super) options: CodegenOptions,
    /// Pure annotation for tree-shaking
    pub(super) pure: bool,
    /// Helpers used during codegen
    pub(super) used_helpers: FxHashSet<RuntimeHelper>,
    /// Cache index for v-once
    pub(super) cache_index: usize,
    /// Template-scope parameters (slot props and v-for aliases) that should
    /// not be prefixed with `_ctx.`
    pub(super) slot_params: FxHashSet<String>,
    /// When true, skip `is` prop in generate_props (used for dynamic components)
    pub(super) skip_is_prop: bool,
    /// When true, skip scope_id attribute in props (used for component/slot elements)
    pub(super) skip_scope_id: bool,
    /// When true, skip normalizeClass/normalizeStyle wrappers (inside mergeProps)
    pub(super) skip_normalize: bool,
    /// When true, we are inside a v-for loop (affects slot stability flags)
    pub(super) in_v_for: bool,
    /// When true, skip v-memo wrapping (already handled by v-for + v-memo)
    pub(super) skip_v_memo: bool,
}

/// Code generation result
pub struct CodegenResult {
    /// Generated code
    pub code: String,
    /// Preamble (imports)
    pub preamble: String,
    /// Source map (JSON)
    pub map: Option<String>,
}

impl CodegenContext {
    /// Create a new codegen context
    pub fn new(options: CodegenOptions) -> Self {
        Self {
            code: Vec::with_capacity(4096),
            indent_level: 0,
            ssr: options.ssr,
            helper_alias: default_helper_alias,
            runtime_global_name: options.runtime_global_name.to_compact_string(),
            runtime_module_name: options.runtime_module_name.to_compact_string(),
            options,
            pure: false,
            used_helpers: FxHashSet::default(),
            cache_index: 0,
            slot_params: FxHashSet::default(),
            skip_is_prop: false,
            skip_scope_id: false,
            skip_normalize: false,
            in_v_for: false,
            skip_v_memo: false,
        }
    }

    /// Add template-scope parameters (identifiers that should not be prefixed)
    pub fn add_slot_params(&mut self, params: &[String]) {
        for param in params {
            self.slot_params.insert(param.clone());
        }
    }

    /// Remove template-scope parameters when exiting their scope
    pub fn remove_slot_params(&mut self, params: &[String]) {
        for param in params {
            self.slot_params.remove(param);
        }
    }

    /// Check if an identifier is a template-scope parameter
    pub fn is_slot_param(&self, name: &str) -> bool {
        self.slot_params.contains(name)
    }

    /// Check if there are any template-scope parameters registered
    #[inline]
    pub fn has_slot_params(&self) -> bool {
        !self.slot_params.is_empty()
    }

    /// Event handler caching is unsafe while template-scope params are in play,
    /// because a cached closure would capture the first scoped value.
    #[inline]
    pub fn cache_handlers_in_current_scope(&self) -> bool {
        self.options.cache_handlers && !self.has_slot_params()
    }

    /// Get next cache index for v-once
    pub fn next_cache_index(&mut self) -> usize {
        let index = self.cache_index;
        self.cache_index += 1;
        index
    }

    /// Push bytes to buffer
    #[inline]
    pub fn push_bytes(&mut self, bytes: &[u8]) {
        self.code.extend_from_slice(bytes);
    }

    /// Push string to buffer
    #[inline]
    pub fn push(&mut self, code: &str) {
        self.code.extend_from_slice(code.as_bytes());
    }

    /// Push code with newline
    #[inline]
    pub fn push_line(&mut self, code: &str) {
        self.push(code);
        self.newline();
    }

    /// Add newline with proper indentation
    #[inline]
    pub fn newline(&mut self) {
        self.code.push(b'\n');
        for _ in 0..self.indent_level {
            self.code.extend_from_slice(b"  ");
        }
    }

    /// Increase indentation
    #[inline]
    pub fn indent(&mut self) {
        self.indent_level += 1;
    }

    /// Decrease indentation
    #[inline]
    pub fn deindent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    /// Add pure annotation /*#__PURE__*/
    #[inline]
    pub fn push_pure(&mut self) {
        if self.pure {
            self.code.extend_from_slice(b"/*#__PURE__*/ ");
        }
    }

    /// Get helper name
    #[inline]
    pub fn helper(&self, helper: RuntimeHelper) -> &'static str {
        (self.helper_alias)(helper)
    }

    /// Track a helper for preamble generation
    #[inline]
    pub fn use_helper(&mut self, helper: RuntimeHelper) {
        self.used_helpers.insert(helper);
    }

    /// Check if a component is in binding metadata (from script setup)
    pub fn is_component_in_bindings(&self, component: &str) -> bool {
        self.resolve_component_binding_name(component).is_some()
    }

    /// Resolve the binding name for a component tag.
    pub fn resolve_component_binding_name(&self, component: &str) -> Option<String> {
        let metadata = self.options.binding_metadata.as_ref()?;

        let resolve_base = |name: &str| {
            if metadata.bindings.contains_key(name) {
                return Some(name.to_compact_string());
            }

            let camel = camelize(name);
            if metadata.bindings.contains_key(camel.as_str()) {
                return Some(camel);
            }

            let pascal = capitalize(&camel);
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

    /// Push string to buffer (alias for `push`, compatible with `appends!`/`append!` macros)
    #[inline]
    #[allow(dead_code)]
    pub fn push_str(&mut self, code: &str) {
        self.code.extend_from_slice(code.as_bytes());
    }

    /// Push formatted line (format_args! + newline with indentation)
    #[inline]
    #[allow(dead_code)]
    pub fn push_line_fmt(&mut self, args: std::fmt::Arguments<'_>) {
        use std::fmt::Write as _;
        self.write_fmt(args).unwrap();
        self.newline();
    }

    /// Get the generated code as a String
    pub fn into_code(self) -> String {
        // SAFETY: We only push valid UTF-8 strings
        unsafe { String::from_utf8_unchecked(self.code) }
    }

    /// Get the generated code as a reference (for temporary use)
    pub fn code_as_str(&self) -> &str {
        // SAFETY: We only push valid UTF-8 strings
        unsafe { std::str::from_utf8_unchecked(&self.code) }
    }
}

impl std::fmt::Write for CodegenContext {
    #[inline]
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.code.extend_from_slice(s.as_bytes());
        Ok(())
    }
}
