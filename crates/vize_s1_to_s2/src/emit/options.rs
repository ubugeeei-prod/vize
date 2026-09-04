//! Emit-time options for the S2 DOM lane (P2-11).
//!
//! The shipped DOM codegen reads its emission-only settings from
//! `CodegenOptions`. The S2 emitter mirrors the subset it honours here so
//! the atelier_dom dual-run can pin each option against the shipped lane
//! one at a time. A field missing from this struct is not a default the
//! emitter silently assumes — it is production surface the series has
//! not reached yet, and the witness for it does not exist.

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DomEmitOptions<'a> {
    /// Module or function output.
    pub mode: DomEmitMode,
    /// The module the helper imports name in [`DomEmitMode::Module`]
    /// (`"vue"` by default).
    pub runtime_module_name: &'a str,
    /// The global the helper destructure reads in
    /// [`DomEmitMode::Function`] (`"Vue"` by default).
    pub runtime_global_name: &'a str,
    /// The shipped lane's `prefix_identifiers`: free identifiers become
    /// member accesses (`emit::prefix`).
    pub prefix_identifiers: bool,
    /// The shipped lane's `inline`: the render function is inlined into
    /// `setup()`, so setup bindings are read straight from the closure —
    /// refs through `.value`, `let`/maybe-ref bindings through `_unref`,
    /// props through `__props.` — instead of the `$setup` proxy.
    pub inline: bool,
    /// The shipped lane's `component_name`: the SFC's own name (its file
    /// stem). A component tag that resolves to it is a self-reference, and
    /// the shipped lane asks the runtime to resolve it as one.
    pub component_name: Option<&'a str>,
    /// The shipped lane's `cache_handlers`: an inline `v-on` handler is
    /// hoisted into the render function's `_cache` array, so the closure
    /// is created once instead of on every render. Turned on with
    /// [`DomEmitOptions::inline`] by `compile_template_block`.
    pub cache_handlers: bool,
    /// SFC scoped-style attr for module-level static VNode hoists. The DOM
    /// SFC wrapper passes this separately from `scope_id` because runtime
    /// VNodes receive scope attrs from Vue; only import-time hoists need it
    /// baked into generated props.
    pub hoisted_scope_id: Option<&'a str>,
    /// The shipped lane's `scope_id`: `<style scoped>` gives the SFC an
    /// attribute name (`data-v-abc123`) that every element's props object
    /// carries as a trailing `"data-v-abc123": ""` pair.
    pub scope_id: Option<&'a str>,
    /// The shipped lane's `is_ts`: template expressions are TypeScript,
    /// so each one is type-erased (`emit::prefix::typescript`) before the
    /// identifier pass reads it.
    pub is_ts: bool,
    /// The shipped lane's `binding_metadata`, honoured in non-inline mode:
    /// prefixed identifiers resolve to `$setup.` / `$props.` / `$data.` /
    /// `$options.`, components and directives resolve to `$setup` members,
    /// and the render signature carries all six arguments.
    pub bindings: Option<&'a BindingTable>,
}

impl DomEmitOptions<'static> {
    /// The shipped lane's `CodegenOptions::default()` projected onto the
    /// fields the emitter honours.
    pub const DEFAULT: Self = Self {
        mode: DomEmitMode::Function,
        runtime_module_name: "vue",
        runtime_global_name: "Vue",
        prefix_identifiers: false,
        inline: false,
        component_name: None,
        cache_handlers: false,
        hoisted_scope_id: None,
        scope_id: None,
        is_ts: false,
        bindings: None,
    };
}

impl Default for DomEmitOptions<'static> {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
mod tests {
    use super::{BindingKind, BindingTable, DomEmitMode, DomEmitOptions};

    #[test]
    fn default_options_project_the_shipped_codegen_defaults() {
        assert_eq!(
            DomEmitOptions::default(),
            DomEmitOptions {
                mode: DomEmitMode::Function,
                runtime_module_name: "vue",
                runtime_global_name: "Vue",
                prefix_identifiers: false,
                inline: false,
                component_name: None,
                cache_handlers: false,
                hoisted_scope_id: None,
                scope_id: None,
                is_ts: false,
                bindings: None,
            }
        );
    }

    #[test]
    fn render_signatures_match_the_shipped_lane_per_mode() {
        assert_eq!(
            DomEmitMode::Function.render_signature(false),
            "function render(_ctx, _cache, $props, $setup, $data, $options) {"
        );
        assert_eq!(
            DomEmitMode::Module.render_signature(false),
            "export function render(_ctx, _cache) {"
        );
        assert_eq!(
            DomEmitMode::Module.render_signature(true),
            "export function render(_ctx, _cache, $props, $setup, $data, $options) {"
        );
    }

    #[test]
    fn binding_table_lookups_and_alias_order() {
        let table = BindingTable::new(
            [
                ("msg", BindingKind::SetupRef),
                ("Comp", BindingKind::SetupConst),
                ("msg", BindingKind::SetupLet),
            ],
            [("local", "prop-key")],
            true,
        );
        assert_eq!(table.kind("msg"), Some(BindingKind::SetupLet));
        assert_eq!(table.kind("Comp"), Some(BindingKind::SetupConst));
        assert_eq!(table.kind("other"), None);
        // `contains` is the membership half of the same lookup; both
        // answers are pinned, and the call stays out of the assertion so
        // the Davinci assertion lint's partial-match scan reads it as the
        // exact query it is.
        let named = (table.contains("Comp"), table.contains("other"));
        assert_eq!(named, (true, false));
        assert_eq!(
            table.aliases().collect::<alloc::vec::Vec<_>>(),
            [("local", "prop-key")]
        );
        assert!(table.is_script_setup());
    }

    #[test]
    fn non_inline_prefixes_match_the_shipped_binding_types() {
        assert_eq!(
            BindingKind::SetupRef.non_inline_template_prefix(),
            "$setup."
        );
        assert_eq!(BindingKind::Props.non_inline_template_prefix(), "$props.");
        assert_eq!(BindingKind::Data.non_inline_template_prefix(), "$data.");
        assert_eq!(
            BindingKind::Options.non_inline_template_prefix(),
            "$options."
        );
        assert_eq!(BindingKind::VueGlobal.non_inline_template_prefix(), "_ctx.");
        assert_eq!(
            BindingKind::ExternalModule.non_inline_template_prefix(),
            "$setup."
        );
    }
}
