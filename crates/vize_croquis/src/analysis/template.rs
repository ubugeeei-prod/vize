//! Template-level types for Vue SFC analysis.
//!
//! Contains metadata about template structure, component usage,
//! template expressions, and element IDs.

use vize_carton::{CompactString, SmallVec};

/// Template-level metadata collected during analysis.
#[derive(Debug, Clone, Default)]
pub struct TemplateInfo {
    /// Number of root elements at depth 0 in template.
    /// A value > 1 indicates multi-root component (fragments).
    pub root_element_count: usize,
    /// Whether $attrs is referenced anywhere in the template.
    pub uses_attrs: bool,
    /// Whether v-bind="$attrs" is explicitly used (not just $attrs.class etc.)
    pub binds_attrs_explicitly: bool,
    /// Whether inheritAttrs: false is set in defineOptions.
    pub inherit_attrs_disabled: bool,
    /// Start offset of template content (relative to template block).
    pub content_start: u32,
    /// End offset of template content (relative to template block).
    pub content_end: u32,
}

impl TemplateInfo {
    /// Check if the component has multiple root elements.
    #[inline]
    pub fn has_multiple_roots(&self) -> bool {
        self.root_element_count > 1
    }

    /// Check if fallthrough attrs may be lost (multi-root without explicit binding).
    #[inline]
    pub fn may_lose_fallthrough_attrs(&self) -> bool {
        self.has_multiple_roots() && !self.binds_attrs_explicitly
    }
}

/// Information about element IDs in template (for cross-file uniqueness checking).
#[derive(Debug, Clone)]
pub struct ElementIdInfo {
    /// The ID value (for static IDs) or expression (for dynamic IDs)
    pub value: CompactString,
    /// Start offset in template
    pub start: u32,
    /// End offset in template
    pub end: u32,
    /// Whether this is a static ID (vs dynamic :id binding)
    pub is_static: bool,
    /// Whether this is inside a v-for loop
    pub in_loop: bool,
    /// The scope this ID belongs to
    pub scope_id: crate::scope::ScopeId,
    /// Kind of ID (id attribute, for reference, aria reference, etc.)
    pub kind: ElementIdKind,
}

/// Kind of element ID or ID reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementIdKind {
    /// id="..." or :id="..."
    Id,
    /// for="..." or :for="..."
    For,
    /// aria-labelledby, aria-describedby, aria-controls, etc.
    AriaReference,
    /// headers, list, form, popovertarget, anchor
    OtherReference,
}

impl ElementIdKind {
    /// Get the string representation.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::For => "for",
            Self::AriaReference => "aria-reference",
            Self::OtherReference => "other-reference",
        }
    }

    /// Check if this is an ID definition (not a reference).
    #[inline]
    pub const fn is_definition(&self) -> bool {
        matches!(self, Self::Id)
    }

    /// Check if this is an ID reference.
    #[inline]
    pub const fn is_reference(&self) -> bool {
        !self.is_definition()
    }
}

/// Template expression for type checking.
#[derive(Debug, Clone)]
pub struct TemplateExpression {
    /// The expression content
    pub content: CompactString,
    /// Kind of expression
    pub kind: TemplateExpressionKind,
    /// Start offset in template (relative to template block)
    pub start: u32,
    /// End offset in template (relative to template block)
    pub end: u32,
    /// The scope this expression belongs to
    pub scope_id: crate::scope::ScopeId,
    /// v-if guard condition (if this expression is inside a v-if block)
    pub vif_guard: Option<CompactString>,
}

/// Kind of template expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateExpressionKind {
    /// Mustache interpolation: {{ expr }}
    Interpolation,
    /// v-bind: :prop="expr" or v-bind:prop="expr"
    VBind,
    /// v-on handler (non-inline): @event="handler"
    VOn,
    /// v-if condition: v-if="cond"
    VIf,
    /// v-show condition: v-show="cond"
    VShow,
    /// v-model: v-model="value"
    VModel,
}

impl TemplateExpressionKind {
    /// Get the string representation without allocation.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Interpolation => "Interpolation",
            Self::VBind => "VBind",
            Self::VOn => "VOn",
            Self::VIf => "VIf",
            Self::VShow => "VShow",
            Self::VModel => "VModel",
        }
    }
}

/// Information about a component used in template.
///
/// Uses SmallVec to avoid heap allocations for typical component usage
/// (most components have < 8 props, < 4 events, < 2 slots).
#[derive(Debug, Clone)]
pub struct ComponentUsage {
    /// Component name (e.g., "MyButton", "user-card")
    pub name: CompactString,
    /// Start offset in template
    pub start: u32,
    /// End offset in template
    pub end: u32,
    /// Props passed to this component (stack-allocated for ≤8 props)
    pub props: SmallVec<[PassedProp; 8]>,
    /// Event listeners on this component (stack-allocated for ≤4 events)
    pub events: SmallVec<[EventListener; 4]>,
    /// Slots provided to this component (stack-allocated for ≤2 slots)
    pub slots: SmallVec<[SlotUsage; 2]>,
    /// Whether v-bind="$attrs" or similar spread is used
    pub has_spread_attrs: bool,
    /// The scope this component usage is in (for v-for prop checking)
    pub scope_id: crate::scope::ScopeId,
    /// Combined v-if / v-else-if guard active for this component usage.
    pub vif_guard: Option<CompactString>,
}

/// A prop passed to a component in template.
#[derive(Debug, Clone)]
pub struct PassedProp {
    /// Prop name (kebab-case or camelCase as written)
    pub name: CompactString,
    /// The expression if dynamic, or literal value if static
    pub value: Option<CompactString>,
    /// Start offset
    pub start: u32,
    /// End offset
    pub end: u32,
    /// Whether this is a dynamic binding (:prop or v-bind:prop)
    pub is_dynamic: bool,
}

/// An event listener on a component.
#[derive(Debug, Clone)]
pub struct EventListener {
    /// Event name (e.g., "click", "update:modelValue")
    pub name: CompactString,
    /// Handler expression
    pub handler: Option<CompactString>,
    /// Modifiers (stack-allocated for ≤4 modifiers)
    pub modifiers: SmallVec<[CompactString; 4]>,
    /// Start offset
    pub start: u32,
    /// End offset
    pub end: u32,
}

/// A slot provided to a component.
#[derive(Debug, Clone)]
pub struct SlotUsage {
    /// Slot name ("default" if unnamed)
    pub name: CompactString,
    /// Scope variable names if any (stack-allocated for ≤4 vars)
    pub scope_vars: SmallVec<[CompactString; 4]>,
    /// Start offset
    pub start: u32,
    /// End offset
    pub end: u32,
    /// Whether this slot has scope (v-slot:name="scope")
    pub has_scope: bool,
}
