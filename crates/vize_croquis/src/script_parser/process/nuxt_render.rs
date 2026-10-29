//! Closed script certainty for Nuxt render-context analysis.
//!
//! This fold runs inside the existing statement visit. It does not scan source,
//! parse, walk the program again, access files, or change semantic projections.

use oxc_ast::ast::{Expression, ObjectPropertyKind, PropertyKey, Statement};
use vize_carton::CompactString;

use super::super::ScriptParseResult;
use crate::croquis::NuxtPageLayout;

pub(super) fn observe(result: &mut ScriptParseResult, statement: &Statement<'_>) {
    observe_page_layout(result, statement);
    if result.nuxt_static_render_script != Some(true) {
        return;
    }
    let known = match statement {
        Statement::EmptyStatement(_) => true,
        Statement::ExpressionStatement(statement) if !result.is_non_setup_script => {
            static_page_meta(&statement.expression)
        }
        _ => false,
    };
    if !known {
        result.nuxt_static_render_script = Some(false);
    }
}

fn observe_page_layout(result: &mut ScriptParseResult, statement: &Statement<'_>) {
    if result.is_non_setup_script {
        return;
    }
    let Statement::ExpressionStatement(statement) = statement else {
        return;
    };
    let Some(layout) = page_layout(&statement.expression) else {
        return;
    };
    result.nuxt_page_meta_calls = result.nuxt_page_meta_calls.saturating_add(1);
    result.nuxt_page_layout = (result.nuxt_page_meta_calls == 1)
        .then_some(layout)
        .flatten();
}

/// Return `None` for an unrelated expression and `Some(selection)` for a
/// top-level `definePageMeta` call. An unknown/dynamic selection is represented
/// by `Some(None)` and keeps the existing default-layout fallback.
fn page_layout(expression: &Expression<'_>) -> Option<Option<NuxtPageLayout>> {
    let Expression::CallExpression(call) = expression else {
        return None;
    };
    if !matches!(&call.callee, Expression::Identifier(name) if name.name == "definePageMeta") {
        return None;
    }
    if call.optional || call.type_arguments.is_some() || call.arguments.len() != 1 {
        return Some(None);
    }
    let Some(Expression::ObjectExpression(object)) = call
        .arguments
        .first()
        .and_then(|argument| argument.as_expression())
    else {
        return Some(None);
    };
    let mut layout = None;
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            // A spread after an explicit layout may replace it. A later
            // explicit property can establish the final value again.
            layout = None;
            continue;
        };
        if property.computed || property.method {
            // A computed property can alias `layout` and override an earlier
            // literal. Do not guess until a later explicit property wins.
            layout = None;
            continue;
        }
        let key = match &property.key {
            PropertyKey::StaticIdentifier(name) => name.name.as_str(),
            PropertyKey::StringLiteral(name) => name.value.as_str(),
            _ => {
                layout = None;
                continue;
            }
        };
        if key != "layout" {
            continue;
        }
        layout = match &property.value {
            Expression::StringLiteral(value) => Some(NuxtPageLayout::Named(CompactString::new(
                value.value.as_str(),
            ))),
            Expression::BooleanLiteral(value) if !value.value => Some(NuxtPageLayout::Disabled),
            _ => None,
        };
    }
    Some(layout)
}

fn static_page_meta(expression: &Expression<'_>) -> bool {
    let Expression::CallExpression(call) = expression else {
        return false;
    };
    if !matches!(&call.callee, Expression::Identifier(name) if name.name == "definePageMeta")
        || call.optional
        || call.type_arguments.is_some()
        || call.arguments.len() != 1
    {
        return false;
    }
    let Some(Expression::ObjectExpression(object)) = call
        .arguments
        .first()
        .and_then(|argument| argument.as_expression())
    else {
        return false;
    };
    let mut seen = 0_u8;
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return false;
        };
        if property.computed || property.method {
            return false;
        }
        let key = match &property.key {
            PropertyKey::StaticIdentifier(name) => name.name.as_str(),
            PropertyKey::StringLiteral(name) => name.value.as_str(),
            _ => return false,
        };
        let (bit, known) = match key {
            "layout" => (
                1,
                matches!(&property.value, Expression::StringLiteral(_))
                    || matches!(&property.value, Expression::BooleanLiteral(value) if !value.value),
            ),
            "pageTransition" => (
                2,
                matches!(&property.value, Expression::BooleanLiteral(value) if !value.value),
            ),
            "layoutTransition" => (
                4,
                matches!(&property.value, Expression::BooleanLiteral(value) if !value.value),
            ),
            _ => return false,
        };
        if !known || seen & bit != 0 {
            return false;
        }
        seen |= bit;
    }
    true
}

#[cfg(test)]
mod tests {
    use crate::script_parser::parse_script_setup;

    #[test]
    fn records_literal_layout_amid_ordinary_page_setup() {
        let analysis = parse_script_setup(
            r#"import { inject } from "vue";
definePageMeta({ layout: "bare" });
const busy = inject("busy")"#,
        );
        assert!(matches!(
            analysis.nuxt_page_layout,
            Some(crate::croquis::NuxtPageLayout::Named(ref name)) if name == "bare"
        ));
        assert_ne!(analysis.nuxt_static_render_script, Some(true));
    }

    #[test]
    fn records_disabled_layout_and_keeps_default_or_dynamic_as_fallback() {
        let disabled = parse_script_setup("definePageMeta({ layout: false })");
        assert!(matches!(
            disabled.nuxt_page_layout,
            Some(crate::croquis::NuxtPageLayout::Disabled)
        ));
        for source in [
            "",
            "definePageMeta({ title: 'login' })",
            "definePageMeta({ layout })",
            "definePageMeta({ layout: 'bare', ...preset })",
            "definePageMeta({ layout: 'bare', [key]: value })",
            "definePageMeta({ layout: 'bare' }); definePageMeta({ title: 'login' })",
            "const definePageMeta = installRuntime; definePageMeta({ layout: 'bare' })",
        ] {
            assert!(
                parse_script_setup(source).nuxt_page_layout.is_none(),
                "{source}"
            );
        }
        assert!(matches!(
            parse_script_setup("definePageMeta({ ...preset, layout: 'bare' })")
                .nuxt_page_layout,
            Some(crate::croquis::NuxtPageLayout::Named(ref name)) if name == "bare"
        ));
    }

    #[test]
    fn admits_only_closed_static_page_meta_statements() {
        for source in [
            "",
            ";;",
            r#"definePageMeta({ layout: "bare" })"#,
            r#"definePageMeta({ layout: false, pageTransition: false, layoutTransition: false })"#,
        ] {
            assert_eq!(
                parse_script_setup(source).nuxt_static_render_script,
                Some(true),
                "{source}"
            );
        }
    }

    #[test]
    fn runtime_transition_spread_shadowing_and_original_errors_are_unknown() {
        for source in [
            r#"definePageMeta({ ...preset, layout: "bare" })"#,
            r#"definePageMeta({ layout: "bare", pageTransition: { mode: "in-out" } })"#,
            r#"definePageMeta({ layoutTransition: true })"#,
            r#"definePageMeta({ ["layout"]: "bare" })"#,
            r#"definePageMeta({ get layout() { return "bare" } })"#,
            r#"definePageMeta({ layout: "bare", layout: "default" })"#,
            "definePageMeta(meta)",
            "function definePageMeta() {}; definePageMeta({})",
            "const route = useRoute(); route.meta.pageTransition = true",
            "definePageMeta({}); const =",
            "definePageMeta({}); value.",
        ] {
            assert_ne!(
                parse_script_setup(source).nuxt_static_render_script,
                Some(true),
                "{source}"
            );
        }
    }
}

#[cfg(test)]
mod caller_owned_tests {
    #[test]
    fn an_already_parsed_program_does_not_assert_original_parse_admission() {
        let allocator = oxc_allocator::Allocator::default();
        let source = "definePageMeta({})";
        let parsed =
            oxc_parser::Parser::new(&allocator, source, oxc_span::SourceType::ts()).parse();
        assert!(parsed.diagnostics.is_empty());
        let mut drawer = crate::Drawer::with_options(crate::DrawerOptions::full());
        drawer.draw_script_setup_program(&parsed.program, source, None);
        assert_ne!(drawer.finish().nuxt_static_render_script, Some(true));
    }
}

#[cfg(test)]
mod specialized_dispatch_tests {
    #[test]
    fn builtin_variable_fast_path_never_asserts_common_statement_fold_coverage() {
        for source in [
            "",
            "const router = useRouter()",
            "const definePageMeta = installRuntime; definePageMeta({})",
        ] {
            let mut drawer = crate::Drawer::with_options(crate::DrawerOptions::full());
            drawer.draw_script_setup_with_builtin_types(source);
            assert_ne!(
                drawer.finish().nuxt_static_render_script,
                Some(true),
                "{source}"
            );
        }
    }
}
