//! Collects value identifiers referenced from type position.
//!
//! Used by virtual-TS generation to decide whether a `type` / `interface`
//! declaration may be hoisted to module scope. `typeof X` and computed type
//! members such as `[Keys.A]` both read values. When that value is declared
//! inside `<script setup>`, the declaration must stay in the setup function;
//! otherwise generated module-level TS cannot resolve it.

use oxc_ast::ast::{
    Expression, IdentifierReference, PropertyKey, TSInterfaceDeclaration, TSMethodSignature,
    TSPropertySignature, TSType, TSTypeAliasDeclaration, TSTypeName, TSTypeQueryExprName,
    TSTypeReference,
};
use oxc_ast_visit::{Visit, walk};
use vize_carton::{CompactString, FxHashSet};

#[derive(Default)]
pub(crate) struct TypeDependencyRefs {
    pub typeof_value_refs: FxHashSet<CompactString>,
    pub type_refs: FxHashSet<CompactString>,
}

/// Visitor that records value and type dependencies inside a TypeScript type
/// tree without confusing ordinary type names with runtime reads.
#[derive(Default)]
pub(crate) struct TypeDependencyCollector {
    pub refs: TypeDependencyRefs,
}

impl<'a> Visit<'a> for TypeDependencyCollector {
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

    fn visit_ts_property_signature(&mut self, it: &TSPropertySignature<'a>) {
        if it.computed {
            record_computed_key_refs(&it.key, &mut self.refs.typeof_value_refs);
        }
        walk::walk_ts_property_signature(self, it);
    }

    fn visit_ts_method_signature(&mut self, it: &TSMethodSignature<'a>) {
        if it.computed {
            record_computed_key_refs(&it.key, &mut self.refs.typeof_value_refs);
        }
        walk::walk_ts_method_signature(self, it);
    }
}

/// A computed key is an expression, so collect every referenced identifier.
/// Static member names are `IdentifierName` nodes and are deliberately not
/// included: `[Keys.A]` records only `Keys`, while `[Keys[A]]` records both
/// runtime bindings.
struct ComputedKeyValueRefs<'r> {
    refs: &'r mut FxHashSet<CompactString>,
}

impl<'a> Visit<'a> for ComputedKeyValueRefs<'_> {
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        self.refs.insert(CompactString::new(it.name.as_str()));
    }
}

fn record_computed_key_refs(key: &PropertyKey<'_>, refs: &mut FxHashSet<CompactString>) {
    ComputedKeyValueRefs { refs }.visit_property_key(key);
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
    let mut visitor = TypeDependencyCollector::default();
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
    let mut visitor = TypeDependencyCollector::default();
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

/// Check only a type subtree reached by the existing demanded value walk.
pub(crate) fn has_value_type_reads(annotation: &TSType<'_>) -> bool {
    let mut visitor = TypeDependencyCollector::default();
    visitor.visit_ts_type(annotation);
    !visitor.refs.typeof_value_refs.is_empty()
}

pub(crate) fn annotation_has_value_reads(
    annotation: Option<&oxc_ast::ast::TSTypeAnnotation<'_>>,
) -> bool {
    annotation.is_some_and(|annotation| has_value_type_reads(&annotation.type_annotation))
}

pub(crate) fn parameters_have_value_reads(
    parameters: Option<&oxc_ast::ast::TSTypeParameterDeclaration<'_>>,
) -> bool {
    parameters.is_some_and(|parameters| {
        parameters.params.iter().any(|parameter| {
            parameter
                .constraint
                .as_ref()
                .is_some_and(has_value_type_reads)
                || parameter.default.as_ref().is_some_and(has_value_type_reads)
        })
    })
}

pub(crate) fn arguments_have_value_reads(
    arguments: Option<&oxc_ast::ast::TSTypeParameterInstantiation<'_>>,
) -> bool {
    arguments.is_some_and(|arguments| arguments.params.iter().any(has_value_type_reads))
}

// `TSType` is re-exported here so the `Visit` impl above type-checks against
// the AST crate version used by the workspace.
fn _ts_type_witness(_: &TSType<'_>) {}
