//! Caller-built shared roots retain their full authored-module origins.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "test packets use ordinary owned strings and JSON fixtures"
)]

mod backend_origin_oracle;
use backend_origin_oracle::{check_map, rejects_mutations};
use serde_json::json;
use vize_atelier_core::{lane::transform_with_source_text, options::TransformOptions};
use vize_atelier_jsx::{JsxLang, lower_source};
use vize_atelier_ssr::{
    SsrCompilerExperimentalOptions, SsrCompilerOptions, compile_l2_to_ssr,
    compile_l2_to_ssr_with_experimental_options,
};
use vize_atelier_vapor::{
    VaporGenerateExperimentalOptions, VaporGenerateOptions, generate_vapor_from_transformed_root,
    generate_vapor_with_options, transform_to_ir,
};
use vize_l0::{Allocator, Span};

#[test]
fn caller_built_jsx_roots_keep_real_vapor_and_ssr_origins() {
    let mut packets = Vec::new();
    for (prefix, ending) in [
        ("// before\n", "\n"),
        ("// 日本語 😀 'quoted'\r\n", "\r\n"),
        ("// edit adds an earlier line\n// 日本語 😀\n", "\n"),
    ] {
        let source = format!(
            "{prefix}const label: string = 'text';{ending}export const App = () => <p>{{label}}</p>;{ending}"
        );
        let allocator = Allocator::new();
        let mut lowered = lower_source(&allocator, allocator.as_oxc(), &source, JsxLang::Tsx);
        assert!(lowered.diagnostics.is_empty());
        assert_eq!(lowered.roots.len(), 1);
        let root = lowered.roots.pop().unwrap();
        let analysis = allocator.alloc_owned(lowered.analysis);
        let mut transformed = root.root;
        transform_with_source_text(
            &allocator,
            &mut transformed,
            TransformOptions {
                prefix_identifiers: false,
                vapor: true,
                ..Default::default()
            },
            Some(analysis),
            &source,
        );
        let legacy_ir = transform_to_ir(&allocator, &transformed, &source);
        let legacy = generate_vapor_with_options(
            &legacy_ir,
            None,
            VaporGenerateOptions { jsx_closure: true },
        );
        for mapped in [false, true] {
            let (output, errors, warnings) = generate_vapor_from_transformed_root(
                &allocator,
                &transformed,
                &source,
                None,
                None,
                VaporGenerateOptions { jsx_closure: true },
                VaporGenerateExperimentalOptions {
                    source_map: mapped,
                    source_map_filename: Some("Original 日本語.tsx"),
                    ..Default::default()
                },
            );
            assert!(errors.is_empty(), "{errors:?}");
            assert!(warnings.is_empty(), "{warnings:?}");
            assert_eq!(output.code, legacy.code);
            assert_eq!(output.templates, legacy.templates);
            assert_eq!(output.map.is_some(), mapped);
            if let Some(raw) = &output.map {
                packets.push(json!({ "backend": "vapor", "source": source, "output": check_map(&output.code, &source, raw, "vapor").unwrap() }));
            }
        }

        let l2 = root.l2.unwrap();
        let options = SsrCompilerOptions::default();
        let legacy = compile_l2_to_ssr(&allocator, &source, &l2.root, &options).unwrap();
        for invalid in [Span::new(0, 0), Span::new(0, source.len() as u32 + 1)] {
            assert!(
                compile_l2_to_ssr_with_experimental_options(
                    &allocator,
                    &source,
                    &l2.root,
                    invalid,
                    &options,
                    &SsrCompilerExperimentalOptions {
                        source_map: true,
                        ..Default::default()
                    },
                )
                .is_none()
            );
        }
        for mapped in [false, true] {
            let output = compile_l2_to_ssr_with_experimental_options(
                &allocator,
                &source,
                &l2.root,
                transformed.loc.span,
                &options,
                &SsrCompilerExperimentalOptions {
                    source_map: mapped,
                    source_map_filename: Some("Original 日本語.tsx".into()),
                    ..Default::default()
                },
            )
            .expect("the real L2 plan must own this JSX root");
            assert_eq!(output.code, legacy.code);
            assert_eq!(output.preamble, legacy.preamble);
            assert_eq!(output.map.is_some(), mapped);
            if let Some(raw) = &output.map {
                packets.push(json!({ "backend": "ssr", "source": source, "output": check_map(&output.code, &source, raw, "ssr").unwrap(), "preamble": output.preamble, "negatives": rejects_mutations(&output.code, &source, raw) }));
            }
        }
    }
    assert_eq!(packets.len(), 6);
    if let Some(path) = std::env::var_os("VIZE_JSX_BACKEND_ORIGIN_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&packets).unwrap()).unwrap();
    }
}
