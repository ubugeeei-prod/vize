//! The binding-surface projection (P2-9 series 5): per owner (element,
//! component, slot outlet), the props surface DOM codegen compiles —
//! static attributes, `v-bind` one-way bindings, `v-on` handlers, custom
//! directives, and the `v-model` contract — compared at the
//! DOM-output-determining level: names (static text, dynamic expression
//! text, or the spread form), modifier lists verbatim, and value texts
//! trimmed.
//!
//! # The owner rule, lane-neutral
//!
//! An owner is every element, component, template carrier, or slot
//! outlet **both** trees keep: the legacy collector skips exactly the
//! wrapper elements the S2 lowering unwraps (`<template v-if>` branch
//! carriers, `<template v-for>` wrappers) and counts their leftover
//! props (`wrapper_attrs` — the recorded wrapper-facts gap, measured on
//! the binding surface for the first time). An outlet's consumed `name`
//! position is excluded by the S2 lowering's own selection rule,
//! mirrored on the legacy side; an empty `:name` is the recorded
//! `drop.slot-name-hole` agreement.
//!
//! # Counted classes, never silence
//!
//! - `builtins_excluded` — legacy props whose directive S2 still defers
//!   (`v-cloak`, `v-pre`) or drops as a structural duplicate, plus the
//!   codegen-only dialect flags (`v-once`, `v-memo`, `v-show`,
//!   `v-html`, `v-text`) that lower as Vue dialect bindings
//!   but are not part of this bind/on/model/directive projection;
//!   excluded on both sides, counted so the remaining set keeps a
//!   number.
//! - `wrapper_attrs` — see above.
//! - `values_compound` — a legacy compound rebuild has no single source
//!   text (never seen under default options).
//! - `entity_templates` — the legacy parser decodes entities in
//!   attribute and directive values; S1 v1 deliberately does not (the
//!   text projection's class, measured over the binding surface for the
//!   first time): the template's surface half skips as one counted
//!   class.
//! - `table_templates` — the legacy parser's in-table tree construction
//!   (foster parenting, implicit sections) against S1's authored
//!   nesting; the whole template's surface half skips as one counted
//!   class ([`SurfaceCounters::table_templates`]).
//! - `keys_excluded` — the branch key the v-if pass extracted off a
//!   carrier's binding surface (the legacy lane removed the same prop
//!   into `user_key`); excluded here because the chain projection
//!   compares it.
//! - the model classes — [`check`]'s docs; `models_dynamic_arg` is a
//!   closed residual counter pinned at zero, and
//!   `models_pattern_scope` is an owner under a slot scope with
//!   destructuring params: the legacy lane enumerates the pattern's
//!   names, S2 deliberately does not until the #4365 seam — the
//!   on-scope verdicts can differ, so the class is counted, never
//!   compared.

use vize_s0::String;

/// The comparator's surface accounting, part of [`super::Counters`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SurfaceCounters {
    /// Owners compared (attrs, binds, ons, directives, models all held).
    pub owners: u64,
    /// Static attribute name/value comparisons that ran.
    pub attrs: u64,
    /// `v-bind` comparisons that ran (static-named).
    pub binds: u64,
    /// Dynamic-name binds compared.
    pub binds_dynamic: u64,
    /// Object spreads (`v-bind="obj"`) compared.
    pub binds_spread: u64,
    /// `v-on` comparisons that ran (static-named).
    pub ons: u64,
    /// Dynamic-name handlers compared.
    pub ons_dynamic: u64,
    /// Object forms (`v-on="handlers"`) compared.
    pub ons_spread: u64,
    /// Custom directives compared.
    pub directives: u64,
    /// `v-model` contracts compared.
    pub models: u64,
    /// Both lanes dropped the model: the S2 fault fact met the legacy
    /// removal.
    pub models_invalid: u64,
    /// Closed residual: dynamic-argument component models are compared.
    pub models_dynamic_arg: u64,
    /// An owner under a destructuring-params slot scope: model verdicts
    /// may differ (#4365), counted per owner, never compared.
    pub models_pattern_scope: u64,
    /// Branch-key bindings excluded here because the chain projection
    /// owns them.
    pub keys_excluded: u64,
    /// Legacy props S2 defers or drops (the remaining `defer.*` set).
    pub builtins_excluded: u64,
    /// Props on unwrapped template wrappers (the wrapper-facts gap,
    /// re-measured on the binding surface).
    pub wrapper_attrs: u64,
    /// Templates skipped: entity-shaped attribute or value text.
    pub entity_templates: u64,
    /// Templates skipped: the template authors a `<table>`. The legacy
    /// parser runs in-table tree construction (foster parenting of
    /// non-table content and text, implicit `tbody`/`tr` insertion,
    /// sibling closing — `vize_armature/src/parser/element/table.rs`)
    /// that S1 v1 deliberately does not, so the two trees can genuinely
    /// disagree on owner order and count inside table subtrees; counted
    /// per template, never averaged, owned by the future S1 tree-
    /// reconciliation story. Caught by the corpus lane on its first
    /// surface run (element-plus `table.vue`, `<hColgroup>` fostered
    /// out of its `<table>`).
    pub table_templates: u64,
    /// Legacy compound rebuilds: counted, not compared.
    pub values_compound: u64,
}

/// A name position: static text, dynamic expression text (`None` = a
/// legacy compound rebuild), or the spread/object form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PName {
    Static(String),
    Dynamic(Option<String>),
    Spread,
}

/// One `v-bind` or `v-on` unit: name, modifiers verbatim, value text
/// (`None` = no value authored; `Some(None)` = compound rebuild).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PBind {
    pub name: PName,
    pub mods: Vec<String>,
    pub value: Option<Option<String>>,
}

/// One custom directive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PDirective {
    pub name: String,
    pub arg: Option<PName>,
    pub mods: Vec<String>,
    pub value: Option<Option<String>>,
}

/// One `v-model` contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PModel {
    /// The model value's trimmed text (`None` = compound rebuild).
    pub value: Option<String>,
    /// The effective prop position: the authored argument, with the
    /// component default (`modelValue`) applied — the spelling the
    /// legacy product props carry.
    pub prop: Option<PName>,
    /// Modifier names in authored order.
    pub mods: Vec<String>,
    /// Whether the owner is a component.
    pub component: bool,
}

/// One owner's projected surface.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PSurface {
    pub attrs: Vec<(String, Option<String>)>,
    pub binds: Vec<PBind>,
    pub ons: Vec<PBind>,
    pub directives: Vec<PDirective>,
    pub models: Vec<PModel>,
    /// The owner sits under a destructuring-params slot scope.
    pub pattern_scoped: bool,
}

/// The shared simple-identifier predicate both collectors use to decide
/// the pattern-scope class: one leading identifier character, identifier
/// characters after. Lane-neutral by construction — neither parser's
/// classifier enters it.
pub fn is_simple_ident(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_alphabetic() || first == '_' || first == '$')
        && chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}
