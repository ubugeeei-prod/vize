use oxc_ast::ast::{
    ExportAllDeclaration, ExportDefaultDeclaration, ExportNamedDeclaration, Expression,
    IdentifierReference, ImportDeclaration, ImportDeclarationSpecifier, ImportExpression, Program,
};
use oxc_ast_visit::{Visit, walk};
use oxc_semantic::Semantic;

use crate::{
    ModuleDeclaration, ModuleExport, ModuleImport, ModuleImportBinding, ModuleImportBindingKind,
    ModuleReference, ModuleSpan,
};

pub(crate) fn collect(
    program: &Program<'_>,
    semantic: &Semantic<'_>,
    base: u32,
) -> (
    Vec<ModuleImport>,
    Vec<ModuleExport>,
    Vec<ModuleDeclaration>,
    Vec<ModuleReference>,
) {
    let mut modules = ModuleCollector::new(base);
    modules.visit_program(program);
    let scoping = semantic.scoping();
    let declarations = scoping
        .symbol_ids()
        .map(|id| ModuleDeclaration {
            name: scoping.symbol_name(id).into(),
            span: absolute(scoping.symbol_span(id), base),
        })
        .collect::<Vec<_>>();
    let references = semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let oxc_ast::AstKind::IdentifierReference(identifier) = node.kind() else {
                return None;
            };
            reference_fact(identifier, semantic, base)
        })
        .collect();
    (modules.imports, modules.exports, declarations, references)
}

fn reference_fact(
    identifier: &IdentifierReference<'_>,
    semantic: &Semantic<'_>,
    base: u32,
) -> Option<ModuleReference> {
    let reference = semantic
        .scoping()
        .get_reference(identifier.reference_id.get()?);
    Some(ModuleReference {
        name: identifier.name.as_str().into(),
        span: absolute(identifier.span, base),
        resolved_declaration: reference.symbol_id().map(|id| id.index()),
        read: reference.is_read(),
        write: reference.is_write(),
        type_only: reference.flags().is_type(),
    })
}

struct ModuleCollector {
    base: u32,
    imports: Vec<ModuleImport>,
    exports: Vec<ModuleExport>,
}

impl ModuleCollector {
    fn new(base: u32) -> Self {
        Self {
            base,
            imports: Vec::new(),
            exports: Vec::new(),
        }
    }
}

impl<'a> Visit<'a> for ModuleCollector {
    fn visit_import_declaration(&mut self, import: &ImportDeclaration<'a>) {
        self.imports.push(ModuleImport {
            specifier: import.source.value.as_str().into(),
            locals: runtime_import_locals(import),
            bindings: import_bindings(import),
            dynamic: false,
            type_only: import.import_kind.is_type(),
            span: absolute(import.span, self.base),
        });
        walk::walk_import_declaration(self, import);
    }

    fn visit_import_expression(&mut self, import: &ImportExpression<'a>) {
        if let Expression::StringLiteral(source) = &import.source {
            self.imports.push(ModuleImport {
                specifier: source.value.as_str().into(),
                locals: Vec::new(),
                bindings: Vec::new(),
                dynamic: true,
                type_only: false,
                span: absolute(import.span, self.base),
            });
        }
        walk::walk_import_expression(self, import);
    }

    fn visit_export_named_declaration(&mut self, export: &ExportNamedDeclaration<'a>) {
        self.exports.push(ModuleExport {
            source: export
                .source
                .as_ref()
                .map(|source| source.value.as_str().into()),
            names: export
                .specifiers
                .iter()
                .map(|specifier| specifier.exported.name().as_str().into())
                .collect(),
            default: false,
            type_only: export.export_kind.is_type(),
            span: absolute(export.span, self.base),
        });
        walk::walk_export_named_declaration(self, export);
    }

    fn visit_export_default_declaration(&mut self, export: &ExportDefaultDeclaration<'a>) {
        self.exports.push(ModuleExport {
            source: None,
            names: vec!["default".into()],
            default: true,
            type_only: false,
            span: absolute(export.span, self.base),
        });
        walk::walk_export_default_declaration(self, export);
    }

    fn visit_export_all_declaration(&mut self, export: &ExportAllDeclaration<'a>) {
        self.exports.push(ModuleExport {
            source: Some(export.source.value.as_str().into()),
            names: vec!["*".into()],
            default: false,
            type_only: export.export_kind.is_type(),
            span: absolute(export.span, self.base),
        });
        walk::walk_export_all_declaration(self, export);
    }
}

fn runtime_import_locals(import: &ImportDeclaration<'_>) -> Vec<Box<str>> {
    import
        .specifiers
        .iter()
        .flatten()
        .filter_map(|specifier| match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(specifier)
                if !specifier.import_kind.is_type() =>
            {
                Some(specifier.local.name.as_str().into())
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                Some(specifier.local.name.as_str().into())
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                Some(specifier.local.name.as_str().into())
            }
            ImportDeclarationSpecifier::ImportSpecifier(_) => None,
        })
        .collect()
}

fn import_bindings(import: &ImportDeclaration<'_>) -> Vec<ModuleImportBinding> {
    import
        .specifiers
        .iter()
        .flatten()
        .map(|specifier| match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(specifier) => ModuleImportBinding {
                imported: Some(specifier.imported.name().as_str().into()),
                local: specifier.local.name.as_str().into(),
                kind: ModuleImportBindingKind::Named,
                type_only: import.import_kind.is_type() || specifier.import_kind.is_type(),
            },
            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => ModuleImportBinding {
                imported: Some("default".into()),
                local: specifier.local.name.as_str().into(),
                kind: ModuleImportBindingKind::Default,
                type_only: import.import_kind.is_type(),
            },
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                ModuleImportBinding {
                    imported: None,
                    local: specifier.local.name.as_str().into(),
                    kind: ModuleImportBindingKind::Namespace,
                    type_only: import.import_kind.is_type(),
                }
            }
        })
        .collect()
}

pub(crate) const fn absolute(span: oxc_span::Span, base: u32) -> ModuleSpan {
    ModuleSpan::new(
        base.saturating_add(span.start),
        base.saturating_add(span.end),
    )
}
