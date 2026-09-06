//! Emit-time options for the S2 DOM lane (P2-11).
//!
//! The shipped DOM codegen reads its emission-only settings from
//! `CodegenOptions`. The S2 emitter mirrors the subset it honours here so
//! the atelier_dom dual-run can pin each option against the shipped lane
//! one at a time. A field missing from this struct is not a default the
//! emitter silently assumes — it is production surface the series has

use alloc::vec::Vec as StdVec;
use vize_s0::String;

/// Which module form the render function is emitted in — the shipped
/// lane's `CodegenMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DomEmitMode {
    /// `const { … } = Vue` destructure plus the full
    /// `function render(_ctx, _cache, $props, $setup, $data, $options)`
    /// signature (the shipped default).
    #[default]
    Function,
    /// `import { … } from "vue"` plus `export function render(_ctx, _cache)`;
    /// the six-argument signature returns with binding metadata.
    Module,
}

impl DomEmitMode {
    /// The render-function header the shipped lane writes for this mode.
    #[must_use]
    pub(super) const fn render_signature(self, with_bindings: bool) -> &'static str {
        match (self, with_bindings) {
            (Self::Function, _) => {
                "function render(_ctx, _cache, $props, $setup, $data, $options) {"
            }
            (Self::Module, true) => {
                "export function render(_ctx, _cache, $props, $setup, $data, $options) {"
            }
            (Self::Module, false) => "export function render(_ctx, _cache) {",
        }
    }
}

/// The shipped lane's `BindingType`: how a template identifier resolves
/// once script analysis has named it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    /// `let` in `<script setup>`.
    SetupLet,
    /// `const` in `<script setup>` that may hold a ref.
    SetupMaybeRef,
    /// `const` in `<script setup>` that is a ref.
    SetupRef,
    /// `reactive()` / `shallowReactive()` const.
    SetupReactiveConst,
    /// Non-reactive `const` (functions, classes, plain values).
    SetupConst,
    /// A declared prop.
    Props,
    /// A destructured prop with an alias.
    PropsAliased,
    /// `data()` member.
    Data,
    /// Options API member (computed, methods, inject).
    Options,
    /// A literal constant.
    LiteralConst,
    /// A universal JavaScript global.
    JsGlobalUniversal,
    /// A browser-only global.
    JsGlobalBrowser,
    /// A Node.js-only global.
    JsGlobalNode,
    /// A Deno-only global.
    JsGlobalDeno,
    /// A Bun-only global.
    JsGlobalBun,
    /// A Vue instance global (`$slots`, `$emit`, …).
    VueGlobal,
    /// Imported from another module.
    ExternalModule,
}

impl BindingKind {
    /// The member prefix the shipped lane writes in non-inline mode
    /// (`BindingType::non_inline_template_prefix`).
    #[must_use]
    pub const fn non_inline_template_prefix(self) -> &'static str {
        match self {
            Self::SetupLet
            | Self::SetupMaybeRef
            | Self::SetupRef
            | Self::SetupReactiveConst
            | Self::SetupConst
            | Self::LiteralConst
            | Self::JsGlobalUniversal
            | Self::JsGlobalBrowser
            | Self::JsGlobalNode
            | Self::JsGlobalDeno
            | Self::JsGlobalBun
            | Self::ExternalModule => "$setup.",
            Self::Props | Self::PropsAliased => "$props.",
            Self::Data => "$data.",
            Self::Options => "$options.",
            Self::VueGlobal => "_ctx.",
        }
    }

    /// Props and aliased props.
    #[must_use]
    pub const fn is_props(self) -> bool {
        matches!(self, Self::Props | Self::PropsAliased)
    }
}

/// The shipped lane's `BindingMetadata`: names the script analysis
/// resolved, with the destructured-prop aliases.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BindingTable {
    /// `(name, kind)` sorted by name for binary search.
    names: StdVec<(String, BindingKind)>,
    /// `(local, prop key)` for `const { key: local } = defineProps()`.
    aliases: StdVec<(String, String)>,
    /// Whether the bindings come from `<script setup>`.
    is_script_setup: bool,
}

impl BindingTable {
    /// Build a table; later duplicates of a name win, as a map insert would.
    #[must_use]
    pub fn new<'n>(
        names: impl IntoIterator<Item = (&'n str, BindingKind)>,
        aliases: impl IntoIterator<Item = (&'n str, &'n str)>,
        is_script_setup: bool,
    ) -> Self {
        let mut table = Self {
            names: StdVec::new(),
            aliases: StdVec::new(),
            is_script_setup,
        };
        for (name, kind) in names {
            match table
                .names
                .binary_search_by(|(entry, _)| entry.as_str().cmp(name))
            {
                Ok(index) => table.names[index].1 = kind,
                Err(index) => table.names.insert(index, (String::from(name), kind)),
            }
        }
        for (local, key) in aliases {
            match table
                .aliases
                .binary_search_by(|(entry, _)| entry.as_str().cmp(local))
            {
                Ok(index) => table.aliases[index].1 = String::from(key),
                Err(index) => table
                    .aliases
                    .insert(index, (String::from(local), String::from(key))),
            }
        }
        table
    }

    /// The kind recorded for `name`.
    #[must_use]
    pub fn kind(&self, name: &str) -> Option<BindingKind> {
        self.names
            .binary_search_by(|(entry, _)| entry.as_str().cmp(name))
            .ok()
            .map(|index| self.names[index].1)
    }

    /// Whether `name` is recorded at all.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.kind(name).is_some()
    }

    /// The destructured-prop aliases as `(local, key)` pairs.
    pub fn aliases(&self) -> impl Iterator<Item = (&str, &str)> {
        self.aliases
            .iter()
            .map(|(local, key)| (local.as_str(), key.as_str()))
    }

    /// Whether the bindings come from `<script setup>`.
    #[must_use]
    pub const fn is_script_setup(&self) -> bool {
        self.is_script_setup
    }
}

/// Emission settings the S2 DOM emitter honours.
#[derive(Debug, Clone, Copy)]
pub struct DomEmitOptions<'a> {
    /// Module or function output.
    pub mode: DomEmitMode,
    /// Module name imported in [`DomEmitMode::Module`] (`"vue"` by default).
    pub runtime_module_name: &'a str,
    /// Global read in [`DomEmitMode::Function`] (`"Vue"` by default).
    pub runtime_global_name: &'a str,
    /// Free identifiers become member accesses (`emit::prefix`) when enabled.
    pub prefix_identifiers: bool,
    /// Static props and static VNode declarations are emitted only when enabled.
    pub hoist_static: bool,
    /// Inline render functions read setup bindings from closure locals instead
    /// of the `$setup` proxy.
    pub inline: bool,
    /// The shipped lane's `component_name`: the SFC's own name (its file
    /// stem). A component tag that resolves to it is a self-reference, and
    /// the shipped lane asks the runtime to resolve it as one.
    pub component_name: Option<&'a str>,
    /// Cache inline `v-on` handlers in `_cache` when safe for scopes.
    pub cache_handlers: bool,
    /// SFC scoped-style attr for module-level static VNode hoists.
    pub hoisted_scope_id: Option<&'a str>,
    /// The shipped lane's `scope_id`: `<style scoped>` gives the SFC an
    /// attribute name (`data-v-abc123`) that every element's props object
    /// carries as a trailing `"data-v-abc123": ""` pair.
    pub scope_id: Option<&'a str>,
    /// The shipped lane's `is_ts`: template expressions are TypeScript,
    /// so each one is type-erased (`emit::prefix::typescript`) before the
    /// identifier pass reads it.
    pub is_ts: bool,
    /// Preserve ordinary template comments as `_createCommentVNode(...)`.
    pub comments: bool,
    /// Preserve Vue's experimental `//` comments inside opening tag attrs.
    pub experimental_in_tag_comments: bool,
    /// Declarative tag patterns that the shipped lane treats as custom
    /// elements instead of components.
    pub custom_element_patterns: &'a [String],
    /// Static predicate branch for host-defined custom-element classifiers.
    pub custom_element_predicate: Option<fn(&str) -> bool>,
    /// The shipped lane's `binding_metadata`, honoured in non-inline mode:
    /// prefixed identifiers resolve to the same proxy members and signatures.
    pub bindings: Option<&'a BindingTable>,
}

impl PartialEq for DomEmitOptions<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.mode == other.mode
            && self.runtime_module_name == other.runtime_module_name
            && self.runtime_global_name == other.runtime_global_name
            && self.prefix_identifiers == other.prefix_identifiers
            && self.hoist_static == other.hoist_static
            && self.inline == other.inline
            && self.component_name == other.component_name
            && self.cache_handlers == other.cache_handlers
            && self.hoisted_scope_id == other.hoisted_scope_id
            && self.scope_id == other.scope_id
            && self.is_ts == other.is_ts
            && self.comments == other.comments
            && self.experimental_in_tag_comments == other.experimental_in_tag_comments
            && self.custom_element_patterns == other.custom_element_patterns
            && custom_element_predicate_eq(
                self.custom_element_predicate,
                other.custom_element_predicate,
            )
            && self.bindings == other.bindings
    }
}

impl Eq for DomEmitOptions<'_> {}

fn custom_element_predicate_eq(
    left: Option<fn(&str) -> bool>,
    right: Option<fn(&str) -> bool>,
) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => core::ptr::fn_addr_eq(left, right),
        _ => false,
    }
}

impl DomEmitOptions<'static> {
    /// The shipped lane's `CodegenOptions::default()` projected onto the
    /// fields the emitter honours.
    pub const DEFAULT: Self = Self {
        mode: DomEmitMode::Function,
        runtime_module_name: "vue",
        runtime_global_name: "Vue",
        prefix_identifiers: false,
        hoist_static: true,
        inline: false,
        component_name: None,
        cache_handlers: false,
        hoisted_scope_id: None,
        scope_id: None,
        is_ts: false,
        comments: false,
        experimental_in_tag_comments: false,
        custom_element_patterns: &[],
        custom_element_predicate: None,
        bindings: None,
    };
}

impl Default for DomEmitOptions<'static> {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
mod tests;
