//! Script-side facts for `script/prefer-use-template-ref`: which callees are
//! Vue's `ref` / `shallowRef`, which refs the script writes, and the authored
//! type argument the suggested `useTemplateRef` keeps.

use oxc_ast::ast::{
    AssignmentExpression, AssignmentTarget, CallExpression, Expression, ImportDeclarationSpecifier,
    Program, SimpleAssignmentTarget, Statement, StaticMemberExpression, TSType, UpdateExpression,
};
use oxc_ast_visit::{Visit, walk};
use oxc_semantic::Scoping;
use oxc_span::{GetSpan, Span};
use vize_l0::{CompactString, FxHashMap, FxHashSet};

use super::super::composition_api::{is_composition_api_module, reexports_vue_ref_factories};

/// What a declarator's callee is, as far as the script's own imports tell.
/// `S` borrows the callee's names while declarations are filtered and owns
/// them once one is kept.
pub(super) enum Callee<S> {
    /// Vue's `ref` or `shallowRef`.
    Vue(&'static str),
    /// A name imported from another module (`local`, or `local.member`
    /// through a namespace import). It is Vue's factory only when that module
    /// re-exports one, which [`super::reexports`] decides.
    Imported { local: S, member: Option<S> },
}

impl Callee<&str> {
    pub(super) fn owned(&self) -> Callee<CompactString> {
        match self {
            Self::Vue(callee) => Callee::Vue(callee),
            Self::Imported { local, member } => Callee::Imported {
                local: CompactString::new(local),
                member: member.map(CompactString::new),
            },
        }
    }
}

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
    /// Vue's only when that module re-exports it.
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
            let source = import.source.value.as_str();
            if !is_composition_api_module(source) && !reexports_vue_ref_factories(source) {
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

    /// What `callee` calls: Vue's `ref` / `shallowRef`, or a name another
    /// module's import provides.
    pub(super) fn factory<'e>(&self, callee: &'e Expression<'_>) -> Option<Callee<&'e str>> {
        match callee.get_inner_expression() {
            Expression::Identifier(identifier) => {
                let name = identifier.name.as_str();
                match self.vue_imports.get(name) {
                    Some(callee) => Some(Callee::Vue(callee)),
                    None if self.foreign.contains(name) => Some(Callee::Imported {
                        local: name,
                        member: None,
                    }),
                    None => vue_factory(name).map(Callee::Vue),
                }
            }
            Expression::StaticMemberExpression(member) => match &member.object {
                Expression::Identifier(namespace)
                    if self.namespaces.contains(namespace.name.as_str()) =>
                {
                    vue_factory(member.property.name.as_str()).map(Callee::Vue)
                }
                Expression::Identifier(namespace)
                    if self.foreign.contains(namespace.name.as_str()) =>
                {
                    Some(Callee::Imported {
                        local: namespace.name.as_str(),
                        member: Some(member.property.name.as_str()),
                    })
                }
                _ => None,
            },
            _ => None,
        }
    }
}

pub(super) fn vue_factory(name: &str) -> Option<&'static str> {
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
///
/// A name is only a first cut: a nested binding with the same name (a local
/// `input` element in a helper) writes its own `.value`. [`WrittenBindings`]
/// decides which declaration a write of a collected name reaches.
#[derive(Default)]
pub(super) struct ValueWrites {
    pub(super) names: FxHashSet<CompactString>,
}

impl ValueWrites {
    fn record(&mut self, member: &StaticMemberExpression<'_>) {
        if let Some(identifier) = value_write_object(member) {
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

fn value_write_object<'m, 'a>(
    member: &'m StaticMemberExpression<'a>,
) -> Option<&'m oxc_ast::ast::IdentifierReference<'a>> {
    match member.object.get_inner_expression() {
        Expression::Identifier(identifier) if member.property.name == "value" => Some(identifier),
        _ => None,
    }
}

/// The declarations whose own binding a `.value` write reaches, by the span
/// of the binding identifier. A write whose reference did not resolve keeps
/// its name-only answer: every declaration with that name counts as written.
pub(super) struct WrittenBindings<'s> {
    scoping: &'s Scoping,
    names: &'s FxHashSet<CompactString>,
    bindings: FxHashSet<Span>,
    unresolved: FxHashSet<CompactString>,
}

impl<'s> WrittenBindings<'s> {
    pub(super) fn collect(
        program: &Program<'_>,
        scoping: &'s Scoping,
        names: &'s FxHashSet<CompactString>,
    ) -> Self {
        let mut written = Self {
            scoping,
            names,
            bindings: FxHashSet::default(),
            unresolved: FxHashSet::default(),
        };
        written.visit_program(program);
        written
    }

    /// Whether the script writes `.value` of the binding declared at `span`.
    pub(super) fn contains(&self, name: &str, span: Span) -> bool {
        self.bindings.contains(&span) || self.unresolved.contains(name)
    }

    fn record(&mut self, member: &StaticMemberExpression<'_>) {
        let Some(identifier) = value_write_object(member) else {
            return;
        };
        let name = identifier.name.as_str();
        if !self.names.contains(name) {
            return;
        }
        let symbol = identifier
            .reference_id
            .get()
            .map(|reference| self.scoping.get_reference(reference).symbol_id());
        match symbol {
            // A write to an undeclared global reaches no declaration.
            Some(None) => {}
            Some(Some(symbol)) => {
                self.bindings.insert(self.scoping.symbol_span(symbol));
            }
            None => {
                self.unresolved.insert(name.into());
            }
        }
    }
}

impl<'a> Visit<'a> for WrittenBindings<'_> {
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
