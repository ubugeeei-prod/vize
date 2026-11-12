//! Bindings each script program declares, in the shape the two rules report.

use oxc_ast::ast::{
    ArrowFunctionExpression, BindingIdentifier, BindingPattern, CatchClause, Class,
    FormalParameters, Function, ImportDeclaration, ImportDeclarationSpecifier, TSEnumDeclaration,
    TSImportEqualsDeclaration, TSInterfaceDeclaration, TSTypeAliasDeclaration,
    TSTypeParameterDeclaration, VariableDeclarationKind, VariableDeclarator,
};
use oxc_ast_visit::{Visit, walk};
use oxc_syntax::{scope::ScopeFlags, symbol::SymbolId};
use vize_l0::FxHashMap;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Import,
    TypeImport,
    Function,
    Class,
    Type,
    Let,
    Const,
    Var,
    Param,
    Catch,
}

pub(super) struct BindingInfo {
    pub(super) kind: Kind,
    pub(super) has_init: bool,
    pub(super) array_element: bool,
    pub(super) exported: bool,
    pub(super) group: u32,
}

pub(super) struct RestGroup {
    pub(super) rest: Vec<SymbolId>,
    pub(super) siblings: Vec<SymbolId>,
}

pub(super) struct Classified {
    pub(super) infos: FxHashMap<SymbolId, BindingInfo>,
    pub(super) functions: Vec<Vec<Vec<SymbolId>>>,
    pub(super) rests: Vec<RestGroup>,
}

pub(super) fn classify(program: &oxc_ast::ast::Program<'_>) -> Classified {
    let mut classifier = Classifier::default();
    classifier.visit_program(program);
    super::exports::mark_exports(program, &mut classifier.infos);
    Classified {
        infos: classifier.infos,
        functions: classifier.functions,
        rests: classifier.rests,
    }
}

#[derive(Default)]
struct Classifier {
    infos: FxHashMap<SymbolId, BindingInfo>,
    functions: Vec<Vec<Vec<SymbolId>>>,
    rests: Vec<RestGroup>,
    group: u32,
}

impl Classifier {
    fn note(
        &mut self,
        id: &BindingIdentifier<'_>,
        kind: Kind,
        has_init: bool,
        array_element: bool,
        group: u32,
    ) -> Option<SymbolId> {
        let symbol_id = id.symbol_id.get()?;
        self.infos.insert(
            symbol_id,
            BindingInfo {
                kind,
                has_init,
                array_element,
                exported: false,
                group,
            },
        );
        Some(symbol_id)
    }

    fn record_pattern(
        &mut self,
        pattern: &BindingPattern<'_>,
        kind: Kind,
        has_init: bool,
        array_element: bool,
        group: u32,
    ) -> Vec<SymbolId> {
        match pattern {
            BindingPattern::BindingIdentifier(id) => self
                .note(id, kind, has_init, array_element, group)
                .into_iter()
                .collect(),
            BindingPattern::AssignmentPattern(assignment) => {
                self.record_pattern(&assignment.left, kind, true, array_element, group)
            }
            BindingPattern::ObjectPattern(object) => {
                let mut siblings = Vec::new();
                for property in &object.properties {
                    siblings.extend(self.record_pattern(
                        &property.value,
                        kind,
                        has_init,
                        false,
                        group,
                    ));
                }
                let rest = object
                    .rest
                    .as_ref()
                    .map(|rest| self.record_pattern(&rest.argument, kind, has_init, false, group))
                    .unwrap_or_default();
                if !rest.is_empty() {
                    self.rests.push(RestGroup {
                        rest: rest.clone(),
                        siblings: siblings.clone(),
                    });
                }
                siblings.extend(rest);
                siblings
            }
            BindingPattern::ArrayPattern(array) => {
                let mut siblings = Vec::new();
                for element in array.elements.iter().flatten() {
                    siblings.extend(self.record_pattern(element, kind, has_init, true, group));
                }
                let rest = array
                    .rest
                    .as_ref()
                    .map(|rest| self.record_pattern(&rest.argument, kind, has_init, false, group))
                    .unwrap_or_default();
                if !rest.is_empty() {
                    self.rests.push(RestGroup {
                        rest: rest.clone(),
                        siblings: siblings.clone(),
                    });
                }
                siblings.extend(rest);
                siblings
            }
        }
    }

    fn note_params(&mut self, params: &FormalParameters<'_>) {
        let mut slots = Vec::new();
        for item in &params.items {
            slots.push(self.record_pattern(
                &item.pattern,
                Kind::Param,
                item.initializer.is_some(),
                false,
                0,
            ));
        }
        if let Some(rest) = &params.rest {
            slots.push(self.record_pattern(&rest.rest.argument, Kind::Param, false, false, 0));
        }
        if slots.iter().any(|slot| !slot.is_empty()) {
            self.functions.push(slots);
        }
    }

    fn note_type_params(&mut self, params: &TSTypeParameterDeclaration<'_>) {
        for param in &params.params {
            self.note(&param.name, Kind::Type, false, false, 0);
        }
    }
}

impl<'a> Visit<'a> for Classifier {
    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        let kind = match declarator.kind {
            VariableDeclarationKind::Let => Kind::Let,
            VariableDeclarationKind::Var => Kind::Var,
            VariableDeclarationKind::Const
            | VariableDeclarationKind::Using
            | VariableDeclarationKind::AwaitUsing => Kind::Const,
        };
        self.group += 1;
        self.record_pattern(
            &declarator.id,
            kind,
            declarator.init.is_some(),
            false,
            self.group,
        );
        walk::walk_variable_declarator(self, declarator);
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        if let Some(id) = &function.id {
            self.note(id, Kind::Function, false, false, 0);
        }
        if let Some(params) = &function.type_parameters {
            self.note_type_params(params);
        }
        self.note_params(&function.params);
        walk::walk_function(self, function, flags);
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        if let Some(params) = &arrow.type_parameters {
            self.note_type_params(params);
        }
        self.note_params(&arrow.params);
        walk::walk_arrow_function_expression(self, arrow);
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        if let Some(id) = &class.id {
            self.note(id, Kind::Class, false, false, 0);
        }
        if let Some(params) = &class.type_parameters {
            self.note_type_params(params);
        }
        walk::walk_class(self, class);
    }

    fn visit_catch_clause(&mut self, clause: &CatchClause<'a>) {
        if let Some(param) = &clause.param {
            self.record_pattern(&param.pattern, Kind::Catch, false, false, 0);
        }
        walk::walk_catch_clause(self, clause);
    }

    fn visit_import_declaration(&mut self, import: &ImportDeclaration<'a>) {
        let type_only = import.import_kind.is_type();
        let Some(specifiers) = &import.specifiers else {
            return;
        };
        for specifier in specifiers {
            match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                    let kind = if type_only || specifier.import_kind.is_type() {
                        Kind::TypeImport
                    } else {
                        Kind::Import
                    };
                    self.note(&specifier.local, kind, false, false, 0);
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                    let kind = if type_only {
                        Kind::TypeImport
                    } else {
                        Kind::Import
                    };
                    self.note(&specifier.local, kind, false, false, 0);
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                    self.note(&specifier.local, Kind::Import, false, false, 0);
                }
            }
        }
    }

    fn visit_ts_type_alias_declaration(&mut self, declaration: &TSTypeAliasDeclaration<'a>) {
        self.note(&declaration.id, Kind::Type, false, false, 0);
        if let Some(params) = &declaration.type_parameters {
            self.note_type_params(params);
        }
        walk::walk_ts_type_alias_declaration(self, declaration);
    }

    fn visit_ts_interface_declaration(&mut self, declaration: &TSInterfaceDeclaration<'a>) {
        self.note(&declaration.id, Kind::Type, false, false, 0);
        if let Some(params) = &declaration.type_parameters {
            self.note_type_params(params);
        }
        walk::walk_ts_interface_declaration(self, declaration);
    }

    fn visit_ts_enum_declaration(&mut self, declaration: &TSEnumDeclaration<'a>) {
        self.note(&declaration.id, Kind::Type, false, false, 0);
        walk::walk_ts_enum_declaration(self, declaration);
    }

    fn visit_ts_import_equals_declaration(&mut self, declaration: &TSImportEqualsDeclaration<'a>) {
        let kind = if declaration.import_kind.is_type() {
            Kind::TypeImport
        } else {
            Kind::Import
        };
        self.note(&declaration.id, kind, false, false, 0);
        walk::walk_ts_import_equals_declaration(self, declaration);
    }
}
