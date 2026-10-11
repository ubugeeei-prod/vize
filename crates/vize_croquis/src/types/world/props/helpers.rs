use oxc_allocator::Allocator;
use oxc_ast::ast::{PropertyKey, Statement, TSLiteral, TSSignature, TSType};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};
use oxc_syntax::number::ToJsString;
use vize_carton::{CompactString, FxHashSet, cstr};

use super::{ScopedTypeProperties, ScopedTypeProperty};

pub(super) fn text(source: &str, span: Span) -> CompactString {
    CompactString::new(
        source
            .get(span.start as usize..span.end as usize)
            .unwrap_or_default(),
    )
}

pub(super) fn members(
    module: &str,
    source: &str,
    members: &[TSSignature<'_>],
) -> ScopedTypeProperties {
    let mut output = ScopedTypeProperties {
        complete: true,
        ..ScopedTypeProperties::default()
    };
    for member in members {
        let TSSignature::TSPropertySignature(property) = member else {
            output.complete = false;
            continue;
        };
        let name = match &property.key {
            PropertyKey::StaticIdentifier(id) if !property.computed => {
                CompactString::new(id.name.as_str())
            }
            PropertyKey::StringLiteral(literal) => CompactString::new(literal.value.as_str()),
            PropertyKey::NumericLiteral(literal) => {
                CompactString::new(literal.value.to_js_string())
            }
            _ => {
                output.complete = false;
                continue;
            }
        };
        output.properties.push(ScopedTypeProperty {
            name,
            prop_type: property
                .type_annotation
                .as_ref()
                .map(|annotation| text(source, annotation.type_annotation.span())),
            optional: property.optional,
            module: CompactString::new(module),
        });
    }
    output
}

pub(super) fn combine(
    output: &mut ScopedTypeProperties,
    additions: ScopedTypeProperties,
    override_existing: bool,
) {
    output.complete &= additions.complete;
    for prop in additions.properties {
        if let Some(previous) = output
            .properties
            .iter_mut()
            .find(|previous| previous.name == prop.name)
        {
            if override_existing {
                *previous = prop;
            } else if previous != &prop {
                output.complete = false;
                previous.prop_type = None;
                previous.optional &= prop.optional;
            }
        } else {
            output.properties.push(prop);
        }
    }
}

/// Vue merges the public keys of each union branch. A shared key is optional
/// when either authored declaration is optional; branch-local keys retain
/// their authored optionality, matching the macro compiler's runtime contract.
pub(super) fn combine_union(output: &mut ScopedTypeProperties, additions: ScopedTypeProperties) {
    output.complete &= additions.complete;
    for prop in additions.properties {
        if let Some(previous) = output
            .properties
            .iter_mut()
            .find(|previous| previous.name == prop.name)
        {
            previous.optional |= prop.optional;
            // A single textual member cannot own two lexical scopes. Retain
            // the public key but never resolve foreign aliases in the first
            // branch's module or in the root SFC.
            if previous.module != prop.module {
                output.complete = false;
                previous.prop_type = None;
            } else if previous.prop_type != prop.prop_type {
                match (&previous.prop_type, &prop.prop_type) {
                    (Some(left), Some(right)) => {
                        previous.prop_type = Some(cstr!("({left}) | ({right})"));
                    }
                    _ => {
                        output.complete = false;
                        previous.prop_type = None;
                    }
                }
            }
        } else {
            output.properties.push(prop);
        }
    }
}

pub(super) fn parameters(parameters: Option<&str>) -> FxHashSet<CompactString> {
    let Some(parameters) = parameters else {
        return FxHashSet::default();
    };
    let source = cstr!("type __Params{parameters} = never");
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    let Some(Statement::TSTypeAliasDeclaration(alias)) = parsed.program.body.first() else {
        return FxHashSet::default();
    };
    alias
        .type_parameters
        .as_ref()
        .map(|params| {
            params
                .params
                .iter()
                .map(|param| CompactString::new(param.name.name.as_str()))
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn literal_keys(ty: &TSType<'_>) -> Option<FxHashSet<CompactString>> {
    match ty {
        TSType::TSLiteralType(literal) => match &literal.literal {
            TSLiteral::StringLiteral(literal) => Some(FxHashSet::from_iter([CompactString::new(
                literal.value.as_str(),
            )])),
            _ => None,
        },
        TSType::TSUnionType(union) => {
            let mut keys = FxHashSet::default();
            for ty in &union.types {
                keys.extend(literal_keys(ty)?);
            }
            Some(keys)
        }
        _ => None,
    }
}
