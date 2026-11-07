//! v-on directive handling.
//!
//! Processes `@event="handler"` bindings including:
//! - Inline arrow/function callback scope creation
//! - Implicit `$event` parameter detection
//! - Simple handler reference tracking

use crate::drawer::Drawer;
use crate::drawer::helpers::{
    EventHandlerExpression, classify_event_handler_with_identifier, extract_identifiers_oxc,
};
use crate::scope::{EventHandlerScopeData, ScopeId};
use vize_carton::{CompactString, profile};
use vize_relief::ExpressionNode;

impl Drawer {
    /// Handle v-on directive.
    pub(in crate::drawer) fn handle_v_on_directive(
        &mut self,
        dir: &vize_relief::DirectiveNode<'_>,
        scope_vars: &mut Vec<CompactString>,
        target_component: Option<CompactString>,
    ) {
        if let Some(ref exp) = dir.exp {
            let compound_content;
            let content = match exp {
                ExpressionNode::Simple(s) => s.content,
                ExpressionNode::Compound(c) => {
                    compound_content = CompactString::new(c.loc.span.slice(&self.template_source));
                    compound_content.as_str()
                }
            };

            let Some(event_arg) = dir.arg.as_ref() else {
                if self.options.collect_template_expressions {
                    let loc = exp.loc();
                    let scope_id = self.croquis.scopes.current_id();
                    self.croquis
                        .template_expressions
                        .push(crate::croquis::TemplateExpression {
                            content: CompactString::new(content),
                            kind: crate::croquis::TemplateExpressionKind::VOn,
                            start: loc.span.start,
                            end: loc.span.end,
                            scope_id,
                            vif_guard: self.current_vif_guard(),
                        });
                }

                if self.checks_binding_reads() {
                    profile!(
                        "croquis.template.v_on.refs",
                        self.check_expression_refs(exp, scope_vars)
                    );
                }

                return;
            };

            // Every named listener runs in a handler scope. Whether its body
            // contains parentheses, semicolons or nested callbacks cannot change
            // that ownership; statement bodies must never become `void (body)`.
            let (shape, handler_binding) = if self
                .template_semantic_reuse
                .as_ref()
                .is_some_and(|reuse| reuse.is_inline_handler(exp))
            {
                (EventHandlerExpression::Inline, None)
            } else {
                profile!(
                    "croquis.template.event.classify",
                    classify_event_handler_with_identifier(content)
                )
            };
            let handler_vif_guard = handler_binding.as_ref().and_then(|name| {
                self.handler_vif_guard(self.croquis.scopes.current_id(), name.as_str())
            });
            let expression_vif_guard = if handler_binding.is_some() {
                handler_vif_guard
            } else {
                self.current_vif_guard()
            };
            let has_implicit_event = matches!(shape, EventHandlerExpression::Inline);
            let params = match shape {
                EventHandlerExpression::Callback { params, .. } => params,
                _ => Default::default(),
            };
            let (event_name, event_name_is_dynamic) = match event_arg {
                ExpressionNode::Simple(argument) => {
                    (CompactString::new(argument.content), !argument.is_static)
                }
                ExpressionNode::Compound(argument) => (
                    CompactString::new(argument.loc.span.slice(&self.template_source)),
                    true,
                ),
            };
            self.croquis.scopes.enter_event_handler_scope(
                EventHandlerScopeData {
                    event_name,
                    event_name_is_dynamic,
                    has_implicit_event,
                    param_names: params.into_iter().collect(),
                    handler_expression: Some(CompactString::new(content)),
                    target_component,
                },
                dir.loc.span.start,
                dir.loc.span.end,
            );
            if self.options.collect_template_expressions {
                let location = exp.loc();
                self.croquis
                    .template_expressions
                    .push(crate::croquis::TemplateExpression {
                        content: CompactString::new(content),
                        kind: crate::croquis::TemplateExpressionKind::VOn,
                        start: location.span.start,
                        end: location.span.end,
                        scope_id: self.croquis.scopes.current_id(),
                        vif_guard: expression_vif_guard,
                    });
            }
            let previous_count = scope_vars.len();
            scope_vars.extend(
                self.croquis
                    .scopes
                    .current_scope()
                    .bindings()
                    .map(|(name, _)| CompactString::new(name)),
            );
            if self.checks_binding_reads() || self.options.analyze_template_scopes {
                profile!(
                    "croquis.template.v_on.refs",
                    self.check_expression_refs(exp, scope_vars)
                );
            }
            scope_vars.truncate(previous_count);
            self.croquis.scopes.exit_scope();
        }
    }

    fn handler_vif_guard(&self, listener_scope: ScopeId, name: &str) -> Option<CompactString> {
        let listener_owner = self.croquis.scopes.binding_scope_from(listener_scope, name);
        let mut stable_reference = false;
        let mut replay = Vec::new();

        for (guard, guard_scope) in self.vif_guard_stack.iter().zip(&self.vif_guard_scope_stack) {
            let references_binding = extract_identifiers_oxc(guard.as_str())
                .iter()
                .any(|identifier| identifier == name);
            if references_binding
                && self.croquis.scopes.binding_scope_from(*guard_scope, name) != listener_owner
            {
                continue;
            }
            stable_reference |= references_binding;
            replay.push(guard.as_str());
        }

        stable_reference.then(|| CompactString::new(replay.join(" && ")))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Analyzer, AnalyzerOptions, ScopeKind, TemplateExpressionKind};
    use vize_carton::{Allocator, cstr};

    #[test]
    fn undefined_reporting_keeps_whole_lexical_read_facts() {
        const TEMPLATE: &str =
            "<button @click=\"handler($event, missing, Math, JSON, $attrs)\"></button>";
        let mut facts = Vec::new();
        for detect_undefined in [true, false] {
            let allocator = Allocator::new();
            let (root, errors) = vize_armature::parse(&allocator, TEMPLATE);
            assert!(errors.is_empty());
            let mut analyzer =
                Analyzer::with_options(AnalyzerOptions::full()).with_unused_bindings();
            analyzer
                .draw_script_setup("const handler = () => 1; const Math = 1; const unused = 2;");
            analyzer.options.detect_undefined = detect_undefined;
            analyzer.analyze_template(&root);
            let mut summary = analyzer.finish();
            assert_eq!(
                summary
                    .unused_bindings
                    .iter()
                    .map(|name| name.as_str())
                    .collect::<Vec<_>>(),
                ["unused"]
            );
            let diagnostics = summary
                .undefined_refs
                .iter()
                .map(|reference| {
                    (
                        reference.name.as_str(),
                        reference.offset,
                        reference.context.as_str(),
                    )
                })
                .collect::<Vec<_>>();
            let expected = if detect_undefined {
                vec![("missing", 32, "template expression")]
            } else {
                vec![]
            };
            assert_eq!(diagnostics, expected);
            summary.undefined_refs.clear();
            facts.push(cstr!("{summary:?}"));
        }
        assert_eq!(facts[0], facts[1]);
    }

    #[test]
    fn implicit_event_reads_are_tracked_even_without_undefined_diagnostics() {
        for detect_undefined in [true, false] {
            for (body, used) in [
                ("run()", false),
                ("run('$event')", false),
                ("run($event)", true),
                ("run(() => $event.target)", true),
                (r"run(\u0024event)", true),
            ] {
                let allocator = Allocator::new();
                let template = cstr!("<button @click=\"{body}\"></button>");
                let (root, errors) = vize_armature::parse(&allocator, &template);
                assert!(errors.is_empty(), "{body}");
                let mut options = AnalyzerOptions::full();
                options.detect_undefined = detect_undefined;
                let mut analyzer = Analyzer::with_options(options);
                analyzer.analyze_template(&root);
                let summary = analyzer.finish();
                let scope = summary
                    .scopes
                    .iter()
                    .find(|scope| scope.kind == ScopeKind::EventHandler)
                    .unwrap();
                assert_eq!(
                    scope.get_binding("$event").unwrap().is_used(),
                    used,
                    "{body}, detect_undefined={detect_undefined}"
                );
            }
        }
    }

    #[test]
    fn every_named_listener_has_its_own_handler_scope() {
        for body in [
            "run()",
            "for (item of items) {}",
            "for (const it of items) { void it; }",
            "if (ready) run()",
            "while (ready) { run(); break; }",
            "if (ready) return; run()",
            "const callback = () => run(); callback()",
            "run('text; with => punctuation')",
            "function f<T>(x: T) { return x; } f(1)",
        ] {
            let allocator = Allocator::new();
            let template = cstr!("<button @click=\"{body}\"></button>");
            let (root, errors) = vize_armature::parse(&allocator, &template);
            assert!(errors.is_empty(), "{body}");
            let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
            analyzer.analyze_template(&root);
            let summary = analyzer.finish();
            let expression = summary
                .template_expressions
                .iter()
                .find(|expression| expression.kind == TemplateExpressionKind::VOn)
                .unwrap();
            let scope = summary.scopes.get_scope(expression.scope_id).unwrap();
            assert_eq!(scope.kind, ScopeKind::EventHandler, "{body}");
            assert!(scope.bindings().any(|(name, _)| name == "$event"), "{body}");
        }
    }

    #[test]
    fn references_and_callbacks_do_not_capture_an_implicit_event() {
        for body in [
            "handler",
            "$event",
            "$event.handler",
            "handlers?.click",
            "() => run($event)",
        ] {
            let allocator = Allocator::new();
            let template = cstr!("<button @click=\"{body}\"></button>");
            let (root, errors) = vize_armature::parse(&allocator, &template);
            assert!(errors.is_empty(), "{body}");
            let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
            analyzer.analyze_template(&root);
            let summary = analyzer.finish();
            let expression = summary
                .template_expressions
                .iter()
                .find(|expression| expression.kind == TemplateExpressionKind::VOn)
                .unwrap();
            let scope = summary.scopes.get_scope(expression.scope_id).unwrap();
            assert_eq!(scope.kind, ScopeKind::EventHandler, "{body}");
            assert!(
                !scope.bindings().any(|(name, _)| name == "$event"),
                "{body}"
            );
        }
    }

    #[test]
    fn listener_guards_retain_binding_identity_across_v_for_shadowing() {
        let script = "const handlers: (number | (() => void))[] = []; const handler = () => {}; const ready = true;";
        let cases = [
            (
                "<template v-if=\"typeof handler === 'number'\"><div></div></template><template v-else><Row v-for=\"handler in handlers\" @act=\"handler\" /></template>",
                false,
            ),
            (
                "<Row v-if=\"typeof handler === 'function'\" v-for=\"handler in handlers\" @act=\"handler\" />",
                false,
            ),
            (
                "<template v-for=\"handler in handlers\"><Row v-if=\"typeof handler === 'function'\" @act=\"handler\" /></template>",
                true,
            ),
            (
                "<div v-if=\"ready\"><template v-for=\"handler in handlers\"><Row v-if=\"typeof handler === 'function'\" @act=\"handler\" /></template></div>",
                true,
            ),
            (
                "<div v-if=\"handler !== null\"><template v-for=\"handler in handlers\"><Row v-if=\"typeof handler === 'function'\" @act=\"handler\" /></template></div>",
                true,
            ),
        ];

        for (template, expected) in cases {
            let allocator = Allocator::new();
            let (root, errors) = vize_armature::parse(&allocator, template);
            assert!(errors.is_empty(), "{template}: {errors:?}");
            let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
            analyzer.analyze_script_setup(script);
            analyzer.analyze_template(&root);
            let summary = analyzer.finish();
            let expression = summary
                .template_expressions
                .iter()
                .find(|expression| expression.kind == TemplateExpressionKind::VOn)
                .unwrap();
            assert_eq!(expression.vif_guard.is_some(), expected, "{template}");
            if template.contains("handler !== null") {
                let guard = expression.vif_guard.as_deref().unwrap();
                assert!(!guard.contains("handler !== null"), "{guard}");
                assert!(guard.contains("typeof handler === 'function'"), "{guard}");
            }
        }
    }
}
