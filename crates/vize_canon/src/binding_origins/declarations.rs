//! Borrow declaration syntax while lowering only requested dependency facts.
use super::{LocalSymbol, Namespace, collect::Collector};
use oxc_ast::ast::{
    BindingPattern, Declaration, Function, ImportDeclaration, ImportDeclarationSpecifier,
    ImportSpecifier, Statement, TSTypeAliasDeclaration, VariableDeclaration, VariableDeclarator,
};
use oxc_syntax::symbol::SymbolId;
use vize_carton::FxHashMap;

#[derive(Clone, Copy)]
pub(super) enum Syntax<'s, 'a> {
    Opaque,
    Import {
        declaration: &'s ImportDeclaration<'a>,
        specifier: Option<&'s ImportSpecifier<'a>>,
    },
    TypeAlias(&'s TSTypeAliasDeclaration<'a>),
    Variable {
        declaration: &'s VariableDeclarator<'a>,
        ambient: bool,
    },
}
pub(super) enum SyntaxDeclarations<'s, 'a> {
    One(Syntax<'s, 'a>),
    Many(Vec<Syntax<'s, 'a>>),
}
impl<'s, 'a> SyntaxDeclarations<'s, 'a> {
    fn push(&mut self, syntax: Syntax<'s, 'a>) {
        match self {
            Self::One(first) => *self = Self::Many(vec![*first, syntax]),
            Self::Many(declarations) => declarations.push(syntax),
        }
    }
}
#[derive(Default)]
pub(super) struct DeclarationIndex<'s, 'a> {
    pub(super) bindings: FxHashMap<LocalSymbol, SyntaxDeclarations<'s, 'a>>,
    pub(super) namespaces: FxHashMap<SymbolId, &'s ImportDeclaration<'a>>,
}
impl<'s, 'a> Collector<'s, 'a> {
    pub(super) fn binding(
        &mut self,
        name: &str,
        symbol: Option<SymbolId>,
        namespace: Namespace,
        syntax: Syntax<'s, 'a>,
        global: bool,
    ) {
        if let Some(symbol) = symbol {
            let local = LocalSymbol { symbol, namespace };
            self.declarations
                .bindings
                .entry(local)
                .and_modify(|declarations| declarations.push(syntax))
                .or_insert(SyntaxDeclarations::One(syntax));
            if global {
                self.note_global(name, local);
            }
        }
    }
    fn variables(&mut self, declaration: &'s VariableDeclaration<'a>, global: bool, ambient: bool) {
        for declarator in &declaration.declarations {
            if let BindingPattern::BindingIdentifier(id) = &declarator.id {
                self.binding(
                    id.name.as_str(),
                    id.symbol_id.get(),
                    Namespace::Value,
                    Syntax::Variable {
                        declaration: declarator,
                        ambient: ambient || declaration.declare,
                    },
                    global,
                );
            }
        }
    }
    fn function(&mut self, function: &'s Function<'a>, global: bool) {
        if let Some(id) = &function.id {
            self.binding(
                id.name.as_str(),
                id.symbol_id.get(),
                Namespace::Value,
                Syntax::Opaque,
                global,
            );
        }
    }
    fn type_alias(&mut self, alias: &'s TSTypeAliasDeclaration<'a>, global: bool) {
        self.binding(
            alias.id.name.as_str(),
            alias.id.symbol_id.get(),
            Namespace::Type,
            Syntax::TypeAlias(alias),
            global,
        );
    }
    fn import(&mut self, declaration: &'s ImportDeclaration<'a>) {
        if let Some(specifiers) = &declaration.specifiers {
            for specifier in specifiers {
                match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                        for namespace in [Namespace::Value, Namespace::Type] {
                            self.binding(
                                specifier.local.name.as_str(),
                                specifier.local.symbol_id.get(),
                                namespace,
                                Syntax::Import {
                                    declaration,
                                    specifier: Some(specifier),
                                },
                                false,
                            );
                        }
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                        for namespace in [Namespace::Value, Namespace::Type] {
                            self.binding(
                                specifier.local.name.as_str(),
                                specifier.local.symbol_id.get(),
                                namespace,
                                Syntax::Import {
                                    declaration,
                                    specifier: None,
                                },
                                false,
                            );
                        }
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        if let Some(symbol) = specifier.local.symbol_id.get() {
                            self.declarations.namespaces.insert(symbol, declaration);
                        }
                    }
                }
            }
        }
    }
    fn declaration(&mut self, declaration: &'s Declaration<'a>, global: bool, ambient: bool) {
        match declaration {
            Declaration::TSGlobalDeclaration(declaration) => {
                for statement in &declaration.body.body {
                    self.statement(statement, true, true);
                }
                if let Some(scope) = declaration.scope_id.get() {
                    self.global_scope(scope);
                } else {
                    self.missing_semantic_scope = true;
                }
            }
            Declaration::VariableDeclaration(declaration) => {
                self.variables(declaration, global, ambient)
            }
            Declaration::FunctionDeclaration(function) => self.function(function, global),
            Declaration::TSTypeAliasDeclaration(alias) => self.type_alias(alias, global),
            _ => {}
        }
    }
    pub(super) fn statement(&mut self, statement: &'s Statement<'a>, global: bool, ambient: bool) {
        match statement {
            Statement::TSGlobalDeclaration(declaration) => {
                for statement in &declaration.body.body {
                    self.statement(statement, true, true);
                }
                if let Some(scope) = declaration.scope_id.get() {
                    self.global_scope(scope);
                } else {
                    self.missing_semantic_scope = true;
                }
            }
            Statement::VariableDeclaration(declaration) => {
                self.variables(declaration, global, ambient)
            }
            Statement::FunctionDeclaration(function) => self.function(function, global),
            Statement::TSTypeAliasDeclaration(alias) => self.type_alias(alias, global),
            Statement::ImportDeclaration(declaration) => self.import(declaration),
            Statement::ExportNamedDeclaration(export) => {
                if let Some(declaration) = &export.declaration {
                    self.declaration(declaration, global, ambient);
                }
            }
            _ => {}
        }
    }
}
