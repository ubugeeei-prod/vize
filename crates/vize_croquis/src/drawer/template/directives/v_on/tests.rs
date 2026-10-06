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
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full()).with_unused_bindings();
        analyzer.draw_script_setup("const handler = () => 1; const Math = 1; const unused = 2;");
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
