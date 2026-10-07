//! Static relative runtime edges bound the component side of installation.

use oxc_ast::ast::{ImportDeclaration, ImportDeclarationSpecifier, ImportExpression};
use oxc_ast_visit::{Visit, walk};
use vize_carton::FxHashSet;

use super::super::resolve::Script;
use crate::providers::{ModuleId, ProjectSources};

pub(super) fn collect(
    project: &ProjectSources,
    module: ModuleId,
    script: &Script<'_>,
) -> Vec<ModuleId> {
    let mut finder = Imports {
        project,
        module,
        found: Vec::new(),
    };
    finder.visit_program(script.program);
    finder.found
}

pub(super) fn reachable(edges: &[Vec<ModuleId>], roots: &[ModuleId]) -> FxHashSet<ModuleId> {
    let mut pending = roots.to_vec();
    let mut seen = FxHashSet::default();
    while let Some(module) = pending.pop() {
        if !seen.insert(module) {
            continue;
        }
        if let Some(imported) = edges.get(module.index()) {
            pending.extend(imported);
        }
    }
    seen
}

struct Imports<'p> {
    project: &'p ProjectSources,
    module: ModuleId,
    found: Vec<ModuleId>,
}

impl Imports<'_> {
    fn add(&mut self, source: &str) {
        if let Some(module) = self.project.resolve(self.module, source) {
            self.found.push(module);
        }
    }
}

impl<'a> Visit<'a> for Imports<'_> {
    fn visit_import_declaration(&mut self, import: &ImportDeclaration<'a>) {
        if import.import_kind.is_type() {
            return;
        }
        let runtime = import.specifiers.as_ref().is_none_or(|specifiers| {
            specifiers.is_empty()
                || specifiers.iter().any(|specifier| {
                    !matches!(specifier,
                ImportDeclarationSpecifier::ImportSpecifier(named) if named.import_kind.is_type())
                })
        });
        if runtime {
            self.add(import.source.value.as_str());
        }
    }

    fn visit_import_expression(&mut self, import: &ImportExpression<'a>) {
        if let oxc_ast::ast::Expression::StringLiteral(source) =
            import.source.get_inner_expression()
        {
            self.add(source.value.as_str());
        }
        walk::walk_import_expression(self, import);
    }
}
