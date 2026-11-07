//! What a single script block declares about its slots.

use oxc_ast::ast::{
    CallExpression, Expression, Program, PropertyKey, TSAsExpression, TSSignature, TSType,
    TSTypeName,
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
    /// A `defineSlots` whose slot set cannot be enumerated — a bare type
    /// reference (`defineSlots<Slots>()`), an index signature, a computed key,
    /// or a second `defineSlots` call. The set is not fully known, so checking
    /// a `<slot>` against it could report a slot that *is* declared.
    Unknown,
    /// Every declared slot name, exhaustively.
    Known(FxHashSet<CompactString>),
}

/// Everything the script half of the rule needs from one block.
pub(super) struct ScriptSlots {
    /// The declared slot contract.
    pub(super) declared: DeclaredSlots,
}

pub(super) fn collect(program: &Program<'_>) -> ScriptSlots {
    let mut visitor = SlotsVisitor {
        declared: DeclaredSlots::Absent,
    };
    visitor.visit_program(program);
    ScriptSlots {
        declared: visitor.declared,
    }
}

struct SlotsVisitor {
    declared: DeclaredSlots,
}

impl<'a> Visit<'a> for SlotsVisitor {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Expression::Identifier(callee) = &it.callee
            && callee.name == "defineSlots"
        {
            self.merge(declared_from_call(it));
        }
        walk_call_expression(self, it);
    }

    fn visit_ts_as_expression(&mut self, it: &TSAsExpression<'a>) {
        if let Some(declared) = slots_type_declaration(&it.type_annotation) {
            self.merge(declared);
        }
        walk_ts_as_expression(self, it);
    }
}

impl SlotsVisitor {
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
/// Only the single-type-literal form is enumerable. Anything else — no type
/// argument at all (`defineSlots()`), several, a type reference, or a member
/// this cannot name (an index signature, a computed key) — yields
/// [`DeclaredSlots::Unknown`] so the template half reports nothing rather than
/// flagging a slot the type does declare.
fn declared_from_call(call: &CallExpression<'_>) -> DeclaredSlots {
    let Some(type_arguments) = call.type_arguments.as_ref() else {
        return DeclaredSlots::Unknown;
    };
    let [ty] = type_arguments.params.as_slice() else {
        return DeclaredSlots::Unknown;
    };
    declared_from_type(ty)
}

fn slots_type_declaration(ty: &TSType<'_>) -> Option<DeclaredSlots> {
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
    Some(declared_from_type(ty))
}

fn declared_from_type(ty: &TSType<'_>) -> DeclaredSlots {
    match ty {
        TSType::TSTypeLiteral(literal) => declared_from_members(&literal.members),
        TSType::TSIntersectionType(intersection) => {
            let mut names = FxHashSet::default();
            for ty in &intersection.types {
                if let DeclaredSlots::Known(part) = declared_from_type(ty) {
                    names.extend(part);
                }
            }
            DeclaredSlots::Known(names)
        }
        _ => DeclaredSlots::Unknown,
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
