//! Original declaration statements, including patterns that bind no names.
use super::{
    DeclarationExport, ScriptDeclaration, ScriptDeclarationKind as Kind, ScriptDeclarations,
    visit_declaration,
};
use oxc_ast::ast::{Declaration, Statement};
use oxc_span::GetSpan;
use vize_l0::Span;

/// An ordinary declaration, or a declaration under a named export. The span
/// covers the complete statement. Nested declarations and re-exports do not
/// introduce rows, and a statement may have zero or several binding names.
pub struct DeclarationStatement<'p, 'a> {
    declaration: &'p Declaration<'a>,
    start: u32,
    pub span: Span,
    pub export: DeclarationExport,
}

impl<'a> DeclarationStatement<'_, 'a> {
    /// Global augmentations have no declaration name or binding kind. A quoted
    /// module augmentation has namespace form and no local binding identifier.
    pub fn kind(&self) -> Option<Kind> {
        Some(match self.declaration {
            Declaration::VariableDeclaration(d) => Kind::Variable(d.kind),
            Declaration::FunctionDeclaration(_) => Kind::Function,
            Declaration::ClassDeclaration(_) => Kind::Class,
            Declaration::TSEnumDeclaration(_) => Kind::Enum,
            Declaration::TSTypeAliasDeclaration(_) => Kind::TypeAlias,
            Declaration::TSInterfaceDeclaration(_) => Kind::Interface,
            Declaration::TSModuleDeclaration(_) => Kind::Namespace,
            Declaration::TSImportEqualsDeclaration(d) => Kind::ImportEquals {
                type_only: d.import_kind.is_type(),
            },
            Declaration::TSGlobalDeclaration(_) => return None,
        })
    }

    pub fn visit_names(&self, mut visit: impl FnMut(ScriptDeclaration<'a>)) {
        visit_declaration(self.declaration, self.export, &mut |id, kind, export| {
            visit(ScriptDeclaration {
                name: id.name.as_str(),
                span: Span::new(self.start + id.span.start, self.start + id.span.end),
                kind,
                export,
            })
        })
    }
}

impl<'a> ScriptDeclarations<'_, 'a> {
    /// Visit only direct unwrapped and named-export declarations. Readers
    /// choose which statements need a descent into their binding patterns.
    /// Default exports, import statements and export lists have distinct syntax.
    pub fn visit_declaration_statements(
        &self,
        mut visit: impl FnMut(DeclarationStatement<'_, 'a>),
    ) {
        let start = self.block().start();
        for stmt in &self.original().parsed().program.body {
            let (declaration, export) = match stmt {
                Statement::ExportNamedDeclaration(named) => {
                    let Some(declaration) = named.declaration.as_ref() else {
                        continue;
                    };
                    let export = if named.export_kind.is_type() {
                        DeclarationExport::TypeOnly
                    } else {
                        DeclarationExport::Named
                    };
                    (declaration, export)
                }
                stmt => {
                    let Some(declaration) = stmt.as_declaration() else {
                        continue;
                    };
                    (declaration, DeclarationExport::Local)
                }
            };
            let span = stmt.span();
            visit(DeclarationStatement {
                declaration,
                start,
                span: Span::new(start + span.start, start + span.end),
                export,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_ast::ast::VariableDeclarationKind;
    use oxc_span::SourceType;
    use vize_l0::{Allocator, SourceRoot};
    use vize_l1::embed::authored_program::AuthoredProgram;

    #[test]
    fn complete_statement_heads_precede_their_optional_original_names() {
        let block = "const local=1; export const {}=object; export const { key: [first,,...rest] }=value; declare module 'extra' {} export { local as outside }; export default class Hidden {}";
        let text = alloc::format!("<!--帧--><script lang='ts'>{block}</script>");
        let offset = text.find(block).unwrap();
        let frame = SourceRoot::new(&text)
            .unwrap()
            .block(&text[offset..offset + block.len()], offset as u32)
            .unwrap();
        let arena = Allocator::new();
        let syntax = AuthoredProgram::parse(&arena, frame, SourceType::ts());
        let query = ScriptDeclarations::new(&syntax).unwrap();
        let var = Some(Kind::Variable(VariableDeclarationKind::Const));
        let mut expected = [
            (
                "const local=1;",
                DeclarationExport::Local,
                var,
                &["local"][..],
            ),
            (
                "export const {}=object;",
                DeclarationExport::Named,
                var,
                &[][..],
            ),
            (
                "export const { key: [first,,...rest] }=value;",
                DeclarationExport::Named,
                var,
                &["first", "rest"][..],
            ),
            (
                "declare module 'extra' {}",
                DeclarationExport::Local,
                Some(Kind::Namespace),
                &[][..],
            ),
        ]
        .into_iter();
        query.visit_declaration_statements(|group| {
            let (source, export, kind, names) = expected.next().unwrap();
            assert_eq!(
                (group.span.slice(&text), group.export, group.kind()),
                (source, export, kind)
            );
            let mut names = names.iter().copied();
            group.visit_names(|row| {
                assert_eq!(Some(row.name), names.next());
                assert_eq!(row.span.slice(&text), row.name);
                assert_eq!(row.export, group.export);
            });
            assert_eq!(names.next(), None);
        });
        assert_eq!(expected.next(), None);
    }
}
