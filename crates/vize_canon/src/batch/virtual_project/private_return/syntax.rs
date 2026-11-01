//! One complete emitted-TS parse, preserving the exact old independent body.

use std::ops::Range;

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};
use vize_carton::String;

pub(super) const MOVED_NAMES: [&str; 5] = [
    "$UsePublicInstance",
    "$ReplaceComponentProps",
    "$AuthoredKeyWitness",
    "$ComponentInputProps",
    "$ComponentInputGuard",
];

#[derive(Debug, PartialEq, Eq)]
pub(super) struct ReturnSite {
    pub(super) range: Range<usize>,
    pub(super) props: String,
    pub(super) declared: String,
}

pub(super) fn analyze(source: &str) -> Option<ReturnSite> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    let aliases = parsed
        .program
        .body
        .iter()
        .filter_map(|statement| match statement {
            Statement::TSTypeAliasDeclaration(alias)
                if alias.id.name == "$ComponentConstructor" =>
            {
                Some(alias)
            }
            _ => None,
        })
        .take(2)
        .collect::<Vec<_>>();
    let [alias] = aliases.as_slice() else {
        return None;
    };
    if alias.type_parameters.is_some() {
        return None;
    }
    let TSType::TSConstructorType(constructor) = &alias.type_annotation else {
        return None;
    };
    let parameters = &constructor.type_parameters.as_ref()?.params;
    let [parameter] = parameters.as_slice() else {
        return None;
    };
    if constructor.r#abstract
        || parameter.name.name != "$AuthoredProps"
        || parameter.constraint.is_some()
        || !matches!(&parameter.default, Some(TSType::TSUnknownKeyword(_)))
    {
        return None;
    }
    let body = &constructor.return_type.type_annotation;
    let TSType::TSConditionalType(conditional) = body else {
        return None;
    };
    let replace = arguments(&conditional.false_type, "$ReplaceComponentProps")?;
    let [_, input] = replace else {
        return None;
    };
    let TSType::TSIntersectionType(intersection) = input else {
        return None;
    };
    let [_, props, _] = intersection.types.as_slice() else {
        return None;
    };
    let [props, declared] = arguments(props, "$ComponentInputProps")? else {
        return None;
    };
    let props = text(source, props)?;
    let declared = text(source, declared)?;
    // An invalid generic P/D reference used to occur twice in this return.
    // Passing it once would remove a raw TS2314. Admit only a module-local,
    // unique, zero-arity Props alias plus the literal no-emits object.
    if props != "Props" || declared != "{}" {
        return None;
    }
    let props_aliases = parsed
        .program
        .body
        .iter()
        .filter_map(|statement| {
            match statement {
                Statement::TSTypeAliasDeclaration(alias) => Some(alias.as_ref()),
                Statement::ExportNamedDeclaration(export) => match &export.declaration {
                    Some(Declaration::TSTypeAliasDeclaration(alias)) => Some(alias.as_ref()),
                    _ => None,
                },
                _ => None,
            }
            .filter(|alias| alias.id.name == "Props")
        })
        .take(2)
        .collect::<Vec<_>>();
    let [props_alias] = props_aliases.as_slice() else {
        return None;
    };
    if props_alias.type_parameters.is_some()
        || !parsed.program.body.iter().any(|statement| {
            matches!(
                statement,
                Statement::ExportNamedDeclaration(_)
                    | Statement::ExportDefaultDeclaration(_)
                    | Statement::ImportDeclaration(_)
                    | Statement::ExportAllDeclaration(_)
            )
        })
    {
        return None;
    }
    let expected = vize_carton::cstr!(
        "$UsePublicInstance<$AuthoredProps> extends true ? $ComponentInstance : $ReplaceComponentProps<$ComponentInstance, $AuthoredKeyWitness<$AuthoredProps> & $ComponentInputProps<{props}, {declared}> & $ComponentInputGuard<{props}, {declared}, $AuthoredProps>>"
    );
    if text(source, body)? != expected.as_str() {
        return None;
    }
    let mut hygiene = Hygiene {
        generated: alias.span,
        props_binding: props_alias.id.span,
        refused: false,
    };
    hygiene.visit_program(&parsed.program);
    if hygiene.refused {
        return None;
    }
    let span = body.span();
    Some(ReturnSite {
        range: span.start as usize..span.end as usize,
        props: String::from(props),
        declared: String::from(declared),
    })
}

fn arguments<'a, 'b>(ty: &'b TSType<'a>, name: &str) -> Option<&'b [TSType<'a>]> {
    let TSType::TSTypeReference(reference) = ty else {
        return None;
    };
    let TSTypeName::IdentifierReference(identifier) = &reference.type_name else {
        return None;
    };
    (identifier.name == name).then_some(())?;
    Some(reference.type_arguments.as_ref()?.params.as_slice())
}

fn text<'a>(source: &'a str, ty: &TSType<'_>) -> Option<&'a str> {
    let span = ty.span();
    source.get(span.start as usize..span.end as usize)
}

struct Hygiene {
    generated: Span,
    props_binding: Span,
    refused: bool,
}

impl Hygiene {
    fn identifier(&mut self, name: &str, span: Span, binding: bool) {
        if self.generated.start <= span.start && span.end <= self.generated.end {
            return;
        }
        if MOVED_NAMES.contains(&name)
            || (binding && name == "$ComponentConstructor")
            || (binding && name == "Props" && span != self.props_binding)
        {
            self.refused = true;
        }
    }
}

impl<'a> Visit<'a> for Hygiene {
    fn visit_binding_identifier(&mut self, node: &BindingIdentifier<'a>) {
        self.identifier(node.name.as_str(), node.span, true);
    }

    fn visit_identifier_reference(&mut self, node: &IdentifierReference<'a>) {
        self.identifier(node.name.as_str(), node.span, false);
    }

    fn visit_identifier_name(&mut self, node: &IdentifierName<'a>) {
        self.identifier(node.name.as_str(), node.span, false);
    }

    fn visit_ts_module_declaration(&mut self, node: &TSModuleDeclaration<'a>) {
        self.refused = true;
        walk::walk_ts_module_declaration(self, node);
    }

    fn visit_import_expression(&mut self, node: &ImportExpression<'a>) {
        if !matches!(&node.source, Expression::StringLiteral(_)) {
            self.refused = true;
        }
        walk::walk_import_expression(self, node);
    }
}

#[cfg(test)]
mod tests;
