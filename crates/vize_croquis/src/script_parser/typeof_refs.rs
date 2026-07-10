//! Collects identifiers referenced via `typeof X` in type position.
//!
//! Used by virtual-TS generation to decide whether a `type` / `interface`
//! declaration may be hoisted to module scope. If the body of a type
//! declaration uses `typeof X` and `X` is a setup-scope value binding
//! (i.e. a `const`/`let`/`var`/function declared inside `<script setup>`),
//! the declaration must stay in the setup function — otherwise the
//! generated TS at module level cannot resolve `X` and TS reports
//! `Cannot find name 'X'`.

use oxc_ast::ast::{
    Expression, TSInterfaceDeclaration, TSType, TSTypeAliasDeclaration, TSTypeName,
    TSTypeQueryExprName, TSTypeReference,
};
use oxc_ast_visit::{Visit, walk};
use vize_carton::{CompactString, FxHashSet};

#[derive(Default)]
pub(crate) struct TypeDependencyRefs {
    pub typeof_value_refs: FxHashSet<CompactString>,
    pub type_refs: FxHashSet<CompactString>,
}

/// Visitor that records the leftmost identifier of every `typeof X[.Y.Z]`
/// it encounters inside a TypeScript type tree.
#[derive(Default)]
pub(crate) struct TypeofValueRefs {
    pub refs: TypeDependencyRefs,
}

impl<'a> Visit<'a> for TypeofValueRefs {
    fn visit_ts_type_reference(&mut self, it: &TSTypeReference<'a>) {
        record_type_name_root(&it.type_name, &mut self.refs.type_refs);
        walk::walk_ts_type_reference(self, it);
    }

    fn visit_ts_type_query_expr_name(&mut self, it: &TSTypeQueryExprName<'a>) {
        match it {
            TSTypeQueryExprName::IdentifierReference(id) => {
                self.refs
                    .typeof_value_refs
                    .insert(CompactString::new(id.name.as_str()));
            }
            TSTypeQueryExprName::QualifiedName(qn) => {
                // `typeof A.B.C` — only the root identifier `A` resolves to a
                // value binding; `.B.C` are property lookups on that value's
                // type.
                record_type_name_root(&qn.left, &mut self.refs.typeof_value_refs);
            }
            TSTypeQueryExprName::TSImportType(_) => {
                // `typeof import('foo')` resolves at module scope; never a
                // setup-scope reference.
            }
            // `typeof this` is permitted by the TSTypeQueryExprName grammar
            // via inherited TSTypeName variants but never names a value
            // binding we can detect, so just skip it.
            _ => {}
        }
        walk::walk_ts_type_query_expr_name(self, it);
    }
}

fn record_type_name_root(name: &TSTypeName<'_>, refs: &mut FxHashSet<CompactString>) {
    match name {
        TSTypeName::IdentifierReference(id) => {
            refs.insert(CompactString::new(id.name.as_str()));
        }
        TSTypeName::QualifiedName(inner) => record_type_name_root(&inner.left, refs),
        TSTypeName::ThisExpression(_) => {}
    }
}

fn record_expression_root(expr: &Expression<'_>, refs: &mut FxHashSet<CompactString>) {
    match expr {
        Expression::Identifier(id) => {
            refs.insert(CompactString::new(id.name.as_str()));
        }
        Expression::StaticMemberExpression(member) => record_expression_root(&member.object, refs),
        _ => {}
    }
}

/// Collect `typeof` value identifier refs in a type alias body
/// (`type X = ...`), including its generic parameter constraints and
/// default types.
pub(crate) fn collect_from_type_alias(decl: &TSTypeAliasDeclaration<'_>) -> TypeDependencyRefs {
    let mut visitor = TypeofValueRefs::default();
    visitor.visit_ts_type(&decl.type_annotation);
    if let Some(params) = &decl.type_parameters {
        for param in params.params.iter() {
            if let Some(constraint) = &param.constraint {
                visitor.visit_ts_type(constraint);
            }
            if let Some(default) = &param.default {
                visitor.visit_ts_type(default);
            }
        }
    }
    visitor.refs
}

/// Collect `typeof` value identifier refs in an interface body
/// (`interface X { ... }`), including its extends clauses and generic
/// parameter constraints / defaults.
pub(crate) fn collect_from_interface(decl: &TSInterfaceDeclaration<'_>) -> TypeDependencyRefs {
    let mut visitor = TypeofValueRefs::default();
    visitor.visit_ts_interface_body(&decl.body);
    for clause in decl.extends.iter() {
        record_expression_root(&clause.expression, &mut visitor.refs.type_refs);
        if let Some(args) = &clause.type_arguments {
            for arg in args.params.iter() {
                visitor.visit_ts_type(arg);
            }
        }
    }
    if let Some(params) = &decl.type_parameters {
        for param in params.params.iter() {
            if let Some(constraint) = &param.constraint {
                visitor.visit_ts_type(constraint);
            }
            if let Some(default) = &param.default {
                visitor.visit_ts_type(default);
            }
        }
    }
    visitor.refs
}

/// Dispatch helper: collect refs from any `Declaration` that may carry
/// a type alias or interface (used by `export type` / `export interface`).
pub(crate) fn collect_from_declaration(decl: &oxc_ast::ast::Declaration<'_>) -> TypeDependencyRefs {
    match decl {
        oxc_ast::ast::Declaration::TSTypeAliasDeclaration(t) => collect_from_type_alias(t),
        oxc_ast::ast::Declaration::TSInterfaceDeclaration(i) => collect_from_interface(i),
        _ => TypeDependencyRefs::default(),
    }
}

// `TSType` is re-exported here so the `Visit` impl above type-checks against
// the AST crate version used by the workspace.
#[allow(dead_code)]
fn _ts_type_witness(_: &TSType<'_>) {}
