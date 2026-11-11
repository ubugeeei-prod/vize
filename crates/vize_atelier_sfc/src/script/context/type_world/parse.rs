use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Declaration, ExportDefaultDeclarationKind, ImportDeclarationSpecifier, Statement, TSImportType,
    TSInterfaceDeclaration, TSTypeAliasDeclaration, TSTypeName, TSTypeReference,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::{Parser, ParserReturn};
use oxc_span::{GetSpan, SourceType, Span};
use vize_carton::{CompactString, FxHashSet};
use vize_croquis::script_parser::with_shared_parse;
use vize_croquis::types::world::{
    TypeDeclaration, TypeDeclarationKind, TypeExportBinding, TypeImport, TypeModule,
    TypeModuleReference,
};

pub(super) fn parse_module(
    source: &str,
    source_type: SourceType,
) -> (TypeModule, FxHashSet<CompactString>) {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    module_from_parse(source, &parsed)
}

/// [`parse_module`] of the SFC's own script, whose parse its other analyses share.
pub(super) fn parse_shared_module(
    source: &str,
    source_type: SourceType,
) -> (TypeModule, FxHashSet<CompactString>) {
    with_shared_parse(source, source_type, |parsed| {
        module_from_parse(source, parsed)
    })
}

fn module_from_parse(
    source: &str,
    parsed: &ParserReturn<'_>,
) -> (TypeModule, FxHashSet<CompactString>) {
    let mut module = TypeModule {
        complete: !parsed.panicked && parsed.diagnostics.is_empty(),
        ..TypeModule::default()
    };
    let mut refs = TypeRefs::default();
    refs.visit_program(&parsed.program);
    for specifier in &refs.imports {
        module
            .direct_imports
            .insert(specifier.clone(), target(specifier));
    }
    for statement in &parsed.program.body {
        match statement {
            Statement::TSInterfaceDeclaration(decl) => {
                add_interface(&mut module, source, decl, false)
            }
            Statement::TSTypeAliasDeclaration(decl) => add_alias(&mut module, source, decl, false),
            Statement::ImportDeclaration(decl) => {
                for specifier in decl.specifiers.iter().flatten() {
                    let (local, exported) = match specifier {
                        ImportDeclarationSpecifier::ImportSpecifier(spec) => (
                            spec.local.name.as_str(),
                            Some(spec.imported.name().as_str()),
                        ),
                        ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                            (spec.local.name.as_str(), Some("default"))
                        }
                        ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                            (spec.local.name.as_str(), None)
                        }
                    };
                    module.imports.insert(
                        CompactString::new(local),
                        TypeImport {
                            target: target(decl.source.value.as_str()),
                            exported: exported.map(CompactString::new),
                        },
                    );
                }
            }
            Statement::ExportNamedDeclaration(decl) => {
                if let Some(declaration) = &decl.declaration {
                    match declaration {
                        Declaration::TSInterfaceDeclaration(decl) => {
                            add_interface(&mut module, source, decl, true);
                        }
                        Declaration::TSTypeAliasDeclaration(decl) => {
                            add_alias(&mut module, source, decl, true);
                        }
                        _ => {}
                    }
                }
                for specifier in &decl.specifiers {
                    let local = CompactString::new(specifier.local.name().as_str());
                    let binding = decl.source.as_ref().map_or_else(
                        || TypeExportBinding::Local(local.clone()),
                        |source| TypeExportBinding::Forward {
                            target: target(source.value.as_str()),
                            exported: local.clone(),
                        },
                    );
                    module.exports.insert(
                        CompactString::new(specifier.exported.name().as_str()),
                        binding,
                    );
                }
            }
            Statement::ExportAllDeclaration(decl) => {
                let target = target(decl.source.value.as_str());
                match &decl.exported {
                    Some(exported) => {
                        module.exports.insert(
                            CompactString::new(exported.name().as_str()),
                            TypeExportBinding::Namespace(target),
                        );
                    }
                    None => module.star_exports.push(target),
                }
            }
            Statement::ExportDefaultDeclaration(decl) => {
                if let ExportDefaultDeclarationKind::TSInterfaceDeclaration(interface) =
                    &decl.declaration
                {
                    add_interface(&mut module, source, interface, false);
                    module.exports.insert(
                        CompactString::new("default"),
                        TypeExportBinding::Local(CompactString::new(interface.id.name.as_str())),
                    );
                }
            }
            _ => {}
        }
    }
    (module, refs.names)
}

fn target(specifier: &str) -> TypeModuleReference {
    TypeModuleReference {
        specifier: CompactString::new(specifier),
        module: None,
    }
}

fn text(source: &str, span: Span) -> CompactString {
    CompactString::new(
        source
            .get(span.start as usize..span.end as usize)
            .unwrap_or_default(),
    )
}

fn add_interface(
    module: &mut TypeModule,
    source: &str,
    decl: &TSInterfaceDeclaration<'_>,
    exported: bool,
) {
    add_declaration(
        module,
        decl.id.name.as_str(),
        TypeDeclaration {
            kind: TypeDeclarationKind::Interface,
            body: text(source, decl.body.span),
            extends: decl
                .extends
                .iter()
                .map(|clause| text(source, clause.span))
                .collect(),
            type_parameters: decl
                .type_parameters
                .as_ref()
                .map(|params| text(source, params.span)),
            declaration_source: text(source, decl.span),
        },
        exported,
    );
}

fn add_alias(
    module: &mut TypeModule,
    source: &str,
    decl: &TSTypeAliasDeclaration<'_>,
    exported: bool,
) {
    add_declaration(
        module,
        decl.id.name.as_str(),
        TypeDeclaration {
            kind: TypeDeclarationKind::Alias,
            body: text(source, decl.type_annotation.span()),
            extends: Vec::new(),
            type_parameters: decl
                .type_parameters
                .as_ref()
                .map(|params| text(source, params.span)),
            declaration_source: text(source, decl.span),
        },
        exported,
    );
}

fn add_declaration(
    module: &mut TypeModule,
    name: &str,
    declaration: TypeDeclaration,
    exported: bool,
) {
    let name = CompactString::new(name);
    if let Some(existing) = module.declarations.get(&name) {
        module.unsupported_declarations.insert(name.clone());
        module
            .unsupported_contracts
            .entry(name.clone())
            .or_insert_with(|| vec![existing.declaration_source.clone()])
            .push(declaration.declaration_source);
    } else {
        module.declarations.insert(name.clone(), declaration);
    }
    if exported {
        module
            .exports
            .insert(name.clone(), TypeExportBinding::Local(name));
    }
}

#[derive(Default)]
struct TypeRefs {
    names: FxHashSet<CompactString>,
    imports: FxHashSet<CompactString>,
}

impl<'a> Visit<'a> for TypeRefs {
    fn visit_ts_import_type(&mut self, import: &TSImportType<'a>) {
        self.imports
            .insert(CompactString::new(import.source.value.as_str()));
        walk::walk_ts_import_type(self, import);
    }
    fn visit_ts_type_reference(&mut self, reference: &TSTypeReference<'a>) {
        self.record_name(&reference.type_name);
        walk::walk_ts_type_reference(self, reference);
    }

    fn visit_ts_interface_declaration(&mut self, interface: &TSInterfaceDeclaration<'a>) {
        for clause in &interface.extends {
            self.record_expression(&clause.expression);
        }
        walk::walk_ts_interface_declaration(self, interface);
    }
}

impl TypeRefs {
    fn record_expression(&mut self, expression: &oxc_ast::ast::Expression<'_>) {
        match expression {
            oxc_ast::ast::Expression::Identifier(id) => {
                self.names.insert(CompactString::new(id.name.as_str()));
            }
            oxc_ast::ast::Expression::StaticMemberExpression(member) => {
                self.record_expression(&member.object)
            }
            _ => {}
        }
    }

    fn record_name(&mut self, name: &TSTypeName<'_>) {
        match name {
            TSTypeName::IdentifierReference(id) => {
                self.names.insert(CompactString::new(id.name.as_str()));
            }
            TSTypeName::QualifiedName(name) => self.record_name(&name.left),
            TSTypeName::ThisExpression(_) => {}
        }
    }
}
