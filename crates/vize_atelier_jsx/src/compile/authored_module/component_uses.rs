//! Admit converted values only in proved Vue component roles.
//!
//! A component object cannot preserve an authored callable's local aliases,
//! introspection or value escapes. Resolve actual symbols only for references
//! outside JSX tags and direct exports; ordinary shadowed helpers stay valid.

use super::{JsxDiagnostic, error, plans::Plan};
use oxc_ast::ast::{
    ExportDefaultDeclaration, ExportSpecifier, Expression, IdentifierReference, JSXElementName,
    Program,
};
use oxc_ast_visit::{Visit, walk};
use oxc_semantic::{Scoping, SemanticBuilder};
use oxc_span::Span;
use vize_l0::{FxHashSet, profiler::global_profiler};

pub(super) fn check(
    program: &Program<'_>,
    plans: &[Plan],
    scoping: &mut Option<Scoping>,
) -> Result<(), JsxDiagnostic> {
    let bindings: Vec<_> = plans
        .iter()
        .filter_map(|plan| plan.binding.as_ref())
        .collect();
    if bindings.is_empty() {
        return Ok(());
    }
    let names: FxHashSet<_> = bindings
        .iter()
        .map(|binding| binding.name.as_str())
        .collect();
    let owned: FxHashSet<_> = bindings.iter().map(|binding| binding.span).collect();
    let mut probe = Uses::new(&names, &owned, None);
    probe.visit_program(program);
    let candidates = probe.candidates;
    let mut semantic_builds = 0;
    let mut refusal = None;
    if candidates > 0 {
        if scoping.is_none() {
            *scoping = Some(
                SemanticBuilder::new()
                    .build(program)
                    .semantic
                    .into_scoping(),
            );
            semantic_builds = 1;
        }
        let mut checked = Uses::new(&names, &owned, scoping.as_ref());
        checked.visit_program(program);
        refusal = checked.refusal;
    }
    let profiler = global_profiler();
    if profiler.is_enabled() {
        profiler.record_counter_enabled("atelier.jsx.module.component_use_candidates", candidates);
        profiler.record_counter_enabled(
            "atelier.jsx.module.component_use_semantic_builds",
            semantic_builds,
        );
        profiler.record_counter_enabled(
            "atelier.jsx.module.component_use_refusals",
            u64::from(refusal.is_some()),
        );
    }
    if let Some(span) = refusal {
        return Err(error(
            span.start,
            span.end,
            "Vapor/SSR component-object bindings may be used only as JSX component tags or direct exports; local calls, callable aliases, introspection and value escapes require an explicit callable representation",
        ));
    }
    Ok(())
}

struct Uses<'p> {
    names: &'p FxHashSet<&'p str>,
    owned: &'p FxHashSet<Span>,
    scoping: Option<&'p Scoping>,
    component_role: bool,
    default_export: Option<Span>,
    candidates: u64,
    refusal: Option<Span>,
}

impl<'p> Uses<'p> {
    fn new(
        names: &'p FxHashSet<&'p str>,
        owned: &'p FxHashSet<Span>,
        scoping: Option<&'p Scoping>,
    ) -> Self {
        Self {
            names,
            owned,
            scoping,
            component_role: false,
            default_export: None,
            candidates: 0,
            refusal: None,
        }
    }
}

impl<'a> Visit<'a> for Uses<'_> {
    fn visit_jsx_element_name(&mut self, name: &JSXElementName<'a>) {
        let previous = self.component_role;
        self.component_role = true;
        walk::walk_jsx_element_name(self, name);
        self.component_role = previous;
    }

    fn visit_export_specifier(&mut self, specifier: &ExportSpecifier<'a>) {
        let previous = self.component_role;
        self.component_role = true;
        walk::walk_export_specifier(self, specifier);
        self.component_role = previous;
    }

    fn visit_export_default_declaration(&mut self, declaration: &ExportDefaultDeclaration<'a>) {
        let previous = self.default_export;
        self.default_export = declaration
            .declaration
            .as_expression()
            .and_then(|expression| {
                if let Expression::Identifier(identifier) = expression.get_inner_expression() {
                    Some(identifier.span)
                } else {
                    None
                }
            });
        walk::walk_export_default_declaration(self, declaration);
        self.default_export = previous;
    }

    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        if self.component_role
            || self.default_export == Some(identifier.span)
            || !self.names.contains(identifier.name.as_str())
        {
            return;
        }
        self.candidates += 1;
        if let Some(scoping) = self.scoping
            && identifier
                .reference_id
                .get()
                .and_then(|reference| scoping.get_reference(reference).symbol_id())
                .is_some_and(|symbol| self.owned.contains(&scoping.symbol_span(symbol)))
        {
            self.refusal.get_or_insert(identifier.span);
        }
    }
}
