//! The Options API public-instance form for unknown template names (#3888):
//! opted in only for a Vue 3 plain default export, and never for names the
//! script itself declares.

use crate::virtual_ts::generate_virtual_ts_with_offsets_options_api;

#[test]
fn test_options_api_template_bindings_use_default_instance_type() {
    let script = r#"export default {
    props: {
        initial: Number,
    },
    data() {
        return { count: 0 }
    },
    computed: {
        doubled() {
            return this.count * 2
        },
    },
    methods: {
        bump() {
            return this.count + 1
        },
    },
}
"#;
    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, "<div>{{ count }}</div>");
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full())
        .with_options_api();
    analyzer.analyze_script_plain(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts_with_offsets_options_api(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &Default::default(),
    );

    assert!(
        output.code.contains("type __VizeOptionsInstance<T>"),
        "expected Options API instance helper:\n{}",
        output.code
    );
    assert!(
        output
            .code
            .contains("const count: __VizeOptionsBinding<typeof __default__, \"count\">"),
        "expected data binding to reference the default component instance:\n{}",
        output.code
    );
    assert!(
        !output.code.contains("const count: any = undefined as any;"),
        "template data binding must not be emitted as a fixed any:\n{}",
        output.code
    );
}

/// A `namespace` the plain script declares stays resolvable from template scope
/// (the generated module keeps the declaration in a scope that encloses the
/// template closure), so it is not an unknown template name: a public-instance
/// property access for it would invent a `TS2339` vue-tsc never reports. A name
/// the script does not declare still resolves on the instance.
#[test]
fn test_options_api_script_declared_namespace_is_not_checked_on_the_instance() {
    let script = r#"namespace Bare {
  export const label = 'bare'
}

export default {
    name: 'Namespaces',
}
"#;
    let allocator = vize_carton::Bump::new();
    let (root, _) =
        vize_armature::parse(&allocator, "<div>{{ Bare.label }} {{ missingThing }}</div>");
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full())
        .with_options_api();
    analyzer.analyze_script_plain(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts_with_offsets_options_api(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &Default::default(),
    );

    assert!(
        !output.code.contains("__vize_template_instance.Bare"),
        "a script-declared namespace must not be checked against the public instance:\n{}",
        output.code
    );
    assert!(
        output
            .code
            .contains("__vize_template_instance.missingThing"),
        "a name the script never declares still resolves on the public instance:\n{}",
        output.code
    );
}
