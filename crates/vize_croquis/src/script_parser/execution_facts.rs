//! Execution facts collected by the existing whole-program await visitor.

use oxc_ast::ast::{
    ArrowFunctionExpression, AssignmentTarget, BindingIdentifier, ComputedMemberExpression,
    ForOfStatement, Function, IdentifierReference, ImportDeclaration, ImportDeclarationSpecifier,
    Program, StaticMemberExpression,
};
use oxc_ast_visit::{Visit, walk};
use oxc_syntax::scope::ScopeFlags;
use vize_carton::{CompactString, FxHashMap, FxHashSet};

use super::reexecution::ReexecutedCallbacks;

pub(super) fn execution_facts(
    program: &Program<'_>,
    callbacks: bool,
) -> (bool, Option<Box<ReexecutedCallbacks>>) {
    if callbacks {
        let mut visitor = ExecutionVisitor::<true>::default();
        visitor.visit_program(program);
        let aliases: FxHashSet<CompactString> = visitor
            .vue_callbacks
            .into_iter()
            .filter(|name| visitor.bindings.get(name) == Some(&1) && !visitor.writes.contains(name))
            .collect();
        let callbacks = (!visitor.intrinsic_override && !aliases.is_empty()).then(|| {
            Box::new(ReexecutedCallbacks {
                aliases,
                frames: Vec::new(),
            })
        });
        (visitor.found_await, callbacks)
    } else {
        let mut visitor = ExecutionVisitor::<false>::default();
        visitor.visit_program(program);
        (visitor.found_await, None)
    }
}

#[derive(Default)]
struct ExecutionVisitor<const CALLBACKS: bool> {
    function_depth: usize,
    found_await: bool,
    intrinsic_override: bool,
    bindings: FxHashMap<CompactString, u32>,
    writes: FxHashSet<CompactString>,
    vue_callbacks: FxHashSet<CompactString>,
}

impl<'a, const CALLBACKS: bool> Visit<'a> for ExecutionVisitor<CALLBACKS> {
    fn visit_function(&mut self, it: &Function<'a>, flags: ScopeFlags) {
        self.function_depth += 1;
        walk::walk_function(self, it, flags);
        self.function_depth = self.function_depth.saturating_sub(1);
    }

    fn visit_arrow_function_expression(&mut self, it: &ArrowFunctionExpression<'a>) {
        self.function_depth += 1;
        walk::walk_arrow_function_expression(self, it);
        self.function_depth = self.function_depth.saturating_sub(1);
    }

    fn visit_await_expression(&mut self, it: &oxc_ast::ast::AwaitExpression<'a>) {
        if self.function_depth == 0 {
            self.found_await = true;
        }
        // The callback authority also covers reads/writes nested in an await.
        if CALLBACKS {
            walk::walk_await_expression(self, it);
        }
    }

    fn visit_for_of_statement(&mut self, it: &ForOfStatement<'a>) {
        if self.function_depth == 0 && it.r#await {
            self.found_await = true;
        }
        walk::walk_for_of_statement(self, it);
    }

    fn visit_import_declaration(&mut self, it: &ImportDeclaration<'a>) {
        if CALLBACKS
            && it.source.value == "vue"
            && !it.import_kind.is_type()
            && let Some(specifiers) = &it.specifiers
        {
            for specifier in specifiers {
                if let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier
                    && !specifier.import_kind.is_type()
                    && matches!(
                        specifier.imported.name().as_str(),
                        "computed" | "watchEffect"
                    )
                {
                    self.vue_callbacks
                        .insert(CompactString::new(specifier.local.name.as_str()));
                }
            }
        }
        walk::walk_import_declaration(self, it);
    }

    fn visit_binding_identifier(&mut self, it: &BindingIdentifier<'a>) {
        if CALLBACKS {
            *self
                .bindings
                .entry(CompactString::new(it.name.as_str()))
                .or_default() += 1;
        }
    }

    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        if CALLBACKS {
            // Conservative source-visible ways to obtain or replace intrinsics.
            // External monkeypatches are outside this static source contract.
            self.intrinsic_override |= matches!(
                it.name.as_str(),
                "Array"
                    | "Object"
                    | "Reflect"
                    | "globalThis"
                    | "window"
                    | "self"
                    | "global"
                    | "eval"
                    | "Function"
            );
        }
    }

    fn visit_static_member_expression(&mut self, it: &StaticMemberExpression<'a>) {
        if CALLBACKS {
            self.intrinsic_override |= matches!(
                it.property.name.as_str(),
                "prototype" | "constructor" | "__proto__"
            );
        }
        walk::walk_static_member_expression(self, it);
    }

    fn visit_computed_member_expression(&mut self, it: &ComputedMemberExpression<'a>) {
        // A dynamic property can hide prototype access or an accessor override.
        if CALLBACKS {
            self.intrinsic_override = true;
        }
        walk::walk_computed_member_expression(self, it);
    }

    fn visit_assignment_target(&mut self, it: &AssignmentTarget<'a>) {
        if CALLBACKS {
            match it {
                AssignmentTarget::AssignmentTargetIdentifier(identifier) => {
                    self.writes
                        .insert(CompactString::new(identifier.name.as_str()));
                }
                AssignmentTarget::StaticMemberExpression(member) => {
                    self.intrinsic_override |= member.property.name == "map";
                }
                _ => self.intrinsic_override = true,
            }
        }
        walk::walk_assignment_target(self, it);
    }
}
