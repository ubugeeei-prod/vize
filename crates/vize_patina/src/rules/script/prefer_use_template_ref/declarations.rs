//! Script-side facts for `script/prefer-use-template-ref`: which callees are
//! Vue's `ref` / `shallowRef`, which refs the script writes, and the authored
//! type argument the suggested `useTemplateRef` keeps.

use oxc_ast::ast::{
    AssignmentExpression, AssignmentTarget, CallExpression, Expression, ImportDeclarationSpecifier,
    Program, SimpleAssignmentTarget, Statement, StaticMemberExpression, TSType, UpdateExpression,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::GetSpan;
use vize_l0::{CompactString, FxHashMap, FxHashSet};

/// How `ref` / `shallowRef` are reachable in a script: imported from Vue under
/// any local name, through a Vue namespace import, or as the bare names when
/// no other module's import claims them (auto-imported).
#[derive(Default)]
pub(super) struct RefCallees {
    /// Local names imported from Vue, mapped to the Vue export they name.
    vue_imports: FxHashMap<CompactString, &'static str>,
    /// Local names of Vue namespace imports (`import * as Vue from 'vue'`).
    namespaces: FxHashSet<CompactString>,
    /// Local names imported from other modules: a `ref` from `./local` is
    /// not Vue's.
    foreign: FxHashSet<CompactString>,
}

impl RefCallees {
    /// Record the Vue imports of one script block.
    pub(super) fn add_imports(&mut self, program: &Program<'_>) {
        let callees = self;
        for statement in &program.body {
            let Statement::ImportDeclaration(import) = statement else {
                continue;
            };
            if import.import_kind.is_type() {
                continue;
            }
            if !super::super::composition_api::is_composition_api_module(
                import.source.value.as_str(),
            ) {
                for specifier in import.specifiers.iter().flatten() {
                    callees
                        .foreign
                        .insert(specifier.local().name.as_str().into());
                }
                continue;
            }
            for specifier in import.specifiers.iter().flatten() {
                match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier)
                        if !specifier.import_kind.is_type() =>
                    {
                        if let Some(callee) = vue_factory(specifier.imported.name().as_str()) {
                            callees
                                .vue_imports
                                .insert(specifier.local.name.as_str().into(), callee);
                        }
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        callees
                            .namespaces
                            .insert(specifier.local.name.as_str().into());
                    }
                    _ => {}
                }
            }
        }
    }

    /// The Vue factory `callee` calls, when it is `ref` or `shallowRef`.
    pub(super) fn factory(&self, callee: &Expression<'_>) -> Option<&'static str> {
        match callee.get_inner_expression() {
            Expression::Identifier(identifier) => {
                let name = identifier.name.as_str();
                match self.vue_imports.get(name) {
                    Some(callee) => Some(*callee),
                    None if self.foreign.contains(name) => None,
                    None => vue_factory(name),
                }
            }
            Expression::StaticMemberExpression(member) => match &member.object {
                Expression::Identifier(namespace)
                    if self.namespaces.contains(namespace.name.as_str()) =>
                {
                    vue_factory(member.property.name.as_str())
                }
                _ => None,
            },
            _ => None,
        }
    }
}

fn vue_factory(name: &str) -> Option<&'static str> {
    match name {
        "ref" => Some("ref"),
        "shallowRef" => Some("shallowRef"),
        _ => None,
    }
}

/// The call a declarator initializer evaluates, through parentheses and
/// TypeScript wrappers (`(ref(null))`, `ref(null) as Ref<T>`, `ref(null)!`).
pub(super) fn initializer_call<'e, 'a>(init: &'e Expression<'a>) -> Option<&'e CallExpression<'a>> {
    match init.get_inner_expression() {
        Expression::CallExpression(call) => Some(&**call),
        _ => None,
    }
}

/// The authored type argument without its `null` / `undefined` members, as
/// `useTemplateRef<T>` adds `| null` itself.
pub(super) fn template_ref_type_argument(
    call: &CallExpression<'_>,
    source: &str,
) -> Option<CompactString> {
    let argument = call.type_arguments.as_ref()?.params.first()?;
    let text = |ty: &TSType<'_>| {
        let span = ty.span();
        source.get(span.start as usize..span.end as usize)
    };
    let TSType::TSUnionType(union) = argument else {
        return text(argument).map(CompactString::new);
    };
    let mut printed = CompactString::default();
    for member in union.types.iter().filter(|member| {
        !matches!(
            member,
            TSType::TSNullKeyword(_) | TSType::TSUndefinedKeyword(_)
        )
    }) {
        if !printed.is_empty() {
            printed.push_str(" | ");
        }
        printed.push_str(text(member)?);
    }
    (!printed.is_empty()).then_some(printed)
}

/// Names whose `.value` the script assigns or updates. `useTemplateRef`
/// returns a readonly ref, so replacing such a ref would break the write.
#[derive(Default)]
pub(super) struct ValueWrites {
    pub(super) names: FxHashSet<CompactString>,
}

impl ValueWrites {
    fn record(&mut self, member: &StaticMemberExpression<'_>) {
        if member.property.name == "value"
            && let Expression::Identifier(identifier) = member.object.get_inner_expression()
        {
            self.names.insert(identifier.name.as_str().into());
        }
    }
}

impl<'a> Visit<'a> for ValueWrites {
    fn visit_assignment_expression(&mut self, it: &AssignmentExpression<'a>) {
        if let AssignmentTarget::StaticMemberExpression(member) = &it.left {
            self.record(member);
        }
        walk::walk_assignment_expression(self, it);
    }

    fn visit_update_expression(&mut self, it: &UpdateExpression<'a>) {
        if let SimpleAssignmentTarget::StaticMemberExpression(member) = &it.argument {
            self.record(member);
        }
        walk::walk_update_expression(self, it);
    }
}
