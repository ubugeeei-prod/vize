//! Bindings each script program declares, in the shape the two rules report.
//!
//! Function expression names, setter parameters, parameters of body-less
//! signatures, and anything inside `declare module`, `declare namespace`, or
//! `declare global` are not recorded, as typescript-eslint skips them too.

use oxc_ast::ast::{
    ArrowFunctionExpression, BindingIdentifier, BindingPattern, CatchClause, Class, ClassElement,
    Expression, ForInStatement, ForOfStatement, ForStatementLeft, FormalParameters, Function,
    ImportDeclaration, ImportDeclarationSpecifier, Statement, TSEnumDeclaration,
    TSGlobalDeclaration, TSImportEqualsDeclaration, TSInterfaceDeclaration, TSModuleDeclaration,
    TSModuleDeclarationBody, TSModuleDeclarationName, TSTypeAliasDeclaration,
    TSTypeParameterDeclaration, VariableDeclarationKind, VariableDeclarator,
};
use oxc_ast_visit::{Visit, walk};
use oxc_ecmascript::BoundNames;
use oxc_span::{GetSpan, Span};
use oxc_syntax::{scope::ScopeFlags, symbol::SymbolId};
use vize_l0::{FxHashMap, FxHashSet};

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
    /// The binding written by the last initializing declaration.
    pub(super) init_span: Option<Span>,
    pub(super) array_element: bool,
    pub(super) exported: bool,
    pub(super) group: u32,
    /// Declarations a reference inside of does not count as a use.
    pub(super) self_spans: Vec<Span>,
    /// `for (var name in obj) return;` counts the loop variable as used.
    pub(super) for_return: bool,
    /// A class with a `static {}` block.
    pub(super) static_block: bool,
}

/// One parameter position: its bindings and whether it is a plain identifier.
pub(super) struct ParamSlot {
    pub(super) ids: Vec<SymbolId>,
    pub(super) plain: bool,
}

pub(super) struct RestGroup {
    pub(super) rest: Vec<SymbolId>,
    pub(super) siblings: Vec<SymbolId>,
}

pub(super) struct Classified {
    pub(super) infos: FxHashMap<SymbolId, BindingInfo>,
    pub(super) functions: Vec<Vec<ParamSlot>>,
    pub(super) rests: Vec<RestGroup>,
    /// Identifier spans (start) of `for (name in obj) return;` loop targets.
    pub(super) for_return_targets: FxHashSet<u32>,
}

pub(super) fn classify(program: &oxc_ast::ast::Program<'_>) -> Classified {
    let mut classifier = Classifier::default();
    classifier.visit_program(program);
    super::exports::mark_exports(program, &mut classifier.infos);
    Classified {
        infos: classifier.infos,
        functions: classifier.functions,
        rests: classifier.rests,
        for_return_targets: classifier.for_return_targets,
    }
}

#[derive(Default)]
struct Classifier {
    infos: FxHashMap<SymbolId, BindingInfo>,
    functions: Vec<Vec<ParamSlot>>,
    rests: Vec<RestGroup>,
    for_return_targets: FxHashSet<u32>,
    group: u32,
    ambient: u32,
    dotted: Vec<Span>,
}

struct Note {
    kind: Kind,
    has_init: bool,
    array_element: bool,
    group: u32,
    self_span: Option<Span>,
}

impl Note {
    fn plain(kind: Kind) -> Self {
        Self {
            kind,
            has_init: false,
            array_element: false,
            group: 0,
            self_span: None,
        }
    }

    fn owning(kind: Kind, span: Span) -> Self {
        Self {
            self_span: Some(span),
            ..Self::plain(kind)
        }
    }
}

impl Classifier {
    fn note(&mut self, id: &BindingIdentifier<'_>, note: Note) -> Option<SymbolId> {
        if self.ambient > 0 {
            return None;
        }
        let symbol_id = id.symbol_id.get()?;
        let init_span = note.has_init.then_some(id.span);
        let info = self.infos.entry(symbol_id).or_insert_with(|| BindingInfo {
            kind: note.kind,
            has_init: false,
            init_span: None,
            array_element: false,
            exported: false,
            group: note.group,
            self_spans: Vec::new(),
            for_return: false,
            static_block: false,
        });
        info.kind = note.kind;
        info.has_init |= note.has_init;
        if init_span.is_some() {
            info.init_span = init_span;
        }
        info.array_element |= note.array_element;
        info.group = note.group;
        if let Some(span) = note.self_span {
            info.self_spans.push(span);
        }
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
            BindingPattern::BindingIdentifier(id) => {
                let note = Note {
                    kind,
                    has_init,
                    array_element,
                    group,
                    self_span: None,
                };
                self.note(id, note).into_iter().collect()
            }
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
                self.push_rest(&mut siblings, rest);
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
                self.push_rest(&mut siblings, rest);
                siblings
            }
        }
    }

    fn push_rest(&mut self, siblings: &mut Vec<SymbolId>, rest: Vec<SymbolId>) {
        if !rest.is_empty() {
            self.rests.push(RestGroup {
                rest: rest.clone(),
                siblings: siblings.clone(),
            });
        }
        siblings.extend(rest);
    }

    fn note_params(&mut self, params: &FormalParameters<'_>) {
        let mut slots = Vec::new();
        for item in &params.items {
            // A TypeScript parameter property is a class member, not a local.
            if item.accessibility.is_some() || item.readonly || item.r#override {
                continue;
            }
            let plain = matches!(item.pattern, BindingPattern::BindingIdentifier(_))
                && item.initializer.is_none();
            let ids = self.record_pattern(
                &item.pattern,
                Kind::Param,
                item.initializer.is_some(),
                false,
                0,
            );
            slots.push(ParamSlot { ids, plain });
        }
        if let Some(rest) = &params.rest {
            let ids = self.record_pattern(&rest.rest.argument, Kind::Param, false, false, 0);
            slots.push(ParamSlot { ids, plain: false });
        }
        if slots.iter().any(|slot| !slot.ids.is_empty()) {
            self.functions.push(slots);
        }
    }

    fn note_type_params(&mut self, params: &TSTypeParameterDeclaration<'_>) {
        for param in &params.params {
            self.note(&param.name, Note::plain(Kind::Type));
        }
    }

    fn note_for_return(&mut self, left: &ForStatementLeft<'_>, body: &Statement<'_>) {
        let first = match body {
            Statement::BlockStatement(block) => block.body.first(),
            other => Some(other),
        };
        if !matches!(first, Some(Statement::ReturnStatement(_))) {
            return;
        }
        match left {
            ForStatementLeft::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    if let BindingPattern::BindingIdentifier(id) = &declarator.id
                        && let Some(symbol) = id.symbol_id.get()
                        && let Some(info) = self.infos.get_mut(&symbol)
                    {
                        info.for_return = true;
                    }
                }
            }
            ForStatementLeft::AssignmentTargetIdentifier(id) => {
                self.for_return_targets.insert(id.span.start);
            }
            _ => {}
        }
    }

    fn mark_loop_declarations(&mut self, left: &ForStatementLeft<'_>) {
        let ForStatementLeft::VariableDeclaration(declaration) = left else {
            return;
        };
        for declarator in &declaration.declarations {
            declarator.id.bound_names(&mut |id| {
                if let Some(symbol) = id.symbol_id.get()
                    && let Some(info) = self.infos.get_mut(&symbol)
                {
                    info.has_init = true;
                    info.init_span = Some(id.span);
                }
            });
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
        let ids = self.record_pattern(
            &declarator.id,
            kind,
            declarator.init.is_some(),
            false,
            self.group,
        );
        // `const a = () => a()` reads `a` only from inside its own value.
        if let (BindingPattern::BindingIdentifier(_), Some(init)) =
            (&declarator.id, &declarator.init)
            && matches!(
                init.without_parentheses(),
                Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_)
            )
        {
            for id in ids {
                if let Some(info) = self.infos.get_mut(&id) {
                    info.self_spans.push(init.span());
                }
            }
        }
        walk::walk_variable_declarator(self, declarator);
    }

    fn visit_for_in_statement(&mut self, statement: &ForInStatement<'a>) {
        walk::walk_for_in_statement(self, statement);
        self.mark_loop_declarations(&statement.left);
        self.note_for_return(&statement.left, &statement.body);
    }

    fn visit_for_of_statement(&mut self, statement: &ForOfStatement<'a>) {
        walk::walk_for_of_statement(self, statement);
        self.mark_loop_declarations(&statement.left);
        self.note_for_return(&statement.left, &statement.body);
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        if function.is_declaration()
            && let Some(id) = &function.id
        {
            self.note(id, Note::owning(Kind::Function, function.span));
        }
        if let Some(params) = &function.type_parameters {
            self.note_type_params(params);
        }
        if function.body.is_some() && !function.declare && !flags.is_set_accessor() {
            self.note_params(&function.params);
        }
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
        if let Some(id) = &class.id
            && let Some(symbol) = self.note(id, Note::plain(Kind::Class))
            && class
                .body
                .body
                .iter()
                .any(|element| matches!(element, ClassElement::StaticBlock(_)))
            && let Some(info) = self.infos.get_mut(&symbol)
        {
            info.static_block = true;
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
                    self.note(&specifier.local, Note::plain(kind));
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                    let kind = if type_only {
                        Kind::TypeImport
                    } else {
                        Kind::Import
                    };
                    self.note(&specifier.local, Note::plain(kind));
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                    self.note(&specifier.local, Note::plain(Kind::Import));
                }
            }
        }
    }

    fn visit_ts_type_alias_declaration(&mut self, declaration: &TSTypeAliasDeclaration<'a>) {
        self.note(&declaration.id, Note::owning(Kind::Type, declaration.span));
        if let Some(params) = &declaration.type_parameters {
            self.note_type_params(params);
        }
        walk::walk_ts_type_alias_declaration(self, declaration);
    }

    fn visit_ts_interface_declaration(&mut self, declaration: &TSInterfaceDeclaration<'a>) {
        self.note(&declaration.id, Note::owning(Kind::Type, declaration.span));
        if let Some(params) = &declaration.type_parameters {
            self.note_type_params(params);
        }
        walk::walk_ts_interface_declaration(self, declaration);
    }

    fn visit_ts_enum_declaration(&mut self, declaration: &TSEnumDeclaration<'a>) {
        self.note(&declaration.id, Note::owning(Kind::Type, declaration.span));
        walk::walk_ts_enum_declaration(self, declaration);
    }

    fn visit_ts_module_declaration(&mut self, declaration: &TSModuleDeclaration<'a>) {
        let dotted = self.dotted.contains(&declaration.span);
        if let TSModuleDeclarationName::Identifier(id) = &declaration.id
            && !dotted
        {
            self.note(id, Note::owning(Kind::Type, declaration.span));
        }
        if let Some(TSModuleDeclarationBody::TSModuleDeclaration(inner)) = &declaration.body {
            self.dotted.push(inner.span);
        }
        let ambient = declaration.declare
            || matches!(declaration.id, TSModuleDeclarationName::StringLiteral(_));
        if ambient {
            self.ambient += 1;
        }
        walk::walk_ts_module_declaration(self, declaration);
        if ambient {
            self.ambient -= 1;
        }
    }

    fn visit_ts_global_declaration(&mut self, declaration: &TSGlobalDeclaration<'a>) {
        self.ambient += 1;
        walk::walk_ts_global_declaration(self, declaration);
        self.ambient -= 1;
    }

    fn visit_ts_import_equals_declaration(&mut self, declaration: &TSImportEqualsDeclaration<'a>) {
        let kind = if declaration.import_kind.is_type() {
            Kind::TypeImport
        } else {
            Kind::Import
        };
        self.note(&declaration.id, Note::plain(kind));
        walk::walk_ts_import_equals_declaration(self, declaration);
    }
}
