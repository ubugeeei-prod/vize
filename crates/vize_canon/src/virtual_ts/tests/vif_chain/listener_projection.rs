//! Listener relations are declared once per component and instantiated by key.
use super::{Analyzer, AnalyzerOptions, generate_virtual_ts};

#[test]
fn listeners_share_the_component_property_projection() {
    let script = "import Child from './Child.vue'\nconst select = (value: number) => value\nconst close = () => undefined\n";
    let template = r#"<Child @select="select" @close="close" />"#;
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, template);
    assert!(errors.is_empty(), "{errors:?}");
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let output = generate_virtual_ts(&analyzer.finish(), Some(script), Some(&root), 0);
    assert_eq!(
        output.code.matches("_listener_args<__P, __K extends string | number | symbol, __Raw extends string | number | symbol = never>").count(),
        1,
        "the component's listener argument relation must be declared once: {}",
        output.code,
    );
    assert_eq!(
        output
            .code
            .matches("[Extract<NonNullable<__H>, (...args: any) => any>] extends [never]")
            .count(),
        1,
        "per-event aliases must instantiate the shared relation: {}",
        output.code,
    );
}

#[test]
fn hoisted_listeners_use_the_program_argument_relation() {
    use crate::virtual_ts::{
        VirtualTsGenerationOptions, VirtualTsOptions, generate_virtual_ts_with_offsets_and_checks,
    };
    let script = "import Child from './Child.vue'\nconst value = 1\nconst select = (value: number) => value\nconst close = () => undefined\n";
    let template = r#"<Child :value="value" @select="select" @close="close" />"#;
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, template);
    assert!(errors.is_empty(), "{errors:?}");
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let output = generate_virtual_ts_with_offsets_and_checks(
        &analyzer.finish(),
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
        VirtualTsGenerationOptions {
            hoist_shared_preamble: true,
            ..Default::default()
        },
    );
    assert_eq!(
        output.code.matches("_listener_args<").count(),
        0,
        "{}",
        output.code
    );
    assert_eq!(
        output.code.matches("_listener_missing<").count(),
        0,
        "{}",
        output.code
    );
    assert_eq!(
        output
            .code
            .matches("import(\"vize:virtual-ts\").$ListenerArgs<")
            .count(),
        4,
        "{}",
        output.code
    );
    for (prefix, generic) in [
        ("type Extract<T, U> = never;\n", None),
        ("type NonNullable<T> = never;\n", None),
        ("import type { Extract } from './utility';\n", None),
        ("import Extract = External.Utility;\n", None),
        ("export import Extract = External.Utility;\n", None),
        ("namespace Utility { export type Item = number; }\n", None),
        ("", Some("Extract")),
        ("", Some("NonNullable")),
    ] {
        let script = vize_carton::cstr!("{prefix}{script}");
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(&script);
        analyzer.analyze_template(&root);
        let generic = generic.map(crate::virtual_ts::generic_parameters::GenericParameters::parse);
        let fallback = generate_virtual_ts_with_offsets_and_checks(
            &analyzer.finish(),
            Some(&script),
            Some(&root),
            0,
            0,
            &VirtualTsOptions::default(),
            VirtualTsGenerationOptions {
                hoist_shared_preamble: true,
                generic_parameters: generic.as_ref(),
                ..Default::default()
            },
        );
        assert_eq!(
            fallback
                .code
                .matches("import(\"vize:virtual-ts\").$ListenerArgs<")
                .count(),
            0,
            "{}",
            fallback.code,
        );
        assert_eq!(
            fallback.code.matches("_listener_args<__P, __K extends string | number | symbol, __Raw extends string | number | symbol = never>").count(),
            1,
            "{}",
            fallback.code,
        );
    }
}
