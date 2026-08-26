use super::helpers::generate_template_context;
use super::{TemplateGlobal, VirtualTsOptions, generate_virtual_ts_with_offsets};
use vize_carton::config::VueVersion;
use vize_croquis::{Analyzer, AnalyzerOptions};

#[test]
fn test_strict_template_context_keeps_router_but_not_unknown_plugin_globals() {
    let ctx = generate_template_context(
        &VirtualTsOptions {
            strict_instance_globals: true,
            ..Default::default()
        },
        VueVersion::V3,
        false,
    );

    assert!(ctx.contains("type __VizeStrictPublicInstanceGlobals = {"));
    assert!(ctx.contains("string extends __Rest ? never"));
    // vue-router registers both `$route` and `$router`; no special case (#4963).
    assert!(!ctx.contains("\"$route\""));
    assert!(ctx.contains("unknown extends T ? any : T"));
    assert!(ctx.contains("type __VizeStrictTemplateContext = {"));
    assert!(ctx.contains("} & __VizeStrictPublicInstanceGlobals;"));
    assert!(!ctx.contains("& __Ctx"));
    assert!(!ctx.contains("$route: any;"));
    assert!(!ctx.contains("$router: any;"));
    assert!(!ctx.contains("$t:"));
}

#[test]
fn test_strict_template_context_does_not_inherit_public_instance_unknowns() {
    let template = r#"<div>{{ $t('account.follow') }}</div>"#;

    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts_with_offsets(
        &summary,
        None,
        Some(&root),
        0,
        0,
        &VirtualTsOptions {
            strict_instance_globals: true,
            ..Default::default()
        },
    );

    assert!(
        output
            .code
            .contains("const $t = __vize_strict_template_context.$t;"),
        "{}",
        output.code
    );
    assert!(
        !output.code.contains("__VizeStrictTemplateContext & __Ctx"),
        "{}",
        output.code
    );
}

#[test]
fn test_strict_template_context_does_not_recheck_v_for_scope_bindings() {
    let template = r#"<div v-for="(item, index) in items">{{ item.label }} {{ index }}</div>"#;
    let script = r#"const items = [{ label: "ready" }];"#;

    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts_with_offsets(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions {
            strict_instance_globals: true,
            ..Default::default()
        },
    );

    assert!(
        !output.code.contains("__vize_strict_template_context.item"),
        "{}",
        output.code
    );
    assert!(
        !output.code.contains("__vize_strict_template_context.index"),
        "{}",
        output.code
    );
}

#[test]
fn test_strict_template_context_does_not_recheck_template_props() {
    let template = r#"<section>{{ title }} {{ confirm }}</section>"#;
    let script = "defineProps<{ title: string; confirm?: string }>();\n";

    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts_with_offsets(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions {
            strict_instance_globals: true,
            ..Default::default()
        },
    );

    assert!(
        !output.code.contains("__vize_strict_template_context.title"),
        "{}",
        output.code
    );
    assert!(
        !output
            .code
            .contains("__vize_strict_template_context.confirm"),
        "{}",
        output.code
    );
}

#[test]
fn test_strict_template_unknown_refs_read_context_without_shadowing_auto_imports() {
    let template = r#"<div>{{ getPreferences() }} {{ getPreferences() }} {{ currentUser.name }} {{ $shout('ok') }}</div>"#;

    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts_with_offsets(
        &summary,
        None,
        Some(&root),
        0,
        0,
        &VirtualTsOptions {
            auto_import_bindings: vec!["currentUser".into()],
            strict_instance_globals: true,
            template_globals: vec![TemplateGlobal {
                name: "$shout".into(),
                type_annotation: "(value: string) => string".into(),
                default_value: "((value: string) => value) as any".into(),
            }],
            ..Default::default()
        },
    );

    assert!(
        output
            .code
            .contains("var getPreferences: any = undefined; void (getPreferences);"),
        "{}",
        output.code
    );
    assert_eq!(
        output
            .code
            .matches("__vize_strict_template_context.getPreferences")
            .count(),
        2,
        "{}",
        output.code
    );
    assert!(
        output
            .code
            .contains("var currentUser: __U<__R_currentUser> = undefined as any;"),
        "{}",
        output.code
    );
    assert!(
        !output
            .code
            .contains("__vize_strict_template_context.currentUser"),
        "{}",
        output.code
    );
    assert!(
        !output
            .code
            .contains("__vize_strict_template_context.$shout"),
        "{}",
        output.code
    );
}

#[test]
fn test_strict_template_expression_keeps_member_root_type() {
    let template = r#"<div>{{ route.missing }}</div>"#;
    let script = "const route = { query: { tab: 'home' } };\n";

    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts_with_offsets(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions {
            strict_instance_globals: true,
            ..Default::default()
        },
    );

    assert!(
        !output.code.contains("var route: any = undefined;"),
        "{}",
        output.code
    );
    assert!(
        output.code.contains("void (route.missing);"),
        "{}",
        output.code
    );
}

#[test]
fn test_strict_template_expression_reports_unknown_member_root() {
    let template = r#"<div>{{ plugin.translate }} {{ plugin["translate"] }} {{ plugin[key] }} {{ (plugin).translate }} {{ plugin /* comment */.translate }}</div>"#;

    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts_with_offsets(
        &summary,
        None,
        Some(&root),
        0,
        0,
        &VirtualTsOptions {
            strict_instance_globals: true,
            ..Default::default()
        },
    );

    assert!(
        output
            .code
            .contains("__vize_strict_template_context.plugin"),
        "{}",
        output.code
    );
    assert_eq!(
        output
            .code
            .matches("__vize_strict_template_context.plugin")
            .count(),
        5,
        "{}",
        output.code
    );
    assert!(
        !output.code.contains("var plugin: any = undefined;"),
        "{}",
        output.code
    );
    assert!(
        !output.code.contains("__vize_strict_template_context.key"),
        "{}",
        output.code
    );
}

#[test]
fn test_strict_template_expression_ignores_literals_and_builtins() {
    let template = r#"<div>{{ false }} {{ true }} {{ null }} {{ undefined }} {{ NaN }} {{ Infinity }} {{ Math.max(1, 2) }} {{ Date.now() }} {{ JSON.stringify({}) }}</div>"#;

    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts_with_offsets(
        &summary,
        None,
        Some(&root),
        0,
        0,
        &VirtualTsOptions {
            strict_instance_globals: true,
            ..Default::default()
        },
    );

    for name in [
        "false",
        "true",
        "null",
        "undefined",
        "NaN",
        "Infinity",
        "Math",
        "Date",
        "JSON",
    ] {
        assert!(
            !output
                .code
                .contains(&format!("var {name}: any = undefined;")),
            "{}",
            output.code
        );
        assert!(
            !output
                .code
                .contains(&format!("__vize_strict_template_context.{name}")),
            "{}",
            output.code
        );
    }
}
