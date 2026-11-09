//! Vue script meaning, read by L1→L2, L3, L4 and the products.
//!
//! Compiler macros are keyed by callee name in one const table. A callee
//! outside [`MACROS`] is an ordinary call.

pub const DEFINE_PROPS: &str = "defineProps";
pub const DEFINE_EMITS: &str = "defineEmits";
pub const DEFINE_EXPOSE: &str = "defineExpose";
pub const DEFINE_OPTIONS: &str = "defineOptions";
pub const DEFINE_SLOTS: &str = "defineSlots";
pub const DEFINE_MODEL: &str = "defineModel";
pub const WITH_DEFAULTS: &str = "withDefaults";

/// A compiler macro, named by its callee.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroKind {
    DefineProps,
    DefineEmits,
    DefineExpose,
    DefineOptions,
    DefineSlots,
    DefineModel,
    WithDefaults,
    /// Musea `defineArt(component, options)`.
    DefineArt,
    /// vue-router `definePage`.
    DefinePage,
    /// Nuxt `definePageMeta`.
    DefinePageMeta,
    /// Nuxt `defineRouteRules`.
    DefineRouteRules,
    DefineLazyHydrationComponent,
}

/// How the SFC compiler treats a macro call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroLifecycle {
    /// Analyzed, then removed from the setup body.
    AnalyzeAndErase,
    /// Rewritten into runtime code.
    Expand,
    /// Extracted as a compiler artifact, then removed from the setup body.
    ExtractAndErase,
}

impl MacroLifecycle {
    #[must_use]
    pub const fn erases_from_runtime(self) -> bool {
        matches!(self, Self::AnalyzeAndErase | Self::ExtractAndErase)
    }

    #[must_use]
    pub const fn produces_artifact(self) -> bool {
        matches!(self, Self::ExtractAndErase)
    }
}

/// One row of the macro table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Macro {
    pub name: &'static str,
    pub kind: MacroKind,
    pub lifecycle: MacroLifecycle,
    /// Vue's own compiler macro, as opposed to an ecosystem one.
    pub builtin: bool,
    /// The artifact kind an extracted call produces.
    pub artifact: Option<&'static str>,
}

const fn vue(name: &'static str, kind: MacroKind) -> Macro {
    Macro {
        name,
        kind,
        lifecycle: MacroLifecycle::AnalyzeAndErase,
        builtin: true,
        artifact: None,
    }
}

const fn ecosystem(
    name: &'static str,
    kind: MacroKind,
    lifecycle: MacroLifecycle,
    artifact: Option<&'static str>,
) -> Macro {
    Macro {
        name,
        kind,
        lifecycle,
        builtin: false,
        artifact,
    }
}

/// Every macro, Vue's first and then the ecosystem's, in compiler order.
pub const MACROS: &[Macro] = &[
    vue(DEFINE_PROPS, MacroKind::DefineProps),
    vue(DEFINE_EMITS, MacroKind::DefineEmits),
    vue(DEFINE_EXPOSE, MacroKind::DefineExpose),
    vue(DEFINE_OPTIONS, MacroKind::DefineOptions),
    vue(DEFINE_SLOTS, MacroKind::DefineSlots),
    vue(DEFINE_MODEL, MacroKind::DefineModel),
    vue(WITH_DEFAULTS, MacroKind::WithDefaults),
    ecosystem(
        "defineArt",
        MacroKind::DefineArt,
        MacroLifecycle::AnalyzeAndErase,
        None,
    ),
    ecosystem(
        "definePage",
        MacroKind::DefinePage,
        MacroLifecycle::ExtractAndErase,
        Some("vue-router.definePage"),
    ),
    ecosystem(
        "definePageMeta",
        MacroKind::DefinePageMeta,
        MacroLifecycle::ExtractAndErase,
        Some("nuxt.definePageMeta"),
    ),
    ecosystem(
        "defineRouteRules",
        MacroKind::DefineRouteRules,
        MacroLifecycle::ExtractAndErase,
        Some("nuxt.defineRouteRules"),
    ),
    ecosystem(
        "defineLazyHydrationComponent",
        MacroKind::DefineLazyHydrationComponent,
        MacroLifecycle::Expand,
        None,
    ),
];

/// The table row for a callee name.
#[must_use]
pub fn lookup(name: &str) -> Option<&'static Macro> {
    MACROS.iter().find(|row| row.name == name)
}

#[must_use]
pub fn is_builtin_macro(name: &str) -> bool {
    lookup(name).is_some_and(|row| row.builtin)
}

/// Whether a call is removed from the setup body.
#[must_use]
pub fn is_runtime_erased_macro(name: &str) -> bool {
    lookup(name).is_some_and(|row| row.lifecycle.erases_from_runtime())
}

#[must_use]
pub fn macro_artifact_kind(name: &str) -> Option<&'static str> {
    lookup(name).and_then(|row| row.artifact)
}

/// Names removed from the setup body, in table order.
pub fn runtime_erased_macro_names() -> impl Iterator<Item = &'static str> {
    MACROS
        .iter()
        .filter(|row| row.lifecycle.erases_from_runtime())
        .map(|row| row.name)
}

/// Names whose calls produce a compiler artifact, in table order.
pub fn artifact_macro_names() -> impl Iterator<Item = &'static str> {
    MACROS
        .iter()
        .filter(|row| row.lifecycle.produces_artifact())
        .map(|row| row.name)
}

mod setup;
pub use setup::{
    MacroCall, PropsDestructure, PropsField, ReactiveSource, SetupBinding, SetupBindingKind,
    SetupCheckpoint, SetupFacts, SetupFactsIncomplete, SetupObserver,
};
