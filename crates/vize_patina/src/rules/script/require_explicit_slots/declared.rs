//! What a single script block declares about its slots.

use oxc_ast::ast::{
    CallExpression, Declaration, Expression, Program, PropertyKey, Statement, TSAsExpression,
    TSSignature, TSType, TSTypeName,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_call_expression, walk_ts_as_expression},
};
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

/// Everything the script half of the rule needs from one block.
pub(super) struct ScriptSlots {
    /// The declared slot contract.
    pub(super) declared: DeclaredSlots,
}

pub(super) fn collect<'a>(
    program: &'a Program<'a>,
    sibling: Option<&'a Program<'a>>,
) -> ScriptSlots {
    let mut programs = vec![program];
    programs.extend(sibling);
    let mut visitor = SlotsVisitor {
        declared: DeclaredSlots::Absent,
        programs,
    };
    visitor.visit_program(program);
    ScriptSlots {
        declared: visitor.declared,
    }
}

struct SlotsVisitor<'a> {
    declared: DeclaredSlots,
    programs: Vec<&'a Program<'a>>,
}

impl<'a> Visit<'a> for SlotsVisitor<'a> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Expression::Identifier(callee) = &it.callee
            && callee.name == "defineSlots"
        {
            let declared = declared_from_call(it, &self.programs);
            self.merge(declared);
        }
        walk_call_expression(self, it);
    }

    fn visit_ts_as_expression(&mut self, it: &TSAsExpression<'a>) {
        if let Some(declared) = slots_type_declaration(&it.type_annotation, &self.programs) {
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
/// Inline literals and same-block interfaces/type aliases are enumerable.
/// Imported references and other unsupported forms yield
/// [`DeclaredSlots::Unknown`].
fn declared_from_call(call: &CallExpression<'_>, programs: &[&Program<'_>]) -> DeclaredSlots {
    let Some(type_arguments) = call.type_arguments.as_ref() else {
        return DeclaredSlots::Unknown;
    };
    let [ty] = type_arguments.params.as_slice() else {
        return DeclaredSlots::Unknown;
    };
    declared_from_type(ty, programs, 0, &mut FxHashSet::default())
}

fn slots_type_declaration(ty: &TSType<'_>, programs: &[&Program<'_>]) -> Option<DeclaredSlots> {
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
    ))
}

fn declared_from_type(
    ty: &TSType<'_>,
    programs: &[&Program<'_>],
    depth: usize,
    seen: &mut FxHashSet<CompactString>,
) -> DeclaredSlots {
    if depth >= 16 {
        return DeclaredSlots::Unknown;
    }
    match ty {
        TSType::TSTypeLiteral(literal) => declared_from_members(&literal.members),
        TSType::TSIntersectionType(intersection) => {
            let mut names = FxHashSet::default();
            for ty in &intersection.types {
                if let DeclaredSlots::Known(part) =
                    declared_from_type(ty, programs, depth + 1, seen)
                {
                    names.extend(part);
                }
            }
            DeclaredSlots::Known(names)
        }
        TSType::TSTypeReference(reference) => {
            let TSTypeName::IdentifierReference(identifier) = &reference.type_name else {
                return DeclaredSlots::Unknown;
            };
            declared_from_named_type(identifier.name.as_str(), programs, depth + 1, seen)
        }
        _ => DeclaredSlots::Unknown,
    }
}

fn declared_from_named_type(
    name: &str,
    programs: &[&Program<'_>],
    depth: usize,
    seen: &mut FxHashSet<CompactString>,
) -> DeclaredSlots {
    let owned = CompactString::new(name);
    if depth >= 16 || !seen.insert(owned.clone()) {
        return DeclaredSlots::Unknown;
    }
    let declared = programs
        .iter()
        .find_map(|program| {
            program
                .body
                .iter()
                .filter_map(statement_declaration)
                .find_map(|declaration| match declaration {
                    Declaration::TSInterfaceDeclaration(interface) if interface.id.name == name => {
                        Some(declared_from_members(&interface.body.body))
                    }
                    Declaration::TSTypeAliasDeclaration(alias) if alias.id.name == name => Some(
                        declared_from_type(&alias.type_annotation, programs, depth + 1, seen),
                    ),
                    _ => None,
                })
        })
        .unwrap_or(DeclaredSlots::Unknown);
    seen.remove(&owned);
    declared
}

fn statement_declaration<'a>(statement: &'a Statement<'a>) -> Option<&'a Declaration<'a>> {
    match statement {
        Statement::ExportNamedDeclaration(export) => export.declaration.as_ref(),
        _ => statement.as_declaration(),
    }
}

fn declared_from_members(members: &[TSSignature<'_>]) -> DeclaredSlots {
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
        names.insert(CompactString::new(name));
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
