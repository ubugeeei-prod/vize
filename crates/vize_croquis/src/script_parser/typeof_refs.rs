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
    TSInterfaceDeclaration, TSType, TSTypeAliasDeclaration, TSTypeName, TSTypeQueryExprName,
};
use oxc_ast_visit::{Visit, walk};
use vize_carton::{CompactString, FxHashSet};

/// Visitor that records the leftmost identifier of every `typeof X[.Y.Z]`
/// it encounters inside a TypeScript type tree.
#[derive(Default)]
pub(crate) struct TypeofValueRefs {
    pub idents: FxHashSet<CompactString>,
}

impl<'a> Visit<'a> for TypeofValueRefs {
    fn visit_ts_type_query_expr_name(&mut self, it: &TSTypeQueryExprName<'a>) {
        match it {
            TSTypeQueryExprName::IdentifierReference(id) => {
                self.idents.insert(CompactString::new(id.name.as_str()));
            }
            TSTypeQueryExprName::QualifiedName(qn) => {
                // `typeof A.B.C` — only the root identifier `A` resolves to a
                // value binding; `.B.C` are property lookups on that value's
                // type.
                let mut left = &qn.left;
                loop {
                    match left {
                        TSTypeName::IdentifierReference(id) => {
                            self.idents.insert(CompactString::new(id.name.as_str()));
                            break;
                        }
                        TSTypeName::QualifiedName(inner) => left = &inner.left,
                        TSTypeName::ThisExpression(_) => break,
                    }
                }
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

/// Collect `typeof` value identifier refs in a type alias body
/// (`type X = ...`), including its generic parameter constraints and
/// default types.
pub(crate) fn collect_from_type_alias(
    decl: &TSTypeAliasDeclaration<'_>,
) -> FxHashSet<CompactString> {
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
    visitor.idents
}

/// Collect `typeof` value identifier refs in an interface body
/// (`interface X { ... }`), including its extends clauses and generic
/// parameter constraints / defaults.
pub(crate) fn collect_from_interface(
    decl: &TSInterfaceDeclaration<'_>,
) -> FxHashSet<CompactString> {
    let mut visitor = TypeofValueRefs::default();
    visitor.visit_ts_interface_body(&decl.body);
    for clause in decl.extends.iter() {
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
    visitor.idents
}

/// Dispatch helper: collect refs from any `Declaration` that may carry
/// a type alias or interface (used by `export type` / `export interface`).
pub(crate) fn collect_from_declaration(
    decl: &oxc_ast::ast::Declaration<'_>,
) -> FxHashSet<CompactString> {
    match decl {
        oxc_ast::ast::Declaration::TSTypeAliasDeclaration(t) => collect_from_type_alias(t),
        oxc_ast::ast::Declaration::TSInterfaceDeclaration(i) => collect_from_interface(i),
        _ => FxHashSet::default(),
    }
}

// `TSType` is re-exported here so the `Visit` impl above type-checks against
// the AST crate version used by the workspace.
#[allow(dead_code)]
fn _ts_type_witness(_: &TSType<'_>) {}
