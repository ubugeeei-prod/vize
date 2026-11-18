//! vue/prefer-pug-lang
//!
//! Report `<template lang="jade">` and every construct in it whose jade 1.x
//! meaning differs from pug 3.
//!
//! Vue's toolchain renders `lang="jade"` with the legacy `jade` 1.x package,
//! not pug. Vize reads the template with its pug 3 grammar under a jade
//! profile and reports each construct where the two languages part ways:
//! `- each` / `- for`, the bodiless `mixin name(args)` call, `each … of`
//! (a literal `<each>` element in jade 1.x) and `#{…}` in an attribute value
//! (interpolated to `undefined` by jade 1.x at build time).
//!
//! The `lang` value is fixed to `pug` automatically only when no jade-only
//! construct is left; each construct carries its own fix.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template lang="jade">
//! ul
//!   - each item in items
//!     li= item
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <template lang="pug">
//! ul
//!   each item in items
//!     li= item
//! </template>
//! ```
//!
//! The diagnostics come from the pug lane of the SFC linter, which reads the
//! authored jade; this registration carries the rule's metadata and config.

use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

pub(crate) static META: RuleMeta = RuleMeta {
    name: "vue/prefer-pug-lang",
    description: "Prefer lang=\"pug\" over the legacy jade 1.x template language",
    category: RuleCategory::Recommended,
    fixable: true,
    default_severity: Severity::Warning,
};

/// Prefer `lang="pug"` over `lang="jade"`.
#[derive(Default)]
pub struct PreferPugLang;

impl Rule for PreferPugLang {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}
