//! Binding metadata and export tracking for Vue SFC analysis.
//!
//! Contains types for script binding resolution, undefined reference
//! detection, unused variable tracking, and export validation.

use crate::macros::{
    DEFINE_ART, DEFINE_EMITS, DEFINE_EXPOSE, DEFINE_MODEL, DEFINE_OPTIONS, DEFINE_PROPS,
    DEFINE_SLOTS, WITH_DEFAULTS,
};
use vize_carton::{CompactString, FxHashMap, String};

/// Vue compiler macro names that are auto-available in `<script setup>`.
///
/// These are transformed at compile time and should NOT be explicitly imported.
/// Used by:
/// - Patina: to lint against explicit imports of compiler macros
pub const COMPILER_MACRO_NAMES: &[&str] = &[
    DEFINE_PROPS,
    DEFINE_EMITS,
    DEFINE_EXPOSE,
    DEFINE_MODEL,
    DEFINE_OPTIONS,
    DEFINE_SLOTS,
    WITH_DEFAULTS,
    DEFINE_ART,
];
use vize_relief::BindingType;

/// Binding metadata extracted from script analysis.
///
/// This is compatible with the existing BindingMetadata in atelier_core
/// but uses CompactString for efficiency.
#[derive(Default, Clone)]
pub struct BindingMetadata {
    /// Binding name to type mapping
    pub bindings: FxHashMap<CompactString, BindingType>,

    /// Whether this is from script setup
    pub is_script_setup: bool,

    /// Props aliases (local name -> prop key)
    pub props_aliases: FxHashMap<CompactString, CompactString>,

    /// Inferred TypeScript types for legacy Vue 2.7 / Nuxt 2 Options API
    /// template bindings (currently `data()` properties), keyed by binding name.
    ///
    /// Lets the virtual-TS generator emit a precise declaration
    /// (`const count: number`) instead of `const count: any` for the common
    /// case of literal-initialized data. Gated behind the `legacy` feature, so
    /// the field — and every write to it — is compiled out of the default Vue 3
    /// build and never touches the standard pipeline.
    #[cfg(feature = "legacy")]
    pub legacy_template_binding_types: FxHashMap<CompactString, CompactString>,
}

// `Debug` is hand-written (rather than derived) so the optional, legacy-only
// `legacy_template_binding_types` side-table never leaks into Debug output or
// snapshots. It is an internal type-inference cache, not part of a binding
// set's identity, and the default Vue 3 build does not compile the field at all
// — keeping the derived-equivalent output stable across both feature configs.
impl core::fmt::Debug for BindingMetadata {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BindingMetadata")
            .field("bindings", &self.bindings)
            .field("is_script_setup", &self.is_script_setup)
            .field("props_aliases", &self.props_aliases)
            .finish()
    }
}

impl BindingMetadata {
    /// Create new empty binding metadata
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create for script setup
    #[inline]
    pub fn script_setup() -> Self {
        Self {
            is_script_setup: true,
            ..Default::default()
        }
    }

    /// Add a binding
    #[inline]
    pub fn add(&mut self, name: impl AsRef<str>, binding_type: BindingType) {
        self.bindings
            .insert(CompactString::new(name.as_ref()), binding_type);
    }

    /// Record an inferred TypeScript type for a legacy Vue 2 template binding.
    ///
    /// Legacy-only: compiled out of the default Vue 3 build.
    #[cfg(feature = "legacy")]
    #[inline]
    pub fn set_legacy_template_binding_type(&mut self, name: &str, ts_type: &str) {
        self.legacy_template_binding_types
            .insert(CompactString::new(name), CompactString::new(ts_type));
    }

    /// Get binding type for a name
    #[inline]
    pub fn get(&self, name: &str) -> Option<BindingType> {
        self.bindings.get(name).copied()
    }

    /// Check if a binding exists
    #[inline]
    pub fn contains(&self, name: &str) -> bool {
        self.bindings.contains_key(name)
    }

    /// Check if a binding is a ref (needs .value in script)
    #[inline]
    pub fn is_ref(&self, name: &str) -> bool {
        matches!(
            self.get(name),
            Some(BindingType::SetupRef | BindingType::SetupMaybeRef)
        )
    }

    /// Check if a binding is from props
    #[inline]
    pub fn is_prop(&self, name: &str) -> bool {
        matches!(
            self.get(name),
            Some(BindingType::Props | BindingType::PropsAliased)
        )
    }

    /// Iterate over all bindings
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&str, BindingType)> {
        self.bindings.iter().map(|(k, v)| (k.as_str(), *v))
    }
}

/// An undefined reference in template
#[derive(Debug, Clone)]
pub struct UndefinedRef {
    /// The identifier name
    pub name: CompactString,
    /// Source offset
    pub offset: u32,
    /// Context (e.g., "v-if expression", "interpolation")
    pub context: CompactString,
}

/// An unused template variable (v-for or v-slot)
#[derive(Debug, Clone)]
pub struct UnusedTemplateVar {
    /// The variable name
    pub name: CompactString,
    /// Source offset of the declaration
    pub offset: u32,
    /// Context where the variable is defined
    pub context: UnusedVarContext,
}

/// Context for unused template variable
#[derive(Debug, Clone)]
pub enum UnusedVarContext {
    /// Value variable in v-for (e.g., "item" in v-for="item in items")
    VForValue,
    /// Key variable in v-for (e.g., "key" in v-for="(item, key) in items")
    VForKey,
    /// Index variable in v-for (e.g., "index" in v-for="(item, index) in items")
    VForIndex,
    /// Slot prop in v-slot (e.g., "item" in v-slot="{ item }")
    VSlot { slot_name: String },
}

/// Type export from script setup (hoisted to module level)
#[derive(Debug, Clone)]
pub struct TypeExport {
    /// The type/interface name
    pub name: CompactString,
    /// Kind of export (type or interface)
    pub kind: TypeExportKind,
    /// Source offset
    pub start: u32,
    pub end: u32,
    /// Whether this is hoisted from script setup
    pub hoisted: bool,
}

/// Kind of type export
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TypeExportKind {
    Type = 0,
    Interface = 1,
}

/// Span of an import statement in script content.
#[derive(Debug, Clone, Copy)]
pub struct ImportStatementInfo {
    pub start: u32,
    pub end: u32,
}

/// Span of a re-export statement (`export { ... } from "..."`) in script content.
#[derive(Debug, Clone, Copy)]
pub struct ReExportInfo {
    pub start: u32,
    pub end: u32,
}

/// Invalid export in script setup
#[derive(Debug, Clone)]
pub struct InvalidExport {
    /// The export name
    pub name: CompactString,
    /// Kind of invalid export
    pub kind: InvalidExportKind,
    /// Source offset
    pub start: u32,
    pub end: u32,
}

/// Kind of invalid export
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InvalidExportKind {
    Const = 0,
    Let = 1,
    Var = 2,
    Function = 3,
    Class = 4,
    Default = 5,
}
