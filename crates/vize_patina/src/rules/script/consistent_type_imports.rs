//! script/consistent-type-imports
//!
//! Require `import type` for an import that is only used as a type, counting
//! every value use the component makes of it.
//!
//! ## Rationale
//!
//! A script-only linter cannot run this check on a Vue SFC: an import the
//! `<template>` renders is a value use that the script never spells. This
//! rule reads the SFC's binding analysis, so a component tag, a custom
//! directive, a template expression, a template ref and a `<style>`
//! `v-bind()` all keep an import a value import, while a `v-for` or slot
//! alias of the same name does not. On a plain TypeScript module the template
//! is absent and the rule is typescript-eslint's `consistent-type-imports`.
//!
//! A template expression or a `<style>` `v-bind()` that evaluates a name the
//! script imports only as a type is reported too: the binding does not exist
//! at runtime. `vue/no-undefined-refs` reports the template read where the
//! template spells it, so that read is left to it while it is active.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <script setup lang="ts">
//! import { Foo } from "./foo";
//! defineProps<{ item: Foo }>();
//! </script>
//! ```
//!
//! ### Valid
//! ```vue
//! <script setup lang="ts">
//! import type { Foo } from "./foo";
//! import { Bar } from "./foo";
//! defineProps<{ item: Foo }>();
//! </script>
//!
//! <template>{{ Bar.label }} {{ item.label }}</template>
//! ```

mod metadata;
mod report;
mod rewrite;
mod uses;

#[cfg(test)]
mod tests;

use oxc_ast::ast::Program;

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};
use crate::diagnostic::Severity;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/consistent-type-imports",
    description: "Enforce consistent usage of type imports, counting template uses as value uses",
    default_severity: Severity::Warning,
};

/// Which import kind a type-only import should use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TypeImportPreference {
    /// `import type` (and inline `type`) for imports only used as types.
    #[default]
    TypeImports,
    /// Never `import type`.
    NoTypeImports,
}

/// How the fix writes a type-only import.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FixStyle {
    /// `import type { A } from "m"`.
    #[default]
    SeparateTypeImports,
    /// `import { type A } from "m"`.
    InlineTypeImports,
}

/// Options of `script/consistent-type-imports`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsistentTypeImportsOptions {
    /// Preferred import kind.
    pub prefer: TypeImportPreference,
    /// Fix style under [`TypeImportPreference::TypeImports`].
    pub fix_style: FixStyle,
    /// Report `import("m")` type annotations.
    pub disallow_type_annotations: bool,
    /// Whether the project compiles with TypeScript's `emitDecoratorMetadata`.
    /// A type a decorated signature serializes is then a runtime value.
    pub emit_decorator_metadata: bool,
}

impl ConsistentTypeImportsOptions {
    /// typescript-eslint's defaults. Decorator metadata is assumed on, because
    /// lint does not read `tsconfig.json` and the opposite guess is unsafe.
    pub const DEFAULT: Self = Self {
        prefer: TypeImportPreference::TypeImports,
        fix_style: FixStyle::SeparateTypeImports,
        disallow_type_annotations: true,
        emit_decorator_metadata: true,
    };
}

impl Default for ConsistentTypeImportsOptions {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Enforce consistent usage of type imports across an SFC's blocks.
pub struct ConsistentTypeImports {
    options: ConsistentTypeImportsOptions,
}

impl ConsistentTypeImports {
    /// The rule with its default options.
    pub const fn new() -> Self {
        Self {
            options: ConsistentTypeImportsOptions::DEFAULT,
        }
    }

    /// The rule with explicit options.
    pub const fn configured(options: ConsistentTypeImportsOptions) -> Self {
        Self { options }
    }

    /// The rule with `linter.ruleOptions["script/consistent-type-imports"]`.
    /// A key the configuration leaves out keeps its default.
    pub fn from_options(options: &vize_l0::config::ConsistentTypeImportsOptions) -> Self {
        use vize_l0::config::{TypeImportFixStyle, TypeImportPreference as Preference};
        let default = ConsistentTypeImportsOptions::DEFAULT;
        Self::configured(ConsistentTypeImportsOptions {
            prefer: match options.prefer {
                Some(Preference::NoTypeImports) => TypeImportPreference::NoTypeImports,
                Some(Preference::TypeImports) | None => TypeImportPreference::TypeImports,
            },
            fix_style: match options.fix_style {
                Some(TypeImportFixStyle::InlineTypeImports) => FixStyle::InlineTypeImports,
                Some(TypeImportFixStyle::SeparateTypeImports) | None => {
                    FixStyle::SeparateTypeImports
                }
            },
            disallow_type_annotations: options
                .disallow_type_annotations
                .unwrap_or(default.disallow_type_annotations),
            emit_decorator_metadata: options
                .emit_decorator_metadata
                .unwrap_or(default.emit_decorator_metadata),
        })
    }
}

impl Default for ConsistentTypeImports {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptRule for ConsistentTypeImports {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        self.check_program_with_sfc(program, source, offset, SfcScriptContext::default(), result);
    }

    fn check_program_with_sfc<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        sfc: SfcScriptContext<'_>,
        result: &mut ScriptLintResult,
    ) {
        report::check(&self.options, program, source, offset, sfc, result);
    }
}
