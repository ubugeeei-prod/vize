//! Shared Vue template capability model.
//!
//! Per-version modules own the capability tables. Both compatibility parser
//! options and L1→L2 legalization resolve the same table once per file.
//! Historical public names remain available through Armature's adapter.

use vize_l0::config::VueVersion;

/// A legacy (pre-Vue-3) version line that Vize can opt into supporting.
///
/// Vize targets Vue 3 by default. These variants name the older runtimes whose
/// template syntax, reactivity, and component model differ enough from Vue 3 to
/// require a dedicated parsing / analysis path behind the `legacy` feature:
///
/// - [`V0_10`](Self::V0_10): Vue 0.10.x, the last pre-rewrite 0.x line.
/// - [`V0_11`](Self::V0_11): the 0.11-era post-rewrite 0.x line (previously
///   modeled as `V0`).
/// - [`V1`](Self::V1): Vue 1.x (`v-for`, `track-by`, `$index`, `v-el`, …).
/// - [`V2`](Self::V2): Vue 2.x, including 2.7 (Options-API-first; `slot-scope`,
///   `.sync`, filters, functional components, single root, …).
///
/// Vue 0.10 and the 0.11-era line are deliberately distinct: 0.11.0 was a
/// ground-up rewrite with documented breaking changes (see the per-variant
/// docs), so the two cannot share one dialect entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LegacyVueVersion {
    /// Vue 0.10.x — the last pre-rewrite 0.x line.
    ///
    /// Templates iterate with `v-repeat` (no `v-for`; that arrives in 1.0),
    /// inherit parent data with `v-with`, mount components with
    /// `v-component`, and bind events with directive-value clauses
    /// (`v-on="click: onClick"`); the 1.0+ colon-argument forms
    /// (`v-on:click`, `@click`, `:prop`) do not exist yet. Computed
    /// properties use the `$get` / `$set` object form, which the Vue 0.11.0
    /// rewrite renamed to plain `get` / `set` (Vue 0.11 `changes.md`,
    /// "Computed Properties"). That rename, plus the 0.11 instantiation and
    /// scope-inheritance rework, is why this line is modeled separately from
    /// [`V0_11`](Self::V0_11).
    V0_10,
    /// Vue 0.11-era — the post-rewrite 0.x line (0.11.x and the transitional
    /// 0.12.x releases). Previously modeled as `V0`.
    ///
    /// Shares the clause-style directive surface with 0.10 (`v-repeat`,
    /// `v-with`, `v-component`, `v-on="click: onClick"`), but the 0.11.0
    /// rewrite changed instantiation/scope semantics and renamed computed
    /// `$get` / `$set` to `get` / `set` (Vue 0.11 `changes.md`). During 0.12
    /// the line began migrating to `props`, deprecating `v-with` and
    /// `v-component` (Vue 0.12 release notes) ahead of Vue 1.0's new
    /// directive syntax.
    V0_11,
    /// Vue 1.x.
    ///
    /// The 1.0 migration replaced `v-repeat` with `v-for` (`track-by`,
    /// `$index`) and introduced colon-argument directive syntax with
    /// shorthands (`v-bind:prop` / `:prop`, `v-on:click` / `@click`) per the
    /// "Migrating from 0.12" guide in the Vue 1.0 docs. Mustache
    /// interpolation still works inside plain attribute values
    /// (`href="{{ url }}"`), and filters take space-separated arguments
    /// (`{{ msg | filterBy 'a' }}`).
    V1,
    /// Vue 2.x, including 2.7.
    ///
    /// Options-API-first. Keeps pipe filters but switches their arguments to
    /// call-style (`{{ msg | filterBy('a') }}`; Vue 2 migration guide,
    /// "Filter Argument Syntax") and removes attribute-value mustache
    /// interpolation in favor of `v-bind` ("Interpolation within
    /// Attributes"). Adds scoped-slot sugar (`scope` in 2.1, `slot-scope` in
    /// 2.5), the `.sync` and `.native` modifiers, numeric key modifiers
    /// (`@keyup.13`), functional components, and the single-root template
    /// requirement. Vue 2.7 backports `<script setup>` but shares this
    /// template dialect.
    V2,
}

impl LegacyVueVersion {
    /// Every supported legacy line, oldest first.
    pub const ALL: [LegacyVueVersion; 4] = [Self::V0_10, Self::V0_11, Self::V1, Self::V2];

    /// A short, stable identifier for the line (`"v0.10"`, `"v0.11"`, `"v1"`,
    /// `"v2"`).
    ///
    /// Kept stable so it can be used in diagnostics and feature reporting
    /// without churning across releases. Config-side parsing lives on
    /// [`VueVersion`] in `vize_l0`, which accepts these identifiers as
    /// well as the bare `vue.version` numbers.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::V0_10 => "v0.10",
            Self::V0_11 => "v0.11",
            Self::V1 => "v1",
            Self::V2 => "v2",
        }
    }

    /// Resolve a config-selected [`VueVersion`] into a legacy line.
    ///
    /// Returns `None` for [`VueVersion::V3`]: modern Vue 3 is not a legacy
    /// line and must take the default (non-legacy) code paths. Vue 2.7 and
    /// Vue 2 share the [`V2`](Self::V2) template dialect.
    pub const fn from_dialect(dialect: VueVersion) -> Option<Self> {
        match dialect {
            VueVersion::V3 => None,
            VueVersion::V2_7 | VueVersion::V2 => Some(Self::V2),
            VueVersion::V1 => Some(Self::V1),
            VueVersion::V0_11 => Some(Self::V0_11),
            VueVersion::V0_10 => Some(Self::V0_10),
        }
    }

    /// Resolve this line into its template-syntax capability set.
    ///
    /// Call this once per file/document when building parser or transform
    /// options; see [`LegacyDialectCapabilities`] for the zero-cost contract.
    pub const fn capabilities(self) -> LegacyDialectCapabilities {
        match self {
            Self::V0_10 => super::vue0::V0_10,
            Self::V0_11 => super::vue0::V0_11,
            Self::V1 => super::vue1::CAPABILITIES,
            Self::V2 => super::vue2::CAPABILITIES,
        }
    }
}

/// How a dialect spells directive arguments.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum DirectiveArgStyle {
    /// 0.x clause syntax: the directive *value* carries `arg: expression`
    /// clauses, comma-separable (`v-on="click: onClick, keyup: onKeyup"`).
    Clause,
    /// Colon-argument syntax (`v-on:click="onClick"`) with the `@` / `:`
    /// shorthands, introduced by Vue 1.0's directive-syntax rework and kept
    /// through Vue 2 and Vue 3. This is the default (Vue 3) style.
    #[default]
    Colon,
}

/// Template-syntax capabilities of a dialect, resolved once per document.
///
/// # Zero-cost contract
///
/// Resolve this set exactly once per file — when parser/transform options are
/// built — via [`LegacyDialectCapabilities::for_dialect`] or
/// [`LegacyVueVersion::capabilities`]. Hot paths only read these fields (a
/// field read of a `Copy` struct), and must never re-match on
/// [`LegacyVueVersion`] per token or per node. The default dialect resolves to
/// the all-off [`VUE3`](Self::VUE3) set, so every capability check
/// short-circuits to the same branch the unconditional Vue 3 code takes today;
/// The compatibility parser adapter remains gated by its `legacy` feature.
///
/// Each field documents the version lines it covers and the upstream Vue
/// changelog / migration-guide entry that introduced or removed the surface.
// `#[non_exhaustive]` so adding a capability field as new legacy Vue surfaces
// land stays a non-breaking minor change: downstream crates construct this only
// through [`for_dialect`](Self::for_dialect) / [`VUE3`](Self::VUE3), never a
// struct literal, and the per-push cargo-semver-checks gate would otherwise flag
// every field addition as breaking (`constructible_struct_adds_field`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct LegacyDialectCapabilities {
    /// Pipe filters in text interpolations and binding values
    /// (`{{ msg | capitalize }}`). All legacy lines; removed in Vue 3
    /// (Vue 3 migration guide, "Filters").
    pub supports_filters: bool,
    /// Space-separated filter arguments (`{{ msg | filterBy 'a' }}`), the
    /// 0.x / 1.x form. Vue 2 switched filters to call-style arguments
    /// (Vue 2 migration guide, "Filter Argument Syntax").
    pub space_separated_filter_args: bool,
    /// List rendering via `v-repeat` (with implicit `$index`). 0.x lines
    /// only; Vue 1.0 replaced it with `v-for` ("Migrating from 0.12").
    pub v_repeat_syntax: bool,
    /// How directive arguments are spelled; see [`DirectiveArgStyle`].
    pub directive_arg_style: DirectiveArgStyle,
    /// Parent-data inheritance via the `v-with` directive. 0.x lines only;
    /// deprecated during 0.12 in favor of `props`.
    pub v_with_directive: bool,
    /// Component mounting via the `v-component` directive. 0.x lines only;
    /// deprecated during 0.12 in favor of component elements / `is`.
    pub v_component_directive: bool,
    /// Computed properties using the `$get` / `$set` object form. Vue 0.10
    /// only; the 0.11.0 rewrite renamed them to plain `get` / `set`
    /// (Vue 0.11 `changes.md`).
    pub computed_dollar_get_set: bool,
    /// Mustache interpolation inside plain attribute values
    /// (`href="{{ url }}"`). 0.x / 1.x; removed in Vue 2 in favor of
    /// `v-bind` (Vue 2 migration guide, "Interpolation within Attributes").
    pub attr_value_interpolation: bool,
    /// Scoped-slot attribute sugar: `scope` (added in 2.1) and `slot-scope`
    /// (added in 2.5). Vue 2 only; superseded by `v-slot` in 2.6 and Vue 3.
    pub scoped_slot_attrs: bool,
    /// Triple-mustache raw-HTML (unescaped) text interpolation,
    /// `{{{ html }}}` — the pre-Vue-2 equivalent of `v-html`. Vue 0.x / 1.x;
    /// removed in Vue 2 in favor of `v-html` (Vue 2 migration guide,
    /// "Interpolation"). Vue 2+ treats `{{{ x }}}` as an ordinary `{{ … }}`
    /// mustache containing a stray brace, which is also the default (Vue 3)
    /// behavior.
    pub raw_html_interpolation: bool,
    /// `.native` and numeric key-code event sugar. Vue 2 / 2.7 only.
    pub v2_event_sugar: bool,
}

impl LegacyDialectCapabilities {
    /// The default (modern Vue 3) capability set: every legacy surface off,
    /// colon-style directive arguments.
    ///
    /// This is the set a `legacy`-enabled build resolves for Vue 3 sources,
    /// guaranteeing they take branch-identical paths to a build without the
    /// feature.
    pub const VUE3: LegacyDialectCapabilities = super::vue3::CAPABILITIES;

    /// Resolve a config-selected [`VueVersion`] straight to its capability
    /// set ([`VUE3`](Self::VUE3) for the default dialect).
    ///
    /// This is the once-per-file entry point for option builders.
    pub const fn for_dialect(dialect: VueVersion) -> LegacyDialectCapabilities {
        match LegacyVueVersion::from_dialect(dialect) {
            None => Self::VUE3,
            Some(version) => version.capabilities(),
        }
    }
}

impl Default for LegacyDialectCapabilities {
    fn default() -> Self {
        Self::VUE3
    }
}

#[cfg(test)]
mod tests;
