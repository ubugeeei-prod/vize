//! Runtime type of object-like prop types that may be callable.
//!
//! Mirrors `inferRuntimeType` in `@vue/compiler-sfc`: each member of a type
//! literal or interface body contributes `Function` for call and construct
//! signatures and `Object` for anything else, in member order, so a callable
//! interface such as a chart scale (`(value) => number` plus `range`) accepts
//! functions at runtime instead of failing the `Object` prop check.

use super::ast_resolve::wrap_type_alias_source;
use oxc_allocator::Allocator;
use oxc_ast::ast::{Statement, TSSignature, TSType, TSTypeLiteral};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{String, ToCompactString};

/// Runtime constructors for a type literal, in member order.
pub(super) fn type_literal_runtime_type(literal: &TSTypeLiteral<'_>) -> String {
    let mut kinds = RuntimeKinds::default();
    kinds.push_members(literal);
    kinds.finish()
}

/// Runtime constructors for an object-like type written as source text: a
/// type literal, or an interface body (possibly `Base & { ... }` when it
/// extends other interfaces). Anything that is not a literal member list
/// stays `Object`.
pub(super) fn object_type_source_runtime_type(source: &str) -> String {
    // Call and construct signatures always contain a parameter list.
    if !source.contains('(') {
        return "Object".to_compact_string();
    }
    let wrapped = wrap_type_alias_source(source);
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &wrapped, SourceType::ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return "Object".to_compact_string();
    }
    let Some(Statement::TSTypeAliasDeclaration(alias)) = parsed.program.body.first() else {
        return "Object".to_compact_string();
    };
    let mut kinds = RuntimeKinds::default();
    match &alias.type_annotation {
        TSType::TSTypeLiteral(literal) => kinds.push_members(literal),
        TSType::TSIntersectionType(intersection) => {
            for part in &intersection.types {
                match part {
                    TSType::TSTypeLiteral(literal) => kinds.push_members(literal),
                    _ => kinds.object = true,
                }
            }
        }
        _ => kinds.object = true,
    }
    kinds.finish()
}

#[derive(Default)]
struct RuntimeKinds {
    function: bool,
    object: bool,
    function_first: bool,
}

impl RuntimeKinds {
    fn push_members(&mut self, literal: &TSTypeLiteral<'_>) {
        for member in &literal.members {
            if matches!(
                member,
                TSSignature::TSCallSignatureDeclaration(_)
                    | TSSignature::TSConstructSignatureDeclaration(_)
            ) {
                if !self.function && !self.object {
                    self.function_first = true;
                }
                self.function = true;
            } else {
                self.object = true;
            }
        }
    }

    fn finish(self) -> String {
        match (self.function, self.object) {
            (true, true) if self.function_first => "[Function, Object]".to_compact_string(),
            (true, true) => "[Object, Function]".to_compact_string(),
            (true, false) => "Function".to_compact_string(),
            _ => "Object".to_compact_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::object_type_source_runtime_type;

    #[test]
    fn callable_members_follow_vue_member_order() {
        let cases = [
            (
                "{ (value: number): number; readonly range: number[] }",
                "[Function, Object]",
            ),
            (
                "{ label: string; (value: string): void }",
                "[Object, Function]",
            ),
            ("{ (value: number): number }", "Function"),
            ("{ new (value: number): object }", "Function"),
            ("{ run(): void; label: string }", "Object"),
            ("{ a: number }", "Object"),
            ("{}", "Object"),
            ("Base & { (value: number): number }", "[Object, Function]"),
        ];
        for (source, expected) in cases {
            assert_eq!(
                object_type_source_runtime_type(source).as_str(),
                expected,
                "{source}"
            );
        }
    }
}
