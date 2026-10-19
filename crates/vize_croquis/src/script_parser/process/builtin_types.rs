//! Restricted primitive proofs from existing AST nodes, never source strings.

use super::super::ScriptParseResult;
use crate::reactivity::ReactiveKind;
use crate::scope::ScopeKind;
use crate::types::builtin_reactive::{BuiltinReactiveType, Primitive};
use oxc_ast::ast::{
    BinaryOperator, BindingPattern, Expression, Statement, VariableDeclarationKind,
    VariableDeclarator,
};
use vize_carton::CompactString;

/// Add editor facts only to demanded top-level variables in the existing walk.
pub(in crate::script_parser) fn process_statement_with_builtin(
    result: &mut ScriptParseResult,
    stmt: &Statement<'_>,
    source: &str,
) {
    let Statement::VariableDeclaration(decl) = stmt else {
        super::statements::process_statement(result, stmt, source);
        return;
    };
    super::template_components::collect(result, stmt);
    super::super::extract::invalidate_default_objects(result, stmt);
    for declarator in decl.declarations.iter() {
        result.refuse_declarator_type_reads(declarator);
        super::super::extract::invalidate_default_expression(result, declarator.init.as_ref());
        record(result, declarator, decl.kind);
        super::macros::process_variable_declarator(result, declarator, decl.kind, source);
    }
}

pub(super) fn record(
    result: &mut ScriptParseResult,
    declarator: &VariableDeclarator<'_>,
    kind: VariableDeclarationKind,
) {
    if result.scopes.current_scope().kind != ScopeKind::ScriptSetup {
        return;
    }
    let BindingPattern::BindingIdentifier(id) = &declarator.id else {
        return;
    };
    let name = id.name.as_str();
    result.types.clear_builtin_reactive_type(name);
    if kind != VariableDeclarationKind::Const || declarator.type_annotation.is_some() {
        return;
    }
    let Some(Expression::CallExpression(call)) = &declarator.init else {
        return;
    };
    if call.optional || call.type_arguments.is_some() || call.arguments.len() != 1 {
        return;
    }
    let Expression::Identifier(callee) = &call.callee else {
        return;
    };
    let callee = callee.name.as_str();
    if !is_vue_import(result, callee) {
        return;
    }
    let Some(argument) = call.arguments.first().and_then(|arg| arg.as_expression()) else {
        return;
    };
    let (kind, value) = match result.types.definitions().imported_type_export(callee) {
        Some("ref" | "shallowRef") => {
            let Some(value) = primitive_literal(argument) else {
                return;
            };
            (ReactiveKind::Ref, value)
        }
        Some("computed") => {
            let Expression::ArrowFunctionExpression(arrow) = argument else {
                return;
            };
            if arrow.r#async
                || !arrow.expression
                || !arrow.params.items.is_empty()
                || arrow.params.rest.is_some()
                || arrow.return_type.is_some()
                || arrow.type_parameters.is_some()
                || arrow.body.statements.len() != 1
            {
                return;
            }
            let Some(Statement::ExpressionStatement(body)) = arrow.body.statements.first() else {
                return;
            };
            if !numeric_expression(result, &body.expression) {
                return;
            }
            (ReactiveKind::Computed, Primitive::Number)
        }
        _ => return,
    };
    result
        .types
        .record_builtin_reactive_type(BuiltinReactiveType {
            name: CompactString::new(name),
            span: (id.span.start, id.span.end),
            kind,
            value,
        });
}

fn is_vue_import(result: &ScriptParseResult, name: &str) -> bool {
    result
        .import_sources
        .get(name)
        .is_some_and(|source| source.as_str() == "vue")
        && result.binding_spans.get(name).is_some_and(|(start, end)| {
            result
                .import_statements
                .iter()
                .any(|import| *start >= import.start && *end <= import.end)
        })
}

fn primitive_literal(expression: &Expression<'_>) -> Option<Primitive> {
    match expression {
        Expression::BooleanLiteral(_) => Some(Primitive::Boolean),
        Expression::NumericLiteral(_) => Some(Primitive::Number),
        Expression::StringLiteral(_) => Some(Primitive::String),
        _ => None,
    }
}

fn numeric_expression(result: &ScriptParseResult, expression: &Expression<'_>) -> bool {
    match expression {
        Expression::NumericLiteral(_) => true,
        Expression::StaticMemberExpression(member)
            if !member.optional && member.property.name == "value" =>
        {
            owned_primitive_value(result, &member.object) == Some(Primitive::Number)
        }
        Expression::StaticMemberExpression(member)
            if !member.optional && member.property.name == "length" =>
        {
            let Expression::StaticMemberExpression(value) = &member.object else {
                return false;
            };
            !value.optional
                && value.property.name == "value"
                && owned_primitive_value(result, &value.object) == Some(Primitive::String)
        }
        Expression::BinaryExpression(binary)
            if matches!(
                binary.operator,
                BinaryOperator::Addition
                    | BinaryOperator::Subtraction
                    | BinaryOperator::Multiplication
                    | BinaryOperator::Division
                    | BinaryOperator::Remainder
                    | BinaryOperator::Exponential
            ) =>
        {
            numeric_expression(result, &binary.left) && numeric_expression(result, &binary.right)
        }
        _ => false,
    }
}

fn owned_primitive_value(result: &ScriptParseResult, object: &Expression<'_>) -> Option<Primitive> {
    let Expression::Identifier(object) = object else {
        return None;
    };
    let fact = result.types.builtin_reactive_type(object.name.as_str())?;
    (result.binding_spans.get(object.name.as_str()) == Some(&fact.span)).then_some(fact.value)
}

#[cfg(test)]
mod tests {
    fn parse_builtin(source: &str) -> crate::script_parser::ScriptParseResult {
        crate::script_parser::parse_script_setup_for_unused::<true>(
            source, None, false, false, false, false,
        )
    }

    #[test]
    fn builtin_hints_and_authored_occurrences_share_the_existing_walk() {
        let source = "import { ref } from 'vue'; const count = ref(0); count;";
        let mut ordinary = crate::Drawer::with_options(crate::DrawerOptions::for_lint());
        ordinary.draw_script_setup_with_builtin_types(source);
        let ordinary = ordinary.finish();
        let mut demanded = crate::Drawer::with_options(crate::DrawerOptions::for_lint())
            .with_binding_occurrences();
        demanded.draw_script_setup_with_builtin_types(source);
        let (demanded, packet) = demanded.finish_with_binding_occurrences();
        assert_eq!(
            serde_json::to_value(demanded.semantic_snapshot()).unwrap(),
            serde_json::to_value(ordinary.semantic_snapshot()).unwrap()
        );
        let fact = demanded.types.builtin_reactive_types().next().unwrap();
        assert_eq!((fact.name(), fact.value_type()), ("count", "number"));
        assert_eq!(
            fact.span(),
            ordinary
                .types
                .builtin_reactive_types()
                .next()
                .unwrap()
                .span()
        );
        let packet = packet.unwrap();
        let binding = packet
            .bindings()
            .find(|binding| binding.name == "count")
            .unwrap();
        let reads: Vec<_> = packet
            .occurrences()
            .iter()
            .filter(|occurrence| occurrence.binding == binding.identity)
            .map(|occurrence| (occurrence.block, occurrence.start, occurrence.end))
            .collect();
        let start = source.rfind("count;").unwrap() as u32;
        assert_eq!(
            reads,
            vec![(
                crate::binding_occurrences::OccurrenceBlock::Script,
                start,
                start + 5
            )]
        );

        let malformed = format!("{source} const broken = ;");
        let mut drawer = crate::Drawer::with_options(crate::DrawerOptions::for_lint())
            .with_binding_occurrences();
        drawer.draw_script_setup_with_builtin_types(&malformed);
        let (croquis, packet) = drawer.finish_with_binding_occurrences();
        assert!(packet.is_none());
        assert_eq!(croquis.types.builtin_reactive_types().count(), 0);
    }

    fn facts(source: &str) -> Vec<(String, String, String, String)> {
        let parsed = parse_builtin(source);
        let mut facts: Vec<_> = parsed
            .types
            .builtin_reactive_types()
            .map(|fact| {
                let (start, end) = fact.span();
                (
                    String::from(fact.name()),
                    format!("{:?}", fact.kind()),
                    String::from(fact.value_type()),
                    String::from(source.get(start as usize..end as usize).unwrap()),
                )
            })
            .collect();
        facts.sort();
        facts
    }

    #[test]
    fn builtin_aliases_and_numeric_operators_have_complete_source_facts() {
        let source = "import { ref as r, computed as c, shallowRef } from 'vue';\nconst café = r(0);\nconst active = r(false);\nconst label = shallowRef('hello');\nconst doubled = c(() => café.value * 2);\nconst combined = c(() => doubled.value + café.value / 2 - 1 ** 2 % 3);\n";
        assert_eq!(
            facts(source),
            vec![
                (
                    "active".into(),
                    "Ref".into(),
                    "boolean".into(),
                    "active".into()
                ),
                ("café".into(), "Ref".into(), "number".into(), "café".into()),
                (
                    "combined".into(),
                    "Computed".into(),
                    "number".into(),
                    "combined".into()
                ),
                (
                    "doubled".into(),
                    "Computed".into(),
                    "number".into(),
                    "doubled".into()
                ),
                (
                    "label".into(),
                    "Ref".into(),
                    "string".into(),
                    "label".into()
                ),
            ]
        );
    }

    #[test]
    fn unknown_or_non_builtin_expressions_have_no_source_type_fact() {
        for source in [
            "import { ref, computed } from './fake'; const n = ref(0); const x = computed(() => n.value * 2);",
            "import type { ref } from 'vue'; const n = ref(0);",
            "import { ref } from 'vue'; let n = ref(0);",
            "import { ref } from 'vue'; const n: unknown = ref(0);",
            "import { ref } from 'vue'; const n = ref<number>(0);",
            "import { ref } from 'vue'; const n = ref(0 as number);",
            "import { ref } from 'vue'; const n = ref(1n);",
            "import { ref } from 'vue'; const n = ref(readCount());",
            "import { computed } from 'vue'; const x = computed(readCount);",
            "import { computed } from 'vue'; const x = computed(() => unknown.value * 2);",
            "import { computed } from 'vue'; const x = computed(() => 1n * 2n);",
            "import { computed } from 'vue'; const x = computed(() => 'a' * 2);",
            "import { computed } from 'vue'; const x = computed(async () => 1 * 2);",
            "import { computed } from 'vue'; const x = computed((n: number) => n * 2);",
            "import { computed } from 'vue'; const x = computed((): number => 1 * 2);",
            "import { computed } from 'vue'; const x = computed(() => { return 1 * 2; });",
            "import { ref } from 'vue'; function f(ref: any) { const n = ref(0); }",
            "import { ref } from 'vue'; function f() { const n = ref(0); }",
            "import { ref } from 'vue'; const ref = fake; const n = ref(0);",
            "import { ref } from 'vue'; const n = ref(0); const broken = ;",
        ] {
            assert_eq!(facts(source), Vec::new(), "whole facts for {source}");
        }
        assert_eq!(
            facts(
                "import { ref, computed } from 'vue'; const n = ref(0); const x = computed(() => n.value.toFixed());"
            ),
            vec![("n".into(), "Ref".into(), "number".into(), "n".into())]
        );
    }

    #[test]
    fn later_binding_ownership_and_fresh_parse_lifecycle_do_not_reuse_stale_facts() {
        assert_eq!(
            facts(
                "import { ref, computed } from 'vue'; const n = ref(0); let n = dynamic(); const x = computed(() => n.value * 2);"
            ),
            Vec::new()
        );
        let source = "import { ref } from 'vue'; const count = ref(0);";
        let parsed = parse_builtin(source);
        let mut croquis = crate::Croquis::new();
        parsed.apply_to_croquis(&mut croquis);
        let before = croquis
            .types
            .builtin_reactive_types()
            .next()
            .unwrap()
            .span();
        croquis.shift_script_offsets(17);
        let fact = croquis.types.builtin_reactive_types().next().unwrap();
        assert_eq!(fact.span(), (before.0 + 17, before.1 + 17));
        assert_eq!(croquis.binding_spans.get("count"), Some(&fact.span()));
        assert_eq!(
            facts("import { ref } from 'vue'; const count = ref(dynamic());"),
            Vec::new()
        );
        let debug = format!("{:?}", croquis.types);
        assert!(
            !debug.contains("builtin_reactive"),
            "compatibility dump retains its old shape"
        );
    }

    #[test]
    fn owned_primitive_string_length_has_numeric_type_and_unknown_lengths_are_refused() {
        assert_eq!(
            facts(
                "import { ref, computed } from 'vue'; const message = ref('hello'); const doubled = computed(() => message.value.length * 2);"
            ),
            vec![
                (
                    "doubled".into(),
                    "Computed".into(),
                    "number".into(),
                    "doubled".into()
                ),
                (
                    "message".into(),
                    "Ref".into(),
                    "string".into(),
                    "message".into()
                ),
            ]
        );
        for source in [
            "import { ref, computed } from 'vue'; const message = ref(readMessage()); const x = computed(() => message.value.length * 2);",
            "import { ref, computed } from 'vue'; const message = ref({ length: 1 }); const x = computed(() => message.value.length * 2);",
            "import { ref, computed } from 'vue'; const message = ref(['a']); const x = computed(() => message.value.length * 2);",
            "import { computed } from 'vue'; const x = computed(() => unknown.value.length * 2);",
            "import { ref, computed } from './fake'; const message = ref('hello'); const x = computed(() => message.value.length * 2);",
            "import { ref, computed } from 'vue'; const message = ref('hello'); let message = dynamic(); const x = computed(() => message.value.length * 2);",
        ] {
            assert_eq!(
                facts(source),
                Vec::new(),
                "whole unknown facts for {source}"
            );
        }
        for expression in [
            "message.length * 2",
            "message.value.size * 2",
            "message.value.length() * 2",
            "message.value?.length * 2",
            "message.value.length + unknown.value",
            "(message.value as string).length * 2",
        ] {
            assert_eq!(
                facts(&format!(
                    "import {{ ref, computed }} from 'vue'; const message = ref('hello'); const x = computed(() => {expression});"
                )),
                vec![(
                    "message".into(),
                    "Ref".into(),
                    "string".into(),
                    "message".into()
                )],
                "whole facts for {expression}"
            );
        }
        assert_eq!(
            facts(
                "import { ref, computed } from 'vue'; const message = ref(0); const x = computed(() => message.value.length * 2);"
            ),
            vec![(
                "message".into(),
                "Ref".into(),
                "number".into(),
                "message".into()
            )]
        );
    }
}
