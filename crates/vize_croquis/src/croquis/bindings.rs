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
#[derive(Debug, Default, Clone)]
pub struct BindingMetadata {
    /// Binding name to type mapping
    pub bindings: FxHashMap<CompactString, BindingType>,

    /// Whether this is from script setup
    pub is_script_setup: bool,

    /// Props aliases (local name -> prop key)
    pub props_aliases: FxHashMap<CompactString, CompactString>,
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

/// API shape of the component's default export.
///
/// Recorded by script analysis so downstream consumers (linter, virtual TS,
/// LSP) can tell how the component is authored without re-walking the AST.
/// Purely additive metadata: `Unspecified` preserves pre-existing behavior
/// for every non-class component.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum ComponentShape {
    /// No specific shape detected (script setup, options object, or no
    /// default export).
    #[default]
    Unspecified = 0,
    /// The default export is a class (vue-class-component /
    /// vue-property-decorator style), optionally decorated with
    /// `@Component` / `@Options`.
    ClassApi = 1,
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
