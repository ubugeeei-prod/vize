//! Original import-statement selection and local syntactic declaration facts.
use super::{DeclarationExport, ScriptDeclaration, ScriptDeclarationKind, ScriptDeclarations};
use oxc_ast::ast::{BindingIdentifier, ImportDeclaration, ImportDeclarationSpecifier, Statement};
use vize_l0::Span;

/// One original import, including side-effect imports with no local names.
/// Local spans retain authored spelling; type-only status is syntactic and
/// does not claim how another module resolves a value import.
pub struct ScriptImportStatement<'p, 'a> {
    declaration: &'p ImportDeclaration<'a>,
    start: u32,
    pub span: Span,
}

impl<'a> ScriptImportStatement<'_, 'a> {
    pub fn visit_names(&self, mut visit: impl FnMut(ScriptDeclaration<'a>)) {
        visit_names(self.declaration, |id, type_only| {
            visit(ScriptDeclaration {
                name: id.name.as_str(),
                span: Span::new(self.start + id.span.start, self.start + id.span.end),
                kind: ScriptDeclarationKind::Import { type_only },
                export: DeclarationExport::Local,
            });
        });
    }
}

pub(super) fn visit_names<'a>(
    declaration: &ImportDeclaration<'a>,
    mut visit: impl FnMut(&BindingIdentifier<'a>, bool),
) {
    for specifier in declaration.specifiers.iter().flatten() {
        let (id, type_only) = match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(named) => {
                (&named.local, named.import_kind.is_type())
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => (&default.local, false),
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace) => {
                (&namespace.local, false)
            }
        };
        visit(id, declaration.import_kind.is_type() || type_only);
    }
}

impl<'a> ScriptDeclarations<'_, 'a> {
    /// Visit the exact original top-level import statements in source order.
    /// Each statement is certified by this query's complete original parse.
    pub fn visit_import_statements(&self, mut visit: impl FnMut(ScriptImportStatement<'_, 'a>)) {
        let start = self.block().start();
        for statement in &self.original().parsed().program.body {
            if let Statement::ImportDeclaration(declaration) = statement {
                visit(ScriptImportStatement {
                    declaration,
                    start,
                    span: Span::new(start + declaration.span.start, start + declaration.span.end),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_span::SourceType;
    use vize_l0::{Allocator, SourceRoot};
    use vize_l1::embed::authored_program::AuthoredProgram;

    #[test]
    fn statement_selections_keep_side_effects_raw_spellings_and_type_status() {
        let script = r"import 'side'; import Default, { type Kind, value as \u006cocal } from 'value'; import type * as Types from 'types';";
        let source = alloc::format!("<!--🌱-->\r\n<script lang='ts'>{script}</script>");
        let start = source.find(script).unwrap();
        let block = SourceRoot::new(&source)
            .unwrap()
            .block(&source[start..start + script.len()], start as u32)
            .unwrap();
        let arena = Allocator::new();
        let original = AuthoredProgram::parse(&arena, block, SourceType::ts());
        let query = ScriptDeclarations::new(&original).unwrap();
        let mut rows = alloc::vec::Vec::new();
        query.visit_import_statements(|statement| {
            let text = &source[statement.span.start as usize..statement.span.end as usize];
            let mut names = alloc::vec::Vec::new();
            statement.visit_names(|name| {
                names.push((
                    name.name,
                    &source[name.span.start as usize..name.span.end as usize],
                    name.kind,
                ))
            });
            rows.push((text, names));
        });
        use ScriptDeclarationKind::Import;
        assert_eq!(
            rows,
            [
                ("import 'side';", alloc::vec![]),
                (
                    r"import Default, { type Kind, value as \u006cocal } from 'value';",
                    alloc::vec![
                        ("Default", "Default", Import { type_only: false }),
                        ("Kind", "Kind", Import { type_only: true }),
                        ("local", r"\u006cocal", Import { type_only: false }),
                    ]
                ),
                (
                    "import type * as Types from 'types';",
                    alloc::vec![("Types", "Types", Import { type_only: true })]
                ),
            ]
        );
    }
}
