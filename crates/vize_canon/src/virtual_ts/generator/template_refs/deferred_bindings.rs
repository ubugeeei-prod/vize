//! Template shadows for setup bindings whose assignment is deferred.
//!
//! Vue runs `setup()` to completion — including top-level `await` and any
//! callback that fires synchronously, such as a `{ immediate: true }` watcher —
//! before the render function can observe a single binding. `vue-tsc` models
//! that boundary structurally: template expressions reach setup bindings
//! through a `typeof` type query on its `__VLS_ctx` object, and a type query
//! carries the *declared* type without running control-flow analysis. So
//! `vue-tsc` cannot report TS2454 for a template read, whatever the assignment
//! looks like, while a read that stays inside the setup body keeps exact
//! TypeScript control-flow semantics and still reports.
//!
//! Vize emits template expressions as direct lexical references inside the
//! nested `__template()` function. TypeScript normally assumes an outer
//! variable is initialized when it is read from a nested function, but that
//! assumption is withdrawn for a mutable local declared without an initializer
//! whose every assignment is itself conditional or nested in another function
//! (`isNeverInitialized` in the checker). Those bindings therefore reached
//! control-flow analysis and produced TS2454 that `vue-tsc` never emits.
//!
//! The fix restores the type query. Each such binding gets a captured alias in
//! setup scope and a shadow declaration in template scope:
//!
//! ```ts
//! type __D_paginator = typeof paginator;
//! ;(function __template() {
//!   var paginator: __D_paginator = undefined as any;
//! ```
//!
//! The shadow's type is exactly `typeof paginator`, so the template keeps the
//! binding's declared type — nothing is widened, no `undefined` is added, and
//! no diagnostic is filtered. Setup-body reads are untouched, so genuinely
//! conditional or asynchronous assignment still reports TS2454 there.

use oxc_allocator::Allocator;
use oxc_ast::ast::{BindingPattern, Statement, VariableDeclarationKind};
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;
use std::ops::Range;

use vize_croquis::{BindingType, Croquis};
use vize_s0::{FxHashMap, FxHashSet, String, cstr};

use crate::virtual_ts::{VizeSemanticLink, VizeSemanticLinkKind};

/// Collects template-referenced bindings that need the type-query shadow.
///
/// `already_shadowed` reports the names template scope already redeclares for
/// ref unwrapping; shadowing one twice would redeclare it with a conflicting
/// type.
pub(super) fn collect_deferred_setup_bindings(
    summary: &Croquis,
    script_content: Option<&str>,
    template_referenced_names: Option<&FxHashSet<String>>,
    already_shadowed: impl Fn(&str) -> bool,
) -> Vec<String> {
    // Only `<script setup>` puts its bindings in the template's lexical scope.
    // A plain `<script>` reaches the template through the component instance,
    // and the Options API through the `__VizeOptionsSetupBinding` type query,
    // which never runs control-flow analysis to begin with.
    if !summary.bindings.is_script_setup {
        return Vec::new();
    }
    let (Some(script), Some(referenced)) = (script_content, template_referenced_names) else {
        return Vec::new();
    };

    let mut names: Vec<String> = collect_uninitialized_bindings(script)
        .into_iter()
        .filter(|name| referenced.contains(name.as_str()))
        .filter(|name| summary.bindings.get(name.as_str()) == Some(BindingType::SetupLet))
        .filter(|name| !already_shadowed(name.as_str()))
        .collect();
    names.sort_unstable();
    names.dedup();
    names
}

/// Captures the declared type before template scope shadows the name.
pub(super) fn emit_type_captures(
    ts: &mut String,
    names: &[String],
) -> FxHashMap<String, Range<usize>> {
    let mut captures = FxHashMap::default();
    if names.is_empty() {
        return captures;
    }
    ts.push_str("  // Deferred-assignment type captures (setup runs before render)\n");
    for name in names {
        let line = cstr!("  type __D_{name} = typeof {name};\n");
        let start = ts.len()
            + line
                .rfind(name.as_str())
                .expect("deferred capture line should contain binding name");
        ts.push_str(line.as_str());
        captures.insert(name.clone(), start..start + name.len());
    }
    captures
}

/// Redeclares the bindings in template scope carrying their captured types.
pub(super) fn emit_template_variables(
    ts: &mut String,
    names: &[String],
    captures: &FxHashMap<String, Range<usize>>,
    semantic_links: &mut Vec<VizeSemanticLink>,
) {
    if names.is_empty() {
        return;
    }
    ts.push_str("    // Vue completes setup before the render function reads these\n");
    for name in names {
        let line = cstr!("    var {name}: __D_{name} = undefined as any;\n");
        let start = ts.len()
            + line
                .find(name.as_str())
                .expect("deferred shadow line should contain binding name");
        ts.push_str(line.as_str());
        if let Some(source_range) = captures.get(name) {
            semantic_links.push(VizeSemanticLink {
                source_range: source_range.clone(),
                target_range: start..start + name.len(),
                kind: VizeSemanticLinkKind::VueSetupTemplateRefUnwrap,
            });
        }
    }
}

/// Names bound by a top-level `let`/`var` declarator that has neither an
/// initializer nor a `!` definite-assignment assertion.
///
/// Only the statement list the setup body itself emits is walked: a binding
/// declared inside a nested block or function is never in scope for the
/// template, so it can never be the one TypeScript flags.
fn collect_uninitialized_bindings(script: &str) -> Vec<String> {
    let allocator = Allocator::default();
    let parsed = parse_script(&allocator, script);
    if parsed.panicked {
        return Vec::new();
    }

    let mut names = Vec::new();
    for statement in parsed.program.body.iter() {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        if !matches!(
            declaration.kind,
            VariableDeclarationKind::Let | VariableDeclarationKind::Var
        ) {
            continue;
        }
        for declarator in declaration.declarations.iter() {
            if declarator.init.is_some() || declarator.definite {
                continue;
            }
            let BindingPattern::BindingIdentifier(identifier) = &declarator.id else {
                continue;
            };
            names.push(String::from(identifier.name.as_str()));
        }
    }
    names
}

/// Parses the setup body under whichever dialect it actually is.
///
/// `<script setup lang="tsx">` is not valid TypeScript — `<span />` parses as a
/// type assertion — while a `lang="ts"` generic arrow (`<T>(x: T) => x`) is not
/// valid TSX, so neither dialect can serve both. TS is tried first and TSX is
/// kept only when it parses strictly better; a TS parse that merely recovered
/// would otherwise drop the declarations after the error and leave the deferred
/// set incomplete, re-exposing those template reads to TS2454.
fn parse_script<'a>(allocator: &'a Allocator, script: &'a str) -> ParserReturn<'a> {
    let as_ts = Parser::new(allocator, script, SourceType::ts()).parse();
    if !as_ts.panicked && as_ts.diagnostics.is_empty() {
        return as_ts;
    }
    let as_tsx = Parser::new(allocator, script, SourceType::tsx()).parse();
    if (as_tsx.panicked, as_tsx.diagnostics.len()) < (as_ts.panicked, as_ts.diagnostics.len()) {
        as_tsx
    } else {
        as_ts
    }
}
