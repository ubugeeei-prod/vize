//! v-on directive handling.
//!
//! Processes `@event="handler"` bindings including:
//! - Inline arrow/function callback scope creation
//! - Implicit `$event` parameter detection
//! - Simple handler reference tracking

use crate::drawer::Drawer;
use crate::drawer::helpers::{EventHandlerExpression, classify_event_handler};
use crate::scope::EventHandlerScopeData;
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
            let shape = if self
                .template_semantic_reuse
                .as_ref()
                .is_some_and(|reuse| reuse.is_inline_handler(exp))
            {
                EventHandlerExpression::Inline
            } else {
                profile!(
                    "croquis.template.event.classify",
                    classify_event_handler(content)
                )
            };
            let has_implicit_event = matches!(shape, EventHandlerExpression::Inline);
            let params = match shape {
                EventHandlerExpression::Callback { params, .. } => params,
                _ => Default::default(),
            };
            let event_name = match event_arg {
                ExpressionNode::Simple(argument) => CompactString::new(argument.content),
                ExpressionNode::Compound(argument) => {
                    CompactString::new(argument.loc.span.slice(&self.template_source))
                }
            };
            self.croquis.scopes.enter_event_handler_scope(
                EventHandlerScopeData {
                    event_name,
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
                        vif_guard: self.current_vif_guard(),
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
}
