use super::deps::collect_binding_pattern_names;
use super::{collect_reactive_sources, collect_vue_api_aliases, scoped_node_name};
use oxc_ast::ast::{Declaration, Program, Statement, VariableDeclaration};
use vize_carton::{CompactString, FxHashMap, FxHashSet};

#[derive(Debug, Default, Clone)]
pub(super) struct EffectBuildContext {
    pub(super) api_aliases: FxHashMap<CompactString, CompactString>,
    pub(super) blocked_api_names: FxHashSet<CompactString>,
    pub(super) reactive_sources: FxHashSet<CompactString>,
    pub(super) reactive_node_ids: FxHashMap<CompactString, CompactString>,
    pub(super) prefix: CompactString,
}

pub(super) fn context_for_program(
    program: &Program<'_>,
    inherited: Option<&EffectBuildContext>,
    prefix: &str,
) -> EffectBuildContext {
    let mut context = inherited.cloned().unwrap_or_default();
    context.prefix = CompactString::new(prefix);

    let mut local_bindings = FxHashSet::default();
    for statement in &program.body {
        collect_top_level_bindings(statement, &mut local_bindings);
    }
    for binding in local_bindings {
        context.api_aliases.remove(&binding);
        context.blocked_api_names.insert(binding.clone());
        context.reactive_sources.remove(&binding);
        context.reactive_node_ids.remove(&binding);
    }
    for statement in &program.body {
        collect_vue_api_aliases(statement, &mut context.api_aliases);
    }
    for api_name in context.api_aliases.keys() {
        context.blocked_api_names.remove(api_name);
    }

    let mut local_reactive_sources = FxHashSet::default();
    for statement in &program.body {
        collect_reactive_sources(
            statement,
            &context.api_aliases,
            &context.blocked_api_names,
            &mut local_reactive_sources,
        );
    }
    for source in local_reactive_sources {
        context.reactive_sources.insert(source.clone());
        context
            .reactive_node_ids
            .insert(source.clone(), scoped_node_name(prefix, &source));
    }
    context
}

pub(super) fn variable_declaration_from_statement<'a>(
    statement: &'a Statement<'a>,
) -> Option<&'a VariableDeclaration<'a>> {
    match statement {
        Statement::VariableDeclaration(declaration) => Some(declaration),
        Statement::ExportNamedDeclaration(export) => match export.declaration.as_ref() {
            Some(Declaration::VariableDeclaration(declaration)) => Some(declaration),
            _ => None,
        },
        _ => None,
    }
}

fn collect_top_level_bindings(statement: &Statement<'_>, bindings: &mut FxHashSet<CompactString>) {
    if let Some(declaration) = variable_declaration_from_statement(statement) {
        collect_variable_bindings(declaration, bindings);
        return;
    }
    match statement {
        Statement::FunctionDeclaration(function) => collect_named_binding(&function.id, bindings),
        Statement::ClassDeclaration(class) => collect_named_binding(&class.id, bindings),
        Statement::ExportNamedDeclaration(export) => {
            if let Some(declaration) = &export.declaration {
                match declaration {
                    Declaration::FunctionDeclaration(function) => {
                        collect_named_binding(&function.id, bindings);
                    }
                    Declaration::ClassDeclaration(class) => {
                        collect_named_binding(&class.id, bindings);
                    }
                    _ => {}
                }
            }
        }
        Statement::ImportDeclaration(import) => {
            if import.import_kind.is_type() {
                return;
            }
            for specifier in import.specifiers.iter().flatten() {
                let local = match specifier {
                    oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                        if specifier.import_kind.is_type() {
                            continue;
                        }
                        &specifier.local
                    }
                    oxc_ast::ast::ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                        &specifier.local
                    }
                    oxc_ast::ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(
                        specifier,
                    ) => &specifier.local,
                };
                bindings.insert(CompactString::new(local.name.as_str()));
            }
        }
        _ => {}
    }
}

fn collect_variable_bindings(
    declaration: &VariableDeclaration<'_>,
    bindings: &mut FxHashSet<CompactString>,
) {
    for declarator in &declaration.declarations {
        collect_binding_pattern_names(&declarator.id, bindings);
    }
}

fn collect_named_binding(
    identifier: &Option<oxc_ast::ast::BindingIdentifier<'_>>,
    bindings: &mut FxHashSet<CompactString>,
) {
    if let Some(identifier) = identifier {
        bindings.insert(CompactString::new(identifier.name.as_str()));
    }
}
