//! Resolve event names from public `defineEmits<T>()` contracts.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPattern, FormalParameters, PropertyKey, Statement, TSLiteral, TSSignature, TSType,
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{CompactString, FxHashSet, cstr};

use super::{ResolvedTypeWorld, TypeDeclarationId, TypeDeclarationKind, TypeLookup};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScopedTypeEvents {
    pub events: Vec<CompactString>,
    pub complete: bool,
}

impl ResolvedTypeWorld {
    pub fn resolve_events(&self, type_expression: &str) -> ScopedTypeEvents {
        self.events_in(
            &self.root_module,
            type_expression,
            &mut FxHashSet::default(),
        )
    }

    fn events_in(
        &self,
        module: &str,
        expression: &str,
        visiting: &mut FxHashSet<TypeDeclarationId>,
    ) -> ScopedTypeEvents {
        if visiting.len() >= 128 {
            return ScopedTypeEvents::default();
        }
        let source = cstr!("type $Emits = {expression}");
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
        if parsed.panicked || !parsed.diagnostics.is_empty() {
            return ScopedTypeEvents::default();
        }
        let Some(Statement::TSTypeAliasDeclaration(alias)) = parsed.program.body.first() else {
            return ScopedTypeEvents::default();
        };
        self.events_ast(module, &source, &alias.type_annotation, visiting)
    }

    fn events_ast(
        &self,
        module: &str,
        source: &str,
        ty: &TSType<'_>,
        visiting: &mut FxHashSet<TypeDeclarationId>,
    ) -> ScopedTypeEvents {
        match ty {
            TSType::TSTypeLiteral(literal) => events_from_members(&literal.members),
            TSType::TSFunctionType(function) => {
                let mut output = ScopedTypeEvents {
                    complete: true,
                    ..ScopedTypeEvents::default()
                };
                if function.this_param.is_none() {
                    output.events.extend(parameter_events(&function.params));
                }
                output
            }
            TSType::TSParenthesizedType(parenthesized) => {
                self.events_ast(module, source, &parenthesized.type_annotation, visiting)
            }
            TSType::TSIntersectionType(intersection) => combine_all(
                intersection
                    .types
                    .iter()
                    .map(|ty| self.events_ast(module, source, ty, visiting)),
            ),
            TSType::TSUnionType(union) => combine_all(
                union
                    .types
                    .iter()
                    .map(|ty| self.events_ast(module, source, ty, visiting)),
            ),
            TSType::TSTypeReference(reference) => {
                let name = source
                    .get(
                        reference.type_name.span().start as usize
                            ..reference.type_name.span().end as usize,
                    )
                    .unwrap_or_default();
                match self.resolve(module, name) {
                    TypeLookup::Found(id) => self.declaration_events(&id, visiting),
                    TypeLookup::Unknown(_) => ScopedTypeEvents::default(),
                }
            }
            TSType::TSImportType(import) => {
                let Some(qualifier) = &import.qualifier else {
                    return ScopedTypeEvents::default();
                };
                let name = source
                    .get(qualifier.span().start as usize..qualifier.span().end as usize)
                    .unwrap_or_default();
                match self.resolve_import(module, &import.source.value, name) {
                    TypeLookup::Found(id) => self.declaration_events(&id, visiting),
                    TypeLookup::Unknown(_) => ScopedTypeEvents::default(),
                }
            }
            _ => ScopedTypeEvents::default(),
        }
    }

    fn declaration_events(
        &self,
        id: &TypeDeclarationId,
        visiting: &mut FxHashSet<TypeDeclarationId>,
    ) -> ScopedTypeEvents {
        if !visiting.insert(id.clone()) {
            return ScopedTypeEvents::default();
        }
        let Some(declaration) = self.declaration(id) else {
            visiting.remove(id);
            return ScopedTypeEvents::default();
        };
        let mut output = ScopedTypeEvents {
            complete: true,
            ..ScopedTypeEvents::default()
        };
        if declaration.kind == TypeDeclarationKind::Interface {
            for base in &declaration.extends {
                combine_into(&mut output, self.events_in(&id.module, base, visiting));
            }
        }
        combine_into(
            &mut output,
            self.events_in(&id.module, &declaration.body, visiting),
        );
        visiting.remove(id);
        output
    }
}

fn events_from_members(members: &[TSSignature<'_>]) -> ScopedTypeEvents {
    let mut output = ScopedTypeEvents {
        complete: true,
        ..ScopedTypeEvents::default()
    };
    for member in members {
        match member {
            TSSignature::TSCallSignatureDeclaration(signature)
                if signature.this_param.is_none() =>
            {
                output.events.extend(parameter_events(&signature.params));
            }
            TSSignature::TSPropertySignature(property) => {
                push_key(&mut output, &property.key);
            }
            TSSignature::TSMethodSignature(method) => push_key(&mut output, &method.key),
            _ => {}
        }
    }
    deduplicate(&mut output.events);
    output
}

fn push_key(output: &mut ScopedTypeEvents, key: &PropertyKey<'_>) {
    let name = match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::Identifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    };
    if let Some(name) = name {
        output.events.push(CompactString::new(name));
    } else {
        output.complete = false;
    }
}

fn parameter_events(parameters: &FormalParameters<'_>) -> Vec<CompactString> {
    let Some(first) = parameters.items.first() else {
        return Vec::new();
    };
    if !matches!(first.pattern, BindingPattern::BindingIdentifier(_)) {
        return Vec::new();
    }
    let Some(annotation) = &first.type_annotation else {
        return Vec::new();
    };
    let mut ty = &annotation.type_annotation;
    while let TSType::TSParenthesizedType(parenthesized) = ty {
        ty = &parenthesized.type_annotation;
    }
    let types: Vec<_> = match ty {
        TSType::TSUnionType(union) => union.types.iter().collect(),
        ty => vec![ty],
    };
    types.into_iter().filter_map(literal_event).collect()
}

fn literal_event(ty: &TSType<'_>) -> Option<CompactString> {
    let TSType::TSLiteralType(literal) = ty else {
        return None;
    };
    match &literal.literal {
        TSLiteral::StringLiteral(literal) => Some(CompactString::new(literal.value.as_str())),
        TSLiteral::NumericLiteral(literal) => Some(CompactString::new(literal.value.to_string())),
        TSLiteral::BooleanLiteral(literal) => Some(CompactString::new(literal.value.to_string())),
        TSLiteral::BigIntLiteral(literal) => Some(CompactString::new(literal.value.as_str())),
        _ => None,
    }
}

fn combine_all(outputs: impl Iterator<Item = ScopedTypeEvents>) -> ScopedTypeEvents {
    let mut combined = ScopedTypeEvents {
        complete: true,
        ..ScopedTypeEvents::default()
    };
    for output in outputs {
        combine_into(&mut combined, output);
    }
    combined
}

fn combine_into(output: &mut ScopedTypeEvents, addition: ScopedTypeEvents) {
    output.complete &= addition.complete;
    output.events.extend(addition.events);
    deduplicate(&mut output.events);
}

fn deduplicate(events: &mut Vec<CompactString>) {
    let mut names = FxHashSet::default();
    events.retain(|name| names.insert(name.clone()));
}
