//! Code generation context and result types.

use crate::ast::{Namespace, Position, RuntimeHelper};
use crate::options::CodegenOptions;
use crate::runtime_helpers::RuntimeHelpers;

use super::helpers::default_helper_alias;
use super::source_map::SourceMapBuilder;
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::ToCompactString;
use vize_carton::camelize;
use vize_carton::capitalize;

/// Code generation context using a UTF-8 string buffer for performance.
pub struct CodegenContext {
    /// Generated code buffer
    pub(super) code: String,
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
    pub(super) used_helpers: RuntimeHelpers,
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
    /// When true, the props currently being generated belong to a plain
    /// (native) element rather than a component/slot/template. Affects v-on
    /// event-name casing rules (Vue preserves case via `on:` for plain
    /// elements that have uppercase letters in the raw event name).
    pub(super) props_is_plain_element: bool,
    /// Namespace of the native parent currently generating children. Vue only
    /// needs block creation at SVG/MathML namespace boundaries, not for every
    /// descendant inside the same namespace.
    pub(super) parent_ns: Namespace,
    /// Whether static child VNodes should be cached in the render function.
    pub(super) static_cache: bool,
    /// True while emitting the children of an already-cached static VNode.
    /// Vue caches the top-most static element of a subtree as one entry and
    /// renders its descendants as plain `createElementVNode(...)` calls inside
    /// the cached array — no nested `_cache[...]` wrappers and no per-descendant
    /// `CACHED` patch flag. This flag suppresses re-caching inside that subtree.
    pub(super) in_cached_static: bool,
    /// Template-wide counter for v-if branch keys. Each branch in any
    /// conditional chain in the template consumes one value, so sibling
    /// `v-if`/`v-else` blocks do not reuse the same `{ key: n }` and a
    /// patch-time element from one branch can't be reused for another
    /// (#961). Vue uses the same shared counter.
    pub(super) v_if_branch_counter: usize,
    /// Source-map segment accumulator. `Some` only when the `source_map`
    /// codegen flag is enabled; the no-map path never allocates it and
    /// `record_mapping` is a no-op, keeping the generated `code` byte-identical
    /// either way.
    pub(super) map_builder: Option<SourceMapBuilder>,
}

/// Byte offsets of the structural sections of a generated render module,
/// recorded at emission time.
///
/// SFC inline assembly needs the generated module split back into imports /
/// hoisted consts / asset preamble / render body. Recording the boundaries
/// while the code is written lets the caller slice the buffer directly
/// instead of re-scanning the output line by line.
#[derive(Debug, Clone, Copy)]
pub struct CodegenSections {
    /// Byte length of the import statement section at the start of
    /// `preamble`. Hoisted declarations (when present) follow after a single
    /// `'\n'` separator.
    pub imports_len: usize,
    /// Byte range in `code` covering the component/directive resolution
    /// statements inside the render function (raw, including indentation).
    pub assets_start: usize,
    pub assets_end: usize,
    /// Byte range in `code` covering the root `return` expression (the bytes
    /// after `"return "` up to the closing brace line).
    pub return_expr_start: usize,
    pub return_expr_end: usize,
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

/// Code generation result with emission-recorded section boundaries.
///
/// This is a separate wrapper so the longstanding public [`CodegenResult`]
/// remains constructible with the same public fields.
pub struct CodegenResultWithSections {
    /// Generated code, preamble, and source map.
    pub result: CodegenResult,
    /// Section boundaries recorded during emission (`None` when codegen
    /// bailed out before producing a render function).
    pub sections: Option<CodegenSections>,
}

impl CodegenResultWithSections {
    /// Drop section metadata and keep the public codegen result.
    pub fn into_result(self) -> CodegenResult {
        self.result
    }
}

impl CodegenContext {
    /// Create a new codegen context
    pub fn new(options: CodegenOptions) -> Self {
        let map_builder = options.source_map.then(SourceMapBuilder::new);
        Self {
            code: String::with_capacity(4096),
            indent_level: 0,
            ssr: options.ssr,
            helper_alias: default_helper_alias,
            runtime_global_name: options.runtime_global_name.to_compact_string(),
            runtime_module_name: options.runtime_module_name.to_compact_string(),
            options,
            pure: false,
            used_helpers: RuntimeHelpers::default(),
            cache_index: 0,
            slot_params: FxHashSet::default(),
            skip_is_prop: false,
            skip_scope_id: false,
            skip_normalize: false,
            in_v_for: false,
            skip_v_memo: false,
            props_is_plain_element: false,
            parent_ns: Namespace::Html,
            static_cache: false,
            in_cached_static: false,
            v_if_branch_counter: 0,
            map_builder,
        }
    }

    /// Record a source-map segment for the token about to be written.
    ///
    /// Captures the current generated byte offset (the buffer length, i.e. where
    /// the next `push` lands) and pairs it with the source byte `offset` of the
    /// originating AST node. Byte offsets are used (rather than the AST's
    /// `line`/`column`) because the parser does not track line breaks for node
    /// positions; the offset is exact and the line/column are computed from it
    /// at map-assembly time. No-op unless the `source_map` flag is on, so call
    /// sites can invoke it unconditionally right before pushing the mapped text
    /// without affecting the generated `code`.
    #[inline]
    pub(super) fn record_mapping(&mut self, loc: &Position) {
        if let Some(builder) = self.map_builder.as_mut() {
            let generated_offset = self.code.len();
            builder.add_raw(generated_offset, loc.offset);
        }
    }

    /// Take the source-map builder out of the context, if any.
    ///
    /// Consumed at the end of the pipeline to serialize the map once the full
    /// code buffer is known.
    pub(super) fn take_map_builder(&mut self) -> Option<SourceMapBuilder> {
        self.map_builder.take()
    }

    /// Run codegen while treating `ns` as the current native parent namespace.
    pub(super) fn with_parent_namespace<T>(
        &mut self,
        ns: Namespace,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let prev = self.parent_ns;
        self.parent_ns = ns;
        let result = f(self);
        self.parent_ns = prev;
        result
    }

    /// Allocate the next v-if branch key for the current template.
    ///
    /// Branch keys are unique across the whole template, matching Vue's
    /// shared counter, so two sibling conditional blocks never collide on
    /// `{ key: n }`.
    pub(super) fn next_v_if_branch_key(&mut self) -> usize {
        let key = self.v_if_branch_counter;
        self.v_if_branch_counter = self.v_if_branch_counter.saturating_add(1);
        key
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

    /// Push string to buffer
    #[inline]
    pub fn push(&mut self, code: &str) {
        self.code.push_str(code);
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
        self.code.push('\n');
        for _ in 0..self.indent_level {
            self.code.push_str("  ");
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
            self.code.push_str("/*#__PURE__*/ ");
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
        self.used_helpers.add(helper);
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
        self.code.push_str(code);
    }

    /// Push formatted line (format_args! + newline with indentation)
    #[inline]
    #[allow(dead_code)]
    pub fn push_line_fmt(&mut self, args: std::fmt::Arguments<'_>) {
        use std::fmt::Write as _;
        let _ = self.write_fmt(args);
        self.newline();
    }

    /// Get the generated code as a String
    pub fn into_code(self) -> String {
        self.code
    }

    /// Get the generated code as a reference (for temporary use)
    pub fn code_as_str(&self) -> &str {
        &self.code
    }
}

impl std::fmt::Write for CodegenContext {
    #[inline]
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.code.push_str(s);
        Ok(())
    }
}
