//! Module-scope lifting for `enum` exports of a plain Vue `<script>`.
//!
//! A named value export of a plain `<script>` normally stays inside the
//! synthetic `__setup()` wrapper and reaches the module through
//! `export const E = __vize_plain_script_exports.E`, which carries the value
//! side only. An `enum` also declares a type, so that bridge erases it and a
//! consumer writing `const mode: E = E.Member` gets
//! `TS2749: 'E' refers to a value, but is being used as a type here`. Emitting
//! the declaration itself at module scope keeps both sides.
//!
//! Only an enum whose members are all literal-initialized moves: a computed
//! member may read a setup-scope binding that module scope cannot see, and
//! lifting that one would trade a missing type for a missing name.

use oxc_ast::ast::{Declaration, ExportNamedDeclaration, Expression, TSEnumDeclaration};

/// Span of an `export enum` statement worth lifting, `export` keyword included.
pub(super) fn hoistable_export_span(export: &ExportNamedDeclaration<'_>) -> Option<(u32, u32)> {
    if export.source.is_some() || export.export_kind.is_type() {
        return None;
    }
    let Some(Declaration::TSEnumDeclaration(enumeration)) = export.declaration.as_ref() else {
        return None;
    };
    is_hoistable(enumeration).then_some((export.span.start, export.span.end))
}

/// Whether this enum keeps its meaning once lifted out of `__setup()`.
pub(super) fn is_hoistable(enumeration: &TSEnumDeclaration<'_>) -> bool {
    enumeration.body.members.iter().all(|member| {
        member
            .initializer
            .as_ref()
            .is_none_or(is_constant_literal_expression)
    })
}

fn is_constant_literal_expression(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::StringLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BooleanLiteral(_) => true,
        Expression::TemplateLiteral(template) => template.expressions.is_empty(),
        Expression::UnaryExpression(unary) => is_constant_literal_expression(&unary.argument),
        Expression::ParenthesizedExpression(parenthesized) => {
            is_constant_literal_expression(&parenthesized.expression)
        }
        Expression::BinaryExpression(binary) => {
            is_constant_literal_expression(&binary.left)
                && is_constant_literal_expression(&binary.right)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;
    use oxc_ast::ast::Statement;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    fn hoistable_spans(script: &str) -> Vec<(u32, u32)> {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, script, SourceType::ts().with_module(true)).parse();
        assert!(!parsed.panicked, "fixture script should parse");
        parsed
            .program
            .body
            .iter()
            .filter_map(|statement| match statement {
                Statement::ExportNamedDeclaration(export) => super::hoistable_export_span(export),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn literal_member_enum_export_is_hoistable() {
        let script =
            "export enum DiffDisplayMode {\n  Unified = 'unified',\n  Split = 'split',\n}\n";
        let spans = hoistable_spans(script);

        assert_eq!(spans.len(), 1);
        assert_eq!(
            &script[spans[0].0 as usize..spans[0].1 as usize],
            "export enum DiffDisplayMode {\n  Unified = 'unified',\n  Split = 'split',\n}"
        );
    }

    #[test]
    fn implicit_and_computed_literal_members_stay_hoistable() {
        assert_eq!(
            hoistable_spans("export enum Level { Low, High }\n").len(),
            1
        );
        assert_eq!(
            hoistable_spans("export enum Flag { None = 0, Read = 1 << 1 }\n").len(),
            1
        );
    }

    #[test]
    fn enum_member_reading_a_setup_scope_binding_is_not_hoistable() {
        // `base` is declared in the plain <script> body, which stays inside
        // `__setup()`; module scope could not resolve it.
        assert!(hoistable_spans("const base = 1;\nexport enum Level { Low = base }\n").is_empty());
    }

    #[test]
    fn non_enum_and_type_only_exports_are_not_hoistable() {
        assert!(hoistable_spans("export const ready = true;\n").is_empty());
        assert!(hoistable_spans("enum Level { Low }\nexport type { Level };\n").is_empty());
    }
}
