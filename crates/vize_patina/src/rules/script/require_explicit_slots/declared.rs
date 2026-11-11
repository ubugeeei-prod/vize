//! What a single script block declares about its slots.

use oxc_ast::ast::{
    CallExpression, Declaration, Expression, Program, PropertyKey, Statement, TSAsExpression,
    TSLiteral, TSSignature, TSType, TSTypeName,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_call_expression, walk_ts_as_expression},
};
use oxc_span::GetSpan;
use vize_croquis::types::ResolvedTypeWorld;
use vize_l0::{CompactString, FxHashSet};

/// The slot contract the script states.
pub(super) enum DeclaredSlots {
    /// No `defineSlots` in this block.
    Absent,
    /// A `defineSlots` whose slot set cannot be enumerated — an imported type
    /// reference, an index signature, a computed key, or a second
    /// `defineSlots` call.
    Unknown,
    /// Every declared slot name, exhaustively.
    Known(FxHashSet<CompactString>),
}

/// A slot name written a second time in one type body.
pub(super) struct DuplicateSlot {
    pub(super) name: CompactString,
    pub(super) start: u32,
    pub(super) end: u32,
    /// The member lives in the other script block.
    pub(super) sibling: bool,
}

/// Everything the script half of the rule needs from one block.
pub(super) struct ScriptSlots {
    /// The declared slot contract.
    pub(super) declared: DeclaredSlots,
    /// Repeated names inside one type literal, interface, or alias.
    pub(super) duplicates: Vec<DuplicateSlot>,
}

pub(super) fn collect<'a>(
    program: &'a Program<'a>,
    sibling: Option<&'a Program<'a>>,
    source: &'a str,
    world: Option<&'a ResolvedTypeWorld>,
) -> ScriptSlots {
    let mut programs = vec![program];
    programs.extend(sibling);
    let mut visitor = SlotsVisitor {
        declared: DeclaredSlots::Absent,
        programs,
        source,
        world,
        duplicates: Vec::new(),
    };
    visitor.visit_program(program);
    ScriptSlots {
        declared: visitor.declared,
        duplicates: visitor.duplicates,
    }
}

struct SlotsVisitor<'a> {
    declared: DeclaredSlots,
    programs: Vec<&'a Program<'a>>,
    source: &'a str,
    world: Option<&'a ResolvedTypeWorld>,
    duplicates: Vec<DuplicateSlot>,
}

impl<'a> Visit<'a> for SlotsVisitor<'a> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Expression::Identifier(callee) = &it.callee
            && callee.name == "defineSlots"
        {
            let declared = declared_from_call(
                it,
                &self.programs,
                self.source,
                self.world,
                &mut self.duplicates,
            );
            self.merge(declared);
        }
        walk_call_expression(self, it);
    }

    fn visit_ts_as_expression(&mut self, it: &TSAsExpression<'a>) {
        if let Some(declared) =
            slots_type_declaration(&it.type_annotation, &self.programs, &mut self.duplicates)
        {
            self.merge(declared);
        }
        walk_ts_as_expression(self, it);
    }
}

impl SlotsVisitor<'_> {
    fn merge(&mut self, next: DeclaredSlots) {
        let current = std::mem::replace(&mut self.declared, DeclaredSlots::Absent);
        self.declared = match (current, next) {
            (DeclaredSlots::Absent | DeclaredSlots::Unknown, next) => next,
            (current, DeclaredSlots::Absent | DeclaredSlots::Unknown) => current,
            (DeclaredSlots::Known(mut current), DeclaredSlots::Known(next)) => {
                current.extend(next);
                DeclaredSlots::Known(current)
            }
        };
    }
}

/// Enumerate the slot names of a `defineSlots<{ ... }>()` type argument.
///
/// A call with no type argument declares nothing. Imported references stay
/// [`DeclaredSlots::Unknown`] until the cross-file type world enumerates them.
fn declared_from_call<'a>(
    call: &CallExpression<'a>,
    programs: &[&Program<'a>],
    source: &str,
    world: Option<&ResolvedTypeWorld>,
    duplicates: &mut Vec<DuplicateSlot>,
) -> DeclaredSlots {
    let Some(type_arguments) = call.type_arguments.as_ref() else {
        return DeclaredSlots::Known(FxHashSet::default());
    };
    let [ty] = type_arguments.params.as_slice() else {
        return DeclaredSlots::Unknown;
    };
    let local = declared_from_type(
        ty,
        programs,
        0,
        &mut FxHashSet::default(),
        false,
        duplicates,
    );
    if !matches!(local, DeclaredSlots::Unknown) {
        return local;
    }
    resolved_from_world(world, source, ty).unwrap_or(local)
}

fn slots_type_declaration<'a>(
    ty: &TSType<'a>,
    programs: &[&Program<'a>],
    duplicates: &mut Vec<DuplicateSlot>,
) -> Option<DeclaredSlots> {
    let TSType::TSTypeReference(reference) = ty else {
        return None;
    };
    let TSTypeName::IdentifierReference(identifier) = &reference.type_name else {
        return None;
    };
    if identifier.name != "SlotsType" {
        return None;
    }
    let Some(arguments) = &reference.type_arguments else {
        return Some(DeclaredSlots::Unknown);
    };
    let [ty] = arguments.params.as_slice() else {
        return Some(DeclaredSlots::Unknown);
    };
    Some(declared_from_type(
        ty,
        programs,
        0,
        &mut FxHashSet::default(),
        false,
        duplicates,
    ))
}

fn declared_from_type<'a>(
    ty: &TSType<'a>,
    programs: &[&Program<'a>],
    depth: usize,
    seen: &mut FxHashSet<CompactString>,
    sibling: bool,
    duplicates: &mut Vec<DuplicateSlot>,
) -> DeclaredSlots {
    if depth >= 16 {
        return DeclaredSlots::Unknown;
    }
    match ty {
        TSType::TSParenthesizedType(paren) => declared_from_type(
            &paren.type_annotation,
            programs,
            depth + 1,
            seen,
            sibling,
            duplicates,
        ),
        TSType::TSTypeLiteral(literal) => {
            declared_from_members(&literal.members, sibling, duplicates)
        }
        TSType::TSIntersectionType(intersection) => {
            let mut names = FxHashSet::default();
            for ty in &intersection.types {
                match declared_from_type(ty, programs, depth + 1, seen, sibling, duplicates) {
                    DeclaredSlots::Known(part) => names.extend(part),
                    // An unresolved constituent may declare any rendered
                    // slot. Keeping only the enumerable constituents would
                    // turn a partial view of the intersection into a closed
                    // contract and manufacture undeclared-slot findings.
                    DeclaredSlots::Absent | DeclaredSlots::Unknown => {
                        return DeclaredSlots::Unknown;
                    }
                }
            }
            DeclaredSlots::Known(names)
        }
        TSType::TSTypeReference(reference) => {
            let TSTypeName::IdentifierReference(identifier) = &reference.type_name else {
                return DeclaredSlots::Unknown;
            };
            let name = identifier.name.as_str();
            if name == "Record" {
                return record_keys(reference.type_arguments.as_deref());
            }
            if matches!(name, "Readonly" | "Partial" | "Required")
                && let Some(inner) = reference
                    .type_arguments
                    .as_ref()
                    .and_then(|arguments| arguments.params.first())
            {
                let inner =
                    declared_from_type(inner, programs, depth + 1, seen, sibling, duplicates);
                if matches!(inner, DeclaredSlots::Known(_)) {
                    return inner;
                }
                return DeclaredSlots::Unknown;
            }
            declared_from_named_type(name, programs, depth + 1, seen, duplicates)
        }
        _ => DeclaredSlots::Unknown,
    }
}

fn declared_from_named_type<'a>(
    name: &str,
    programs: &[&Program<'a>],
    depth: usize,
    seen: &mut FxHashSet<CompactString>,
    duplicates: &mut Vec<DuplicateSlot>,
) -> DeclaredSlots {
    let owned = CompactString::new(name);
    if depth >= 16 || !seen.insert(owned.clone()) {
        return DeclaredSlots::Unknown;
    }
    let declared = programs
        .iter()
        .enumerate()
        .find_map(|(index, program)| {
            let sibling = index > 0;
            program
                .body
                .iter()
                .filter_map(statement_declaration)
                .find_map(|declaration| match declaration {
                    Declaration::TSInterfaceDeclaration(interface) if interface.id.name == name => {
                        Some(declared_from_members(
                            &interface.body.body,
                            sibling,
                            duplicates,
                        ))
                    }
                    Declaration::TSTypeAliasDeclaration(alias) if alias.id.name == name => {
                        Some(declared_from_type(
                            &alias.type_annotation,
                            programs,
                            depth + 1,
                            seen,
                            sibling,
                            duplicates,
                        ))
                    }
                    _ => None,
                })
        })
        .unwrap_or(DeclaredSlots::Unknown);
    seen.remove(&owned);
    declared
}

/// `Record<"default" | "footer", ...>` enumerates the literal keys.
fn record_keys(
    arguments: Option<&oxc_ast::ast::TSTypeParameterInstantiation<'_>>,
) -> DeclaredSlots {
    let Some(keys) = arguments.and_then(|arguments| arguments.params.first()) else {
        return DeclaredSlots::Unknown;
    };
    let mut names = FxHashSet::default();
    if string_literal_keys(keys, &mut names) {
        DeclaredSlots::Known(names)
    } else {
        DeclaredSlots::Unknown
    }
}

fn string_literal_keys(ty: &TSType<'_>, names: &mut FxHashSet<CompactString>) -> bool {
    match ty {
        TSType::TSLiteralType(literal) => match &literal.literal {
            TSLiteral::StringLiteral(string) => {
                names.insert(CompactString::new(string.value.as_str()));
                true
            }
            _ => false,
        },
        TSType::TSUnionType(union) => union.types.iter().all(|ty| string_literal_keys(ty, names)),
        TSType::TSParenthesizedType(paren) => string_literal_keys(&paren.type_annotation, names),
        _ => false,
    }
}

fn resolved_from_world(
    world: Option<&ResolvedTypeWorld>,
    source: &str,
    ty: &TSType<'_>,
) -> Option<DeclaredSlots> {
    let world = world?;
    let text = source.get(ty.span().start as usize..ty.span().end as usize)?;
    let resolved = world.resolve_properties(text);
    if !resolved.complete {
        return None;
    }
    let names = resolved
        .properties
        .into_iter()
        .map(|property| property.name)
        .collect();
    Some(DeclaredSlots::Known(names))
}

fn statement_declaration<'a>(statement: &'a Statement<'a>) -> Option<&'a Declaration<'a>> {
    match statement {
        Statement::ExportNamedDeclaration(export) => export.declaration.as_ref(),
        _ => statement.as_declaration(),
    }
}

fn declared_from_members(
    members: &[TSSignature<'_>],
    sibling: bool,
    duplicates: &mut Vec<DuplicateSlot>,
) -> DeclaredSlots {
    let mut names = FxHashSet::default();
    for member in members {
        let key = match member {
            TSSignature::TSPropertySignature(property) if !property.computed => &property.key,
            TSSignature::TSMethodSignature(method) if !method.computed => &method.key,
            // Index signatures and unresolved/computed members contribute no
            // enumerable name, but do not erase explicit siblings.
            _ => continue,
        };
        let Some(name) = property_key_name(key) else {
            continue;
        };
        let owned = CompactString::new(name);
        if !names.insert(owned.clone()) {
            let span = key.span();
            duplicates.push(DuplicateSlot {
                name: owned,
                start: span.start,
                end: span.end,
                sibling,
            });
        }
    }
    DeclaredSlots::Known(names)
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}
