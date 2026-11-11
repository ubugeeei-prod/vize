//! script/valid-next-tick
//!
//! Enforce valid `nextTick` calls.
//!
//! `nextTick()` (the function imported from `vue`), `Vue.nextTick()` and
//! `this.$nextTick()` either take one callback or return a Promise that
//! resolves after the next DOM flush. Port of
//! [`vue/valid-next-tick`](https://eslint.vuejs.org/rules/valid-next-tick.html),
//! applied to `<script setup>` and plain scripts as well as component options.
//! For compatibility, the `Vue.nextTick` spelling is recognized syntactically,
//! even when `Vue` is a local binding.
//! It reports:
//! - a call without a callback whose Promise is not awaited, returned,
//!   assigned, chained with `.then()` or passed to `Promise.*([...])`
//!   (`.catch()` and `.finally()` do not wait for the flush);
//! - a call with more than one argument;
//! - a call with a callback whose Promise is also awaited or chained;
//! - `nextTick` used as a value where it is neither called nor passed on.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! import { nextTick } from 'vue'
//!
//! nextTick()               // result ignored
//! nextTick().catch(fail)   // still not awaited
//! await nextTick(focus)    // callback and await
//! nextTick(focus, other)   // too many arguments
//! await Promise.all([nextTick])
//! ```
//!
//! ### Valid
//! ```ts
//! import { nextTick } from 'vue'
//!
//! await nextTick()
//! nextTick().then(() => focusInput())
//! nextTick(() => focusInput())
//! const tick = () => nextTick()
//! this.$nextTick(() => focusInput())
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::{
    AstKind,
    ast::{Expression, ImportDeclarationSpecifier, Program, Statement},
};
use oxc_semantic::{AstNodes, Semantic, SemanticBuilder};
use oxc_span::{GetSpan, Span};
use oxc_syntax::{node::NodeId, symbol::SymbolId};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/valid-next-tick",
    description: "Require the result of a nextTick() call to be awaited, chained, or given a callback",
    default_severity: Severity::Warning,
};

const NOT_AWAITED: &str = "Await the Promise returned by `nextTick()` or pass a callback function.";
const TOO_MANY_ARGUMENTS: &str = "`nextTick()` expects zero or one argument.";
const AWAITED_CALLBACK: &str =
    "Either await the Promise or pass a callback function to `nextTick()`, not both.";
const NOT_CALLED: &str = "`nextTick` is a function; call it instead of using it as a value.";
const HELP: &str = "nextTick() returns a promise that resolves after the DOM updates. \
     `await` it, chain `.then(...)`, or pass a single callback (`nextTick(() => {})`).";

/// Enforce valid `nextTick` calls.
pub struct ValidNextTick;

impl ScriptRule for ValidNextTick {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    #[inline]
    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        if !source.contains("nextTick") {
            return;
        }
        let semantic = SemanticBuilder::new()
            .with_build_nodes(true)
            .build(program)
            .semantic;
        let (named, namespaces) = vue_imports(program);
        let mut reports = Vec::new();
        for &symbol in &named {
            for reference in semantic.symbol_references(symbol) {
                let node = reference.node_id();
                if reference.is_read()
                    && !matches!(
                        semantic.nodes().parent_kind(node),
                        AstKind::ExportSpecifier(_) | AstKind::TSTypeQuery(_)
                    )
                {
                    let span = semantic.nodes().kind(node).span();
                    check_use(semantic.nodes(), node, span, &mut reports);
                }
            }
        }
        for node in semantic.nodes().iter() {
            let AstKind::StaticMemberExpression(member) = node.kind() else {
                continue;
            };
            let next_tick = match member.property.name.as_str() {
                "$nextTick" => matches!(
                    member.object.without_parentheses(),
                    Expression::ThisExpression(_)
                ),
                "nextTick" => match member.object.without_parentheses() {
                    Expression::Identifier(object) => {
                        object.name == "Vue"
                            || resolves_to(&semantic, object.reference_id.get(), &namespaces)
                    }
                    _ => false,
                },
                _ => false,
            };
            if next_tick {
                check_use(
                    semantic.nodes(),
                    node.id(),
                    member.property.span,
                    &mut reports,
                );
            }
        }
        reports.sort_by_key(|(span, _)| span.start);
        for (span, message) in reports {
            let start = offset as u32 + span.start;
            let end = offset as u32 + span.end;
            result.add_diagnostic(
                LintDiagnostic::warn(META.name, message, start, end)
                    .with_label("invalid nextTick usage", start, end)
                    .with_help(HELP),
            );
        }
    }
}

fn vue_imports(program: &Program<'_>) -> (Vec<SymbolId>, Vec<SymbolId>) {
    let mut named = Vec::new();
    let mut namespaces = Vec::new();
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        if import.import_kind.is_type() || import.source.value.as_str() != "vue" {
            continue;
        }
        for specifier in import.specifiers.iter().flatten() {
            match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(specifier)
                    if !specifier.import_kind.is_type()
                        && specifier.imported.name() == "nextTick" =>
                {
                    named.extend(specifier.local.symbol_id.get());
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                    namespaces.extend(specifier.local.symbol_id.get());
                }
                _ => {}
            }
        }
    }
    (named, namespaces)
}

fn resolves_to(
    semantic: &Semantic<'_>,
    reference: Option<oxc_syntax::reference::ReferenceId>,
    symbols: &[SymbolId],
) -> bool {
    reference
        .and_then(|reference| semantic.scoping().get_reference(reference).symbol_id())
        .is_some_and(|symbol| symbols.contains(&symbol))
}

/// The nearest ancestor that is not a parenthesized expression; ESTree, which
/// the reference rule walks, has no parentheses.
fn parent(nodes: &AstNodes<'_>, node: NodeId) -> NodeId {
    let mut parent = nodes.parent_id(node);
    while matches!(nodes.kind(parent), AstKind::ParenthesizedExpression(_)) {
        parent = nodes.parent_id(parent);
    }
    parent
}

/// Check one `nextTick` reference: `node` is the identifier or member
/// expression, `span` is where the report goes.
fn check_use(
    nodes: &AstNodes<'_>,
    node: NodeId,
    span: Span,
    reports: &mut Vec<(Span, &'static str)>,
) {
    let callee = nodes.kind(node).span();
    let mut parent_id = parent(nodes, node);
    if matches!(nodes.kind(parent_id), AstKind::ConditionalExpression(_)) {
        parent_id = parent(nodes, parent_id);
    }
    let call = match nodes.kind(parent_id) {
        AstKind::CallExpression(call) if call.callee.without_parentheses().span() == callee => call,
        AstKind::CallExpression(_)
        | AstKind::VariableDeclarator(_)
        | AstKind::ObjectProperty(_)
        | AstKind::AssignmentExpression(_) => return,
        _ => {
            reports.push((span, NOT_CALLED));
            return;
        }
    };
    let message = match call.arguments.len() {
        0 if !promise_consumed(nodes, parent_id) => NOT_AWAITED,
        0 => return,
        1 if promise_consumed(nodes, parent_id) => AWAITED_CALLBACK,
        1 => return,
        _ => TOO_MANY_ARGUMENTS,
    };
    reports.push((span, message));
}

/// `isAwaitedPromise` of eslint-plugin-vue: whether the call's Promise is waited
/// on or handed to code that can wait on it.
fn promise_consumed(nodes: &AstNodes<'_>, call: NodeId) -> bool {
    let parent_id = parent(nodes, call);
    match nodes.kind(parent_id) {
        AstKind::AwaitExpression(_)
        | AstKind::ReturnStatement(_)
        | AstKind::VariableDeclarator(_)
        | AstKind::AssignmentExpression(_) => true,
        // An expression-bodied arrow returns its single expression statement.
        AstKind::ExpressionStatement(_) => {
            let body = nodes.parent_id(parent_id);
            matches!(nodes.kind(body), AstKind::FunctionBody(_))
                && matches!(nodes.parent_kind(body), AstKind::ArrowFunctionExpression(arrow) if arrow.expression)
        }
        AstKind::StaticMemberExpression(member) => member.property.name == "then",
        AstKind::ArrayExpression(_) => matches!(
            nodes.kind(parent(nodes, parent_id)),
            AstKind::CallExpression(call)
                if matches!(call.callee.without_parentheses(), Expression::StaticMemberExpression(callee)
                    if matches!(&callee.object, Expression::Identifier(object) if object.name == "Promise"))
        ),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::ValidNextTick;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(ValidNextTick));
        linter
    }

    #[test]
    fn test_valid_awaited() {
        let result = create_linter().lint(
            "import { nextTick } from 'vue'\nasync function f() { await nextTick() }",
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_then_chained() {
        let result = create_linter().lint(
            "import { nextTick } from 'vue'\nnextTick().then(() => focusInput())",
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_callback_argument() {
        let result = create_linter().lint(
            "import { nextTick } from 'vue'\nnextTick(() => focusInput())",
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_instance_callback() {
        let result = create_linter().lint("this.$nextTick(() => focusInput())", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_assigned_result() {
        // Assigning the promise is a use; this is a declaration, not a bare
        // expression statement.
        let result =
            create_linter().lint("import { nextTick } from 'vue'\nconst p = nextTick()", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_returned_result() {
        let result = create_linter().lint(
            "import { nextTick } from 'vue'\nfunction f() { return nextTick() }",
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_arrow_expression_return() {
        let result = create_linter().lint(
            "import { nextTick as vueNextTick } from 'vue';\nexport const useTick = () => { const nextTick = () => vueNextTick(); return { nextTick }; };",
            0,
        );
        assert_eq!(result.warning_count, 0, "{:#?}", result.diagnostics);
    }

    #[test]
    fn test_valid_parenthesized_arrow_expression_return() {
        let result = create_linter().lint(
            "import { nextTick } from 'vue'; const tick = () => (nextTick());",
            0,
        );
        assert_eq!(result.warning_count, 0, "{:#?}", result.diagnostics);
    }

    #[test]
    fn test_invalid_arrow_block_bare_call() {
        let result = create_linter().lint(
            "import { nextTick } from 'vue'; const tick = () => { nextTick(); };",
            0,
        );
        assert_eq!(result.warning_count, 1, "{:#?}", result.diagnostics);
    }

    #[test]
    fn test_invalid_arrow_expression_void_call() {
        let result = create_linter().lint(
            "import { nextTick } from 'vue'; const tick = () => void nextTick();",
            0,
        );
        assert_eq!(result.warning_count, 1, "{:#?}", result.diagnostics);
    }

    #[test]
    fn test_nested_arrow_restores_statement_context() {
        let result = create_linter().lint(
            "import { nextTick } from 'vue'; const tick = () => { const inner = () => nextTick(); nextTick(); return inner; };",
            0,
        );
        assert_eq!(result.warning_count, 1, "{:#?}", result.diagnostics);
    }

    #[test]
    fn test_invalid_bare_call() {
        let result = create_linter().lint("import { nextTick } from 'vue'\nnextTick()", 0);
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_bare_instance_call() {
        let result = create_linter().lint("this.$nextTick()", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_unrelated_next_tick_receiver_is_not_flagged() {
        let result = create_linter().lint("vm.$nextTick()", 0);
        assert_eq!(result.warning_count, 0, "{:#?}", result.diagnostics);
    }

    #[test]
    fn test_invalid_aliased_bare_call() {
        let result = create_linter().lint("import { nextTick as tick } from 'vue'\ntick()", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_namespace_call_before_import() {
        let result = create_linter().lint("Vue.nextTick(); import * as Vue from 'vue';", 0);
        assert_eq!(result.warning_count, 1, "{:#?}", result.diagnostics);
    }

    #[test]
    fn test_invalid_void_bare_call() {
        let result = create_linter().lint("import { nextTick } from 'vue'\nvoid nextTick()", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_unrelated_identifier_not_flagged() {
        // `nextTick` not imported from vue and not `$nextTick`: a local function
        // named `tick` is unrelated.
        let result = create_linter().lint("tick()", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_same_named_non_vue_import_not_flagged() {
        let result = create_linter().lint("import { nextTick } from './scheduler'; nextTick();", 0);
        assert_eq!(result.warning_count, 0, "{:#?}", result.diagnostics);
    }

    fn messages(source: &str) -> Vec<(u32, std::string::String)> {
        create_linter()
            .lint(source, 0)
            .diagnostics
            .into_iter()
            .map(|diagnostic| (diagnostic.start, diagnostic.message.to_string()))
            .collect()
    }

    #[test]
    fn test_reports_eslint_plugin_vue_shapes() {
        let source = "import { nextTick } from \"vue\";\nasync function f(done) {\n  nextTick().catch(done);\n  nextTick().finally(done);\n  nextTick(done, done);\n  await nextTick(done);\n  nextTick(done).then(done);\n  const reference = nextTick;\n  await Promise.all([nextTick, reference]);\n}";
        let found: Vec<_> = messages(source)
            .into_iter()
            .map(|(_, message)| message)
            .collect();
        assert_eq!(
            found,
            [
                super::NOT_AWAITED,
                super::NOT_AWAITED,
                super::TOO_MANY_ARGUMENTS,
                super::AWAITED_CALLBACK,
                super::AWAITED_CALLBACK,
                super::NOT_CALLED,
            ]
        );
    }

    #[test]
    fn test_promise_consumers_and_value_passing_are_valid() {
        let source = "import { nextTick } from \"vue\";\nasync function f(done) {\n  const p = nextTick();\n  done(nextTick);\n  done({ nextTick });\n  await Promise.all([nextTick(), p]);\n  nextTick().then(done).catch(done);\n  const pick = done ? nextTick : done;\n  return [p, pick];\n}";
        assert!(messages(source).is_empty(), "{:?}", messages(source));
    }

    #[test]
    fn test_shadowed_parameter_is_not_next_tick() {
        let source =
            "import { nextTick } from \"vue\";\nfunction f(nextTick) { nextTick(); [nextTick]; }";
        assert!(messages(source).is_empty(), "{:?}", messages(source));
    }

    #[test]
    fn test_reports_at_the_next_tick_identifier() {
        let source = "import { nextTick } from \"vue\";\nasync function f() { await nextTick(() => {}); this.$nextTick(); }";
        let starts: Vec<_> = messages(source)
            .into_iter()
            .map(|(start, _)| start)
            .collect();
        let first = source.find("nextTick(() =>").map(|index| index as u32);
        let second = source.find("$nextTick").map(|index| index as u32);
        assert_eq!(
            starts,
            [first, second].into_iter().flatten().collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_offset_applied() {
        let source = "this.$nextTick()";
        let result = create_linter().lint(source, 50);
        assert_eq!(result.warning_count, 1);
        assert_eq!(result.diagnostics[0].start, 55);
    }
}
