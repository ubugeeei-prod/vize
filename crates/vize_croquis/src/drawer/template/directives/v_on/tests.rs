use crate::{Analyzer, AnalyzerOptions, ScopeKind, TemplateExpressionKind};
use vize_carton::{Allocator, cstr};

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
