//! Module, reference, and compiler-macro facts.

#[path = "macros/boolean_keys.rs"]
mod boolean_keys;

use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, Declaration, Expression, IdentifierReference,
    Statement, TSEnumDeclaration, TSTypeName, TSTypeQueryExprName, TSTypeReference,
    VariableDeclarator,
};
use oxc_ast_visit::{Visit, walk};
use vize_carton::{FxHashSet, String};

#[derive(Default)]
pub(super) struct IdentifierUsage {
    pub(super) type_references: FxHashSet<String>,
    pub(super) value_references: FxHashSet<String>,
    type_depth: u32,
}

impl<'a> Visit<'a> for IdentifierUsage {
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        if self.type_depth == 0 {
            self.value_references.insert(ident.name.as_str().into());
        }
    }

    fn visit_ts_type_reference(&mut self, ty: &TSTypeReference<'a>) {
        record_type_name_root(&ty.type_name, &mut self.type_references);
        self.type_depth += 1;
        walk::walk_ts_type_reference(self, ty);
        self.type_depth -= 1;
    }

    fn visit_ts_type_query_expr_name(&mut self, name: &TSTypeQueryExprName<'a>) {
        record_type_query_root(name, &mut self.value_references);
        walk::walk_ts_type_query_expr_name(self, name);
    }
}

pub(super) fn identifier_usage(program: &oxc_ast::ast::Program<'_>) -> IdentifierUsage {
    let mut usage = IdentifierUsage::default();
    usage.visit_program(program);
    usage
}

pub(super) fn define_props_type_references(
    program: &oxc_ast::ast::Program<'_>,
) -> Option<FxHashSet<String>> {
    #[derive(Default)]
    struct Collector {
        references: Option<FxHashSet<String>>,
    }

    impl<'a> Visit<'a> for Collector {
        fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
            if is_define_props_call(call)
                && let Some(type_arguments) = &call.type_arguments
                && let Some(first) = type_arguments.params.first()
            {
                let mut usage = IdentifierUsage::default();
                usage.visit_ts_type(first);
                self.references
                    .get_or_insert_default()
                    .extend(usage.type_references);
            }
            walk::walk_call_expression(self, call);
        }
    }

    let mut collector = Collector::default();
    collector.visit_program(program);
    collector.references
}

pub(super) fn const_enum_names(program: &oxc_ast::ast::Program<'_>) -> FxHashSet<String> {
    #[derive(Default)]
    struct Collector {
        names: FxHashSet<String>,
    }
    impl<'a> Visit<'a> for Collector {
        fn visit_ts_enum_declaration(&mut self, decl: &TSEnumDeclaration<'a>) {
            if decl.r#const {
                self.names.insert(decl.id.name.as_str().into());
            }
        }
    }
    let mut collector = Collector::default();
    collector.visit_program(program);
    collector.names
}

pub(super) fn named_value_exports(program: &oxc_ast::ast::Program<'_>) -> Vec<String> {
    let mut seen = FxHashSet::default();
    let mut names = Vec::new();
    for statement in &program.body {
        let Statement::ExportNamedDeclaration(export) = statement else {
            continue;
        };
        if export.source.is_some() || export.export_kind.is_type() {
            continue;
        }
        if let Some(declaration) = export.declaration.as_ref() {
            collect_declaration_exports(declaration, &mut seen, &mut names);
        }
    }
    names
}

pub(super) fn module_statement_spans(program: &oxc_ast::ast::Program<'_>) -> Vec<(u32, u32)> {
    program
        .body
        .iter()
        .filter_map(|statement| match statement {
            Statement::ImportDeclaration(declaration) => {
                Some((declaration.span.start, declaration.span.end))
            }
            Statement::ExportNamedDeclaration(declaration) if declaration.source.is_some() => {
                Some((declaration.span.start, declaration.span.end))
            }
            Statement::ExportAllDeclaration(declaration) => {
                Some((declaration.span.start, declaration.span.end))
            }
            _ => None,
        })
        .collect()
}

fn collect_declaration_exports(
    declaration: &Declaration<'_>,
    seen: &mut FxHashSet<String>,
    names: &mut Vec<String>,
) {
    match declaration {
        Declaration::VariableDeclaration(variable) => {
            for declarator in &variable.declarations {
                collect_binding_names(&declarator.id, seen, names);
            }
        }
        Declaration::FunctionDeclaration(function) => {
            if let Some(id) = &function.id {
                push_name(id.name.as_str(), seen, names);
            }
        }
        Declaration::ClassDeclaration(class) => {
            if let Some(id) = &class.id {
                push_name(id.name.as_str(), seen, names);
            }
        }
        Declaration::TSEnumDeclaration(enumeration) => {
            push_name(enumeration.id.name.as_str(), seen, names);
        }
        _ => {}
    }
}

fn collect_binding_names(
    pattern: &BindingPattern<'_>,
    seen: &mut FxHashSet<String>,
    names: &mut Vec<String>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => push_name(id.name.as_str(), seen, names),
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                collect_binding_names(&property.value, seen, names);
            }
            if let Some(rest) = &object.rest {
                collect_binding_names(&rest.argument, seen, names);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                collect_binding_names(element, seen, names);
            }
            if let Some(rest) = &array.rest {
                collect_binding_names(&rest.argument, seen, names);
            }
        }
        BindingPattern::AssignmentPattern(assignment) => {
            collect_binding_names(&assignment.left, seen, names);
        }
    }
}

fn push_name(name: &str, seen: &mut FxHashSet<String>, names: &mut Vec<String>) {
    let name: String = name.into();
    if seen.insert(name.clone()) {
        names.push(name);
    }
}

pub(super) fn define_props_result_bindings(
    program: &oxc_ast::ast::Program<'_>,
) -> FxHashSet<String> {
    #[derive(Default)]
    struct Collector {
        bindings: FxHashSet<String>,
    }
    impl<'a> Visit<'a> for Collector {
        fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
            if let BindingPattern::BindingIdentifier(binding) = &declarator.id
                && declarator
                    .init
                    .as_ref()
                    .is_some_and(is_define_props_result_expression)
            {
                self.bindings.insert(binding.name.as_str().into());
            }
            walk::walk_variable_declarator(self, declarator);
        }
    }
    let mut collector = Collector::default();
    collector.visit_program(program);
    collector.bindings
}

fn is_define_props_result_expression(expr: &Expression<'_>) -> bool {
    let Expression::CallExpression(call) = expr else {
        return false;
    };
    is_define_props_call(call) || is_with_defaults_define_props_call(call)
}

fn is_define_props_call(call: &CallExpression<'_>) -> bool {
    matches!(
        &call.callee,
        Expression::Identifier(ident) if ident.name.as_str() == "defineProps"
    )
}

fn is_with_defaults_define_props_call(call: &CallExpression<'_>) -> bool {
    matches!(
        (&call.callee, call.arguments.first()),
        (Expression::Identifier(ident), Some(Argument::CallExpression(inner)))
            if ident.name.as_str() == "withDefaults" && is_define_props_call(inner)
    )
}

fn record_type_name_root(name: &TSTypeName<'_>, refs: &mut FxHashSet<String>) {
    match name {
        TSTypeName::IdentifierReference(ident) => {
            refs.insert(ident.name.as_str().into());
        }
        TSTypeName::QualifiedName(qualified) => record_type_name_root(&qualified.left, refs),
        TSTypeName::ThisExpression(_) => {}
    }
}

fn record_type_query_root(name: &TSTypeQueryExprName<'_>, refs: &mut FxHashSet<String>) {
    match name {
        TSTypeQueryExprName::IdentifierReference(ident) => {
            refs.insert(ident.name.as_str().into());
        }
        TSTypeQueryExprName::QualifiedName(qualified) => {
            record_type_name_root(&qualified.left, refs);
        }
        TSTypeQueryExprName::TSImportType(_) => {}
        _ => {}
    }
}

pub(super) use boolean_keys::define_props_boolean_keys;
