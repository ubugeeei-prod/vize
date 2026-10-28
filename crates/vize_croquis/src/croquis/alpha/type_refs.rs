//! Type dependency reads from an OXC type tree, with lexical type scopes.

use std::cell::Cell;
use std::collections::BTreeSet;

use oxc_ast::ast::{
    Expression, Statement, TSImportType, TSImportTypeQualifier, TSInferType, TSInterfaceHeritage,
    TSMappedType, TSType, TSTypeName, TSTypeParameterDeclaration, TSTypeQuery, TSTypeReference,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_syntax::scope::{ScopeFlags, ScopeId};
use vize_carton::{Allocator, CompactString, cstr};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct TypeRef {
    pub root: CompactString,
    pub name: CompactString,
    pub module: Option<CompactString>,
}

pub(super) struct References {
    pub refs: BTreeSet<TypeRef>,
    pub complete: bool,
}

pub(super) fn is_object_type(source: &str) -> bool {
    let allocator = Allocator::new();
    let input = cstr!("type $$Hoge = {source};");
    let parsed = Parser::new(&allocator, &input, SourceType::ts()).parse();
    parsed.diagnostics.is_empty()
        && !parsed.panicked
        && matches!(parsed.program.body.as_slice(),
            [Statement::TSTypeAliasDeclaration(alias)]
                if matches!(alias.type_annotation, TSType::TSTypeLiteral(_)))
}

pub(super) fn collect(
    source: &str,
    local_parameters: Option<&str>,
    generic: Option<&str>,
) -> References {
    let outer = generic.map_or_else(CompactString::default, |generic| cstr!("<{generic}>"));
    let expression = local_parameters.map_or_else(
        || CompactString::new(source),
        |parameters| cstr!("{parameters}(__value: {source}) => void"),
    );
    let input = cstr!("type $$Hoge{outer} = {expression};");
    parse_source(&input, BTreeSet::new(), true)
}

pub(super) fn collect_declaration(source: &str, generic: Option<&str>) -> References {
    let allocator = Allocator::new();
    let input = generic.map_or_else(
        || CompactString::new("type $$Hoge = unknown;"),
        |generic| cstr!("type $$Hoge<{generic}> = unknown;"),
    );
    let parsed = Parser::new(&allocator, &input, SourceType::ts()).parse();
    let names = match parsed.program.body.as_slice() {
        [Statement::TSTypeAliasDeclaration(alias)] => alias
            .type_parameters
            .as_ref()
            .map(|parameters| {
                parameters
                    .params
                    .iter()
                    .map(|parameter| CompactString::new(parameter.name.name.as_str()))
                    .collect()
            })
            .unwrap_or_default(),
        _ => BTreeSet::new(),
    };
    parse_source(
        source,
        names,
        parsed.diagnostics.is_empty() && !parsed.panicked,
    )
}

fn parse_source(source: &str, outer_names: BTreeSet<CompactString>, complete: bool) -> References {
    let allocator = Allocator::new();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    let mut visitor = ReferencesVisitor {
        refs: BTreeSet::new(),
        scopes: vec![(outer_names, false)],
        complete: complete && parsed.diagnostics.is_empty() && !parsed.panicked,
    };
    visitor.visit_program(&parsed.program);
    References {
        refs: visitor.refs,
        complete: visitor.complete,
    }
}

struct ReferencesVisitor {
    refs: BTreeSet<TypeRef>,
    scopes: Vec<(BTreeSet<CompactString>, bool)>,
    complete: bool,
}

impl<'a> Visit<'a> for ReferencesVisitor {
    fn enter_scope(&mut self, flags: ScopeFlags, _: &Cell<Option<ScopeId>>) {
        self.scopes
            .push((BTreeSet::new(), flags.contains(ScopeFlags::TsConditional)));
    }

    fn leave_scope(&mut self) {
        self.scopes.pop();
    }

    fn visit_ts_type_parameter_declaration(&mut self, parameters: &TSTypeParameterDeclaration<'a>) {
        if let Some((scope, _)) = self.scopes.last_mut() {
            scope.extend(
                parameters
                    .params
                    .iter()
                    .map(|parameter| CompactString::new(parameter.name.name.as_str())),
            );
        }
        walk::walk_ts_type_parameter_declaration(self, parameters);
    }

    fn visit_ts_type_reference(&mut self, reference: &TSTypeReference<'a>) {
        if let Some(root) = root_name(&reference.type_name) {
            if !self
                .scopes
                .iter()
                .rev()
                .any(|(scope, _)| scope.contains(root))
            {
                self.refs.insert(TypeRef {
                    root: CompactString::new(root),
                    name: qualified_name(&reference.type_name),
                    module: None,
                });
            }
        } else {
            self.complete = false;
        }
        walk::walk_ts_type_reference(self, reference);
    }

    fn visit_ts_infer_type(&mut self, inferred: &TSInferType<'a>) {
        if let Some((scope, _)) = self
            .scopes
            .iter_mut()
            .rev()
            .find(|(_, conditional)| *conditional)
        {
            scope.insert(CompactString::new(
                inferred.type_parameter.name.name.as_str(),
            ));
        }
        walk::walk_ts_infer_type(self, inferred);
    }

    fn visit_ts_mapped_type(&mut self, mapped: &TSMappedType<'a>) {
        self.visit_ts_type(&mapped.constraint);
        self.scopes.push((
            BTreeSet::from([CompactString::new(mapped.key.name.as_str())]),
            false,
        ));
        if let Some(name_type) = &mapped.name_type {
            self.visit_ts_type(name_type);
        }
        if let Some(annotation) = &mapped.type_annotation {
            self.visit_ts_type(annotation);
        }
        self.scopes.pop();
    }

    fn visit_ts_type_query(&mut self, query: &TSTypeQuery<'a>) {
        // A value's inferred type needs the TS checker; no function body is
        // copied into an interface contract to pretend to resolve it.
        self.complete = false;
        walk::walk_ts_type_query(self, query);
    }

    fn visit_ts_import_type(&mut self, imported: &TSImportType<'a>) {
        let qualifier = imported
            .qualifier
            .as_ref()
            .map(import_qualified_name)
            .unwrap_or_else(|| CompactString::new("*"));
        self.refs.insert(TypeRef {
            root: qualifier.clone(),
            name: qualifier,
            module: Some(CompactString::new(imported.source.value.as_str())),
        });
        walk::walk_ts_import_type(self, imported);
    }

    fn visit_ts_interface_heritage(&mut self, heritage: &TSInterfaceHeritage<'a>) {
        if let Some((root, name)) = expression_name(&heritage.expression) {
            if !self
                .scopes
                .iter()
                .rev()
                .any(|(scope, _)| scope.contains(root))
            {
                self.refs.insert(TypeRef {
                    root: CompactString::new(root),
                    name,
                    module: None,
                });
            }
        } else {
            self.complete = false;
        }
        walk::walk_ts_interface_heritage(self, heritage);
    }
}

fn root_name<'a>(name: &'a TSTypeName<'_>) -> Option<&'a str> {
    match name {
        TSTypeName::IdentifierReference(identifier) => Some(identifier.name.as_str()),
        TSTypeName::QualifiedName(qualified) => root_name(&qualified.left),
        TSTypeName::ThisExpression(_) => None,
    }
}

fn qualified_name(name: &TSTypeName<'_>) -> CompactString {
    match name {
        TSTypeName::IdentifierReference(identifier) => CompactString::new(identifier.name.as_str()),
        TSTypeName::QualifiedName(qualified) => cstr!(
            "{}.{}",
            qualified_name(&qualified.left),
            qualified.right.name
        ),
        TSTypeName::ThisExpression(_) => CompactString::new("this"),
    }
}

fn import_qualified_name(name: &TSImportTypeQualifier<'_>) -> CompactString {
    match name {
        TSImportTypeQualifier::Identifier(identifier) => {
            CompactString::new(identifier.name.as_str())
        }
        TSImportTypeQualifier::QualifiedName(qualified) => cstr!(
            "{}.{}",
            import_qualified_name(&qualified.left),
            qualified.right.name
        ),
    }
}

fn expression_name<'a>(expression: &'a Expression<'_>) -> Option<(&'a str, CompactString)> {
    match expression {
        Expression::Identifier(identifier) => Some((
            identifier.name.as_str(),
            CompactString::new(identifier.name.as_str()),
        )),
        Expression::StaticMemberExpression(member) => expression_name(&member.object)
            .map(|(root, name)| (root, cstr!("{name}.{}", member.property.name))),
        _ => None,
    }
}
