//! Directly declared names of an original JS, TS, JSX or TSX Program.
//!
//! This demand reads statement heads and binding patterns. The containing
//! source frame, parser grammar and diagnostics belong to the L1 owner.
use oxc_ast::ast::{
    BindingIdentifier, Declaration, ExportDefaultDeclarationKind, ImportDeclarationSpecifier,
    Statement, TSModuleDeclarationName, VariableDeclarationKind,
};
use vize_l0::{SourceBlock, Span};
use vize_l1::embed::authored_program::AuthoredProgram;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarationSyntaxError {
    Panicked,
    Recovered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarationExport {
    Local,
    Named,
    TypeOnly,
    Default,
}

/// A declaration's authored form. A syntactic value import can resolve to a
/// type in another module, so resolution remains a separate project demand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptDeclarationKind {
    Import { type_only: bool },
    Variable(VariableDeclarationKind),
    Function,
    Class,
    Enum,
    TypeAlias,
    Interface,
    Namespace,
    ImportEquals { type_only: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScriptDeclaration<'a> {
    pub name: &'a str,
    /// File-absolute byte range of the original identifier, including escapes.
    pub span: Span,
    pub kind: ScriptDeclarationKind,
    pub export: DeclarationExport,
}

/// Borrow the original parser result for this block. Empty Programs have a
/// complete empty declaration list. Parser recovery remains visible to callers.
pub struct ScriptDeclarations<'p, 'a> {
    original: &'p AuthoredProgram<'a>,
}

impl<'p, 'a> ScriptDeclarations<'p, 'a> {
    pub fn new(original: &'p AuthoredProgram<'a>) -> Result<Self, DeclarationSyntaxError> {
        let parsed = original.parsed();
        if parsed.panicked {
            return Err(DeclarationSyntaxError::Panicked);
        }
        if !parsed.diagnostics.is_empty() {
            return Err(DeclarationSyntaxError::Recovered);
        }
        Ok(Self { original })
    }

    /// The original parser result and its requested grammar.
    pub fn original(&self) -> &'p AuthoredProgram<'a> {
        self.original
    }

    pub fn block(&self) -> SourceBlock<'a> {
        self.original.block()
    }

    /// Visit declaration occurrences in authored order, including repeated
    /// declarations that merge. Re-exports and names inside bodies introduce
    /// no direct declaration. A consumer owns its deduplication policy.
    pub fn visit(&self, mut report: impl FnMut(ScriptDeclaration<'a>)) {
        let start = self.block().start();
        let mut name = |id: &BindingIdentifier<'a>, kind, export| {
            report(ScriptDeclaration {
                name: id.name.as_str(),
                span: Span::new(start + id.span.start, start + id.span.end),
                kind,
                export,
            });
        };
        for statement in &self.original.parsed().program.body {
            match statement {
                Statement::ImportDeclaration(import) => {
                    for specifier in import.specifiers.iter().flatten() {
                        let (id, type_only) = match specifier {
                            ImportDeclarationSpecifier::ImportSpecifier(named) => {
                                (&named.local, named.import_kind.is_type())
                            }
                            ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => {
                                (&default.local, false)
                            }
                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace) => {
                                (&namespace.local, false)
                            }
                        };
                        name(
                            id,
                            ScriptDeclarationKind::Import {
                                type_only: import.import_kind.is_type() || type_only,
                            },
                            DeclarationExport::Local,
                        );
                    }
                }
                Statement::ExportNamedDeclaration(named) => {
                    if let Some(decl) = &named.declaration {
                        let export = if named.export_kind.is_type() {
                            DeclarationExport::TypeOnly
                        } else {
                            DeclarationExport::Named
                        };
                        visit_declaration(decl, export, &mut name);
                    }
                }
                Statement::ExportDefaultDeclaration(default) => {
                    let declared = match &default.declaration {
                        ExportDefaultDeclarationKind::FunctionDeclaration(function) => function
                            .id
                            .as_ref()
                            .map(|id| (id, ScriptDeclarationKind::Function)),
                        ExportDefaultDeclarationKind::ClassDeclaration(class) => class
                            .id
                            .as_ref()
                            .map(|id| (id, ScriptDeclarationKind::Class)),
                        ExportDefaultDeclarationKind::TSInterfaceDeclaration(interface) => {
                            Some((&interface.id, ScriptDeclarationKind::Interface))
                        }
                        _ => None,
                    };
                    if let Some((id, kind)) = declared {
                        name(id, kind, DeclarationExport::Default);
                    }
                }
                stmt => {
                    if let Some(decl) = stmt.as_declaration() {
                        visit_declaration(decl, DeclarationExport::Local, &mut name);
                    }
                }
            }
        }
    }
}

fn visit_declaration<'a>(
    decl: &Declaration<'a>,
    export: DeclarationExport,
    name: &mut impl FnMut(&BindingIdentifier<'a>, ScriptDeclarationKind, DeclarationExport),
) {
    use ScriptDeclarationKind as Kind;
    let declared = match decl {
        Declaration::VariableDeclaration(variable) => {
            for declarator in &variable.declarations {
                // OXC implements BoundNames order, skipping holes, initializer
                // expressions and property keys, and visiting each rest last.
                declarator.id.all_binding_identifiers(&mut |id| {
                    name(id, Kind::Variable(variable.kind), export);
                    true
                });
            }
            None
        }
        Declaration::FunctionDeclaration(function) => {
            function.id.as_ref().map(|id| (id, Kind::Function))
        }
        Declaration::ClassDeclaration(class) => class.id.as_ref().map(|id| (id, Kind::Class)),
        Declaration::TSEnumDeclaration(enumeration) => Some((&enumeration.id, Kind::Enum)),
        Declaration::TSTypeAliasDeclaration(alias) => Some((&alias.id, Kind::TypeAlias)),
        Declaration::TSInterfaceDeclaration(interface) => Some((&interface.id, Kind::Interface)),
        Declaration::TSModuleDeclaration(module) => match &module.id {
            TSModuleDeclarationName::Identifier(id) => Some((id, Kind::Namespace)),
            TSModuleDeclarationName::StringLiteral(_) => None,
        },
        Declaration::TSImportEqualsDeclaration(import) => Some((
            &import.id,
            Kind::ImportEquals {
                type_only: import.import_kind.is_type(),
            },
        )),
        Declaration::TSGlobalDeclaration(_) => None,
    };
    if let Some((id, kind)) = declared {
        name(id, kind, export);
    }
}

#[cfg(test)]
mod tests;

mod declaration;
pub use declaration::DeclarationStatement;

#[cfg(feature = "literal-assignments")]
pub mod assignments;
