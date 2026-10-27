//! Only identity-preserving type queries and aliases establish provenance.
use super::{BindingRef, BindingTarget, LocalSymbol, Namespace, Usage, collect::Collector};
use oxc_ast::ast::{
    Expression, IdentifierReference, TSImportType, TSImportTypeQualifier, TSLiteral, TSType,
    TSTypeName, TSTypeQueryExprName,
};
use vize_carton::String;

impl<'a> Collector<'_, 'a> {
    pub(super) fn reference(
        &self,
        id: &IdentifierReference<'a>,
        namespace: Namespace,
        usage: Usage,
    ) -> BindingTarget {
        if let Some(symbol) = id
            .reference_id
            .get()
            .and_then(|reference| self.semantic.scoping().get_reference(reference).symbol_id())
        {
            BindingTarget::Alias {
                binding: BindingRef {
                    source: self.facts.revision.source,
                    local: LocalSymbol { symbol, namespace },
                },
                usage,
            }
        } else {
            BindingTarget::Global {
                name: String::from(id.name.as_str()),
                namespace,
                usage,
            }
        }
    }
    fn imported_type_query(
        &self,
        import: &TSImportType<'a>,
        member: Option<&str>,
    ) -> BindingTarget {
        if import.options.is_some() || import.type_arguments.is_some() {
            return BindingTarget::Opaque;
        }
        let exported = match (member, &import.qualifier) {
            (Some(member), None) => member,
            (None, Some(TSImportTypeQualifier::Identifier(name))) => name.name.as_str(),
            _ => return BindingTarget::Opaque,
        };
        BindingTarget::Import {
            module: String::from(import.source.value.as_str()),
            exported: String::from(exported),
            namespace: Namespace::Value,
            type_only: false,
            usage: Some(Usage::TypeQuery),
        }
    }
    fn namespace_query(&self, id: &IdentifierReference<'a>, member: &str) -> BindingTarget {
        let Some(symbol) = id
            .reference_id
            .get()
            .and_then(|reference| self.semantic.scoping().get_reference(reference).symbol_id())
        else {
            return BindingTarget::Opaque;
        };
        let Some(import) = self.declarations.namespaces.get(&symbol) else {
            return BindingTarget::Opaque;
        };
        BindingTarget::Import {
            module: String::from(import.source.value.as_str()),
            exported: String::from(member),
            namespace: Namespace::Value,
            type_only: import.import_kind.is_type(),
            usage: Some(Usage::TypeQuery),
        }
    }
    pub(super) fn annotation(&self, annotation: &TSType<'a>) -> BindingTarget {
        match annotation {
            TSType::TSTypeQuery(query) if query.type_arguments.is_none() => {
                match &query.expr_name {
                    TSTypeQueryExprName::TSImportType(import) => {
                        self.imported_type_query(import, None)
                    }
                    TSTypeQueryExprName::IdentifierReference(id) => {
                        self.reference(id, Namespace::Value, Usage::TypeQuery)
                    }
                    TSTypeQueryExprName::QualifiedName(name) => {
                        if let TSTypeName::IdentifierReference(id) = &name.left {
                            self.namespace_query(id, name.right.name.as_str())
                        } else {
                            BindingTarget::Opaque
                        }
                    }
                    _ => BindingTarget::Opaque,
                }
            }
            TSType::TSTypeReference(reference) if reference.type_arguments.is_none() => {
                if let TSTypeName::IdentifierReference(id) = &reference.type_name {
                    self.reference(id, Namespace::Type, Usage::TypeQuery)
                } else {
                    BindingTarget::Opaque
                }
            }
            TSType::TSIndexedAccessType(indexed) => {
                if let TSType::TSTypeQuery(query) = &indexed.object_type
                    && query.type_arguments.is_none()
                    && let TSType::TSLiteralType(literal) = &indexed.index_type
                    && let TSLiteral::StringLiteral(member) = &literal.literal
                {
                    match &query.expr_name {
                        TSTypeQueryExprName::TSImportType(import) => {
                            self.imported_type_query(import, Some(member.value.as_str()))
                        }
                        TSTypeQueryExprName::IdentifierReference(id) => {
                            self.namespace_query(id, member.value.as_str())
                        }
                        _ => BindingTarget::Opaque,
                    }
                } else {
                    BindingTarget::Opaque
                }
            }
            _ => BindingTarget::Opaque,
        }
    }
    pub(super) fn value_alias(&self, expression: &Expression<'a>) -> BindingTarget {
        let mut expression = expression;
        loop {
            match expression {
                Expression::Identifier(id) => {
                    return self.reference(id, Namespace::Value, Usage::Runtime);
                }
                Expression::ParenthesizedExpression(parenthesized) => {
                    expression = &parenthesized.expression
                }
                _ => return BindingTarget::Opaque,
            }
        }
    }
}
