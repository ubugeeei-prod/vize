//! What a single script block declares about its slots.

use oxc_ast::ast::{CallExpression, Expression, Program, PropertyKey, TSSignature, TSType};
use oxc_ast_visit::{
    Visit,
    walk::{walk_call_expression, walk_ts_type},
};
use oxc_span::Span;
use vize_s0::{CompactString, FxHashSet};

/// The slot contract the script states.
pub(super) enum DeclaredSlots {
    /// No `defineSlots` in this block. The template half stays silent: without
    /// a declaration every `<slot>` would be "undeclared", which is the
    /// `useSlots`-without-`defineSlots` case the script half already owns.
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
    /// Whether the block contains any TypeScript-specific syntax. Used as a
    /// sound proxy for `lang="ts"` since a script rule cannot read the SFC
    /// `lang` attribute.
    pub(super) has_ts_syntax: bool,
    /// Span of the first `useSlots()` call, if any.
    pub(super) first_use_slots: Option<Span>,
    /// The declared slot contract.
    pub(super) declared: DeclaredSlots,
}

pub(super) fn collect(program: &Program<'_>) -> ScriptSlots {
    let mut visitor = SlotsVisitor {
        has_ts_syntax: false,
        first_use_slots: None,
        declared: DeclaredSlots::Absent,
    };
    visitor.visit_program(program);
    ScriptSlots {
        has_ts_syntax: visitor.has_ts_syntax,
        first_use_slots: visitor.first_use_slots,
        declared: visitor.declared,
    }
}

struct SlotsVisitor {
    has_ts_syntax: bool,
    first_use_slots: Option<Span>,
    declared: DeclaredSlots,
}

impl<'a> Visit<'a> for SlotsVisitor {
    fn visit_ts_type(&mut self, it: &TSType<'a>) {
        // Any TypeScript type position is a definitive TS-syntax signal.
        self.has_ts_syntax = true;
        walk_ts_type(self, it);
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Expression::Identifier(callee) = &it.callee {
            match callee.name.as_str() {
                "defineSlots" => {
                    // A second `defineSlots` is invalid Vue; which one wins is
                    // not ours to guess, so the set stops being known.
                    self.declared = match self.declared {
                        DeclaredSlots::Absent => declared_from_call(it),
                        _ => DeclaredSlots::Unknown,
                    };
                    // A `defineSlots<T>()` call carries a type argument, which is
                    // itself TS syntax; mark it so a block whose only TS token is
                    // the slots declaration is still recognised as TypeScript.
                    if it.type_arguments.is_some() {
                        self.has_ts_syntax = true;
                    }
                }
                "useSlots" if self.first_use_slots.is_none() => {
                    self.first_use_slots = Some(it.span);
                }
                _ => {}
            }
        }
        walk_call_expression(self, it);
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
    let [TSType::TSTypeLiteral(literal)] = type_arguments.params.as_slice() else {
        return DeclaredSlots::Unknown;
    };
    let mut names = FxHashSet::default();
    for member in &literal.members {
        let key = match member {
            TSSignature::TSPropertySignature(property) if !property.computed => &property.key,
            TSSignature::TSMethodSignature(method) if !method.computed => &method.key,
            _ => return DeclaredSlots::Unknown,
        };
        let Some(name) = property_key_name(key) else {
            return DeclaredSlots::Unknown;
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
