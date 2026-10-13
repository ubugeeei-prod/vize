//! #6880: route metadata must not eagerly evaluate unrelated component imports.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use serde_json::json;
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode};
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode,
    TemplateCompileOptions, compile_sfc_for_adapter_with_experimental_options,
    compile_sfc_for_adapter_with_nuxt_page_meta, parse_sfc,
};

#[test]
fn whole_original_photo_metadata_has_no_unrelated_component_dependencies() {
    let source = include_str!(
        "../../../tests/_fixtures/differential/compiler/vuefes-page-meta-dependencies/Photo.vue.txt"
    );
    let mut observations = Vec::new();
    for shape in ["module", "inline", "ssr", "vapor", "vapor-ssr"] {
        for nuxt in [false, true] {
            for source_map in [false, true] {
                let parse = SfcParseOptions {
                    filename: "app/pages/photo/index.vue".into(),
                    ..Default::default()
                };
                let descriptor = parse_sfc(source, parse.clone()).expect("whole original SFC");
                let compiler = if nuxt {
                    compile_sfc_for_adapter_with_nuxt_page_meta
                } else {
                    compile_sfc_for_adapter_with_experimental_options
                };
                let result = compiler(
                    &descriptor,
                    SfcCompileOptions {
                        parse,
                        script: ScriptCompileOptions {
                            id: Some("app/pages/photo/index.vue".into()),
                            ..Default::default()
                        },
                        template: TemplateCompileOptions {
                            ssr: matches!(shape, "ssr" | "vapor-ssr"),
                            is_prod: true,
                            ..Default::default()
                        },
                        vapor: shape.starts_with("vapor"),
                        ..Default::default()
                    },
                    TemplateSyntaxMode::Standard,
                    Default::default(),
                    CodegenOptions {
                        source_map,
                        ..Default::default()
                    },
                    if matches!(shape, "inline" | "vapor") {
                        SfcScriptOutputMode::InlineTemplate
                    } else {
                        SfcScriptOutputMode::SeparateTemplate
                    },
                    Default::default(),
                )
                .expect("whole original compiler output");
                observations.push(json!({
                    "shape": shape, "nuxt": nuxt, "sourceMap": source_map,
                    "compileResult": result,
                }));
            }
        }
    }
    println!(
        "WHOLE_PHOTO_COMPILER={}",
        json!({"source": source, "observations": observations})
    );
    for observation in &observations {
        if observation["nuxt"] != true {
            continue;
        }
        let artifacts = observation["compileResult"]["macroArtifacts"]
            .as_array()
            .expect("all actual macro artifacts");
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0]["kind"], "nuxt.definePageMeta");
        assert_eq!(
            artifacts[0]["moduleCode"],
            "const __nuxt_page_meta = { prerender: true }\nexport default __nuxt_page_meta\n",
            "literal metadata has no imported dependencies in the pinned Nuxt oracle"
        );
    }
    let before: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/compiler/vuefes-page-meta-dependencies/before.compiler.json"
    ))
    .expect("complete original-source compiler packet, captured before this repair");
    assert_eq!(before["source"], source);
    let mut expected = before["observations"].clone();
    for observation in expected.as_array_mut().expect("all original observations") {
        if observation["nuxt"] == true {
            observation["compileResult"]["macroArtifacts"][0]["moduleCode"] = json!(
                "const __nuxt_page_meta = { prerender: true }\nexport default __nuxt_page_meta\n"
            );
        }
    }
    // #8636 fixes nested Vapor components inside Photo's actual <div> root.
    // Preserve the captured before packet and permit only its two root flags.
    for (index, nuxt, source_map) in [
        (12, false, false),
        (13, false, true),
        (14, true, false),
        (15, true, true),
    ] {
        let observation = &mut expected[index];
        assert_eq!(observation["shape"], "vapor");
        assert_eq!(observation["nuxt"], nuxt);
        assert_eq!(observation["sourceMap"], source_map);
        let mut code = observation["compileResult"]["code"]
            .as_str()
            .expect("complete captured Vapor code")
            .to_owned();
        for (old, new) in [
            (
                "      return n3;\n    } }, true);",
                "      return n3;\n    } }, false);",
            ),
            (
                "    return [n5, n6];\n  } }, true);",
                "    return [n5, n6];\n  } }, false);",
            ),
        ] {
            assert_eq!(
                code.matches(old).count(),
                1,
                "one original nested root flag"
            );
            code = code.replacen(old, new, 1);
        }
        observation["compileResult"]["code"] = json!(code);
    }
    assert_eq!(
        json!(observations),
        expected,
        "whole code, CSS, maps, errors, warnings, bindings and provenance stay exact outside the metadata repair and two nested Vapor root flags"
    );
}

#[expect(
    clippy::expect_used,
    reason = "whole-SFC differential test prerequisite"
)]
fn compile_metadata(source: &str) -> vize_atelier_sfc::SfcCompileResult {
    let parse = SfcParseOptions {
        filename: "Metadata.vue".into(),
        ..Default::default()
    };
    let descriptor = parse_sfc(source, parse.clone()).expect("whole dependency SFC");
    compile_sfc_for_adapter_with_nuxt_page_meta(
        &descriptor,
        SfcCompileOptions {
            parse,
            ..Default::default()
        },
        TemplateSyntaxMode::Standard,
        Default::default(),
        CodegenOptions {
            source_map: true,
            ..Default::default()
        },
        SfcScriptOutputMode::SeparateTemplate,
        Default::default(),
    )
    .expect("whole dependency compiler output")
}

#[test]
fn needed_imports_local_declarations_and_scope_match_the_pinned_nuxt_oracle() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/compiler/vuefes-page-meta-dependencies/official.dependencies.json"
    ))
    .expect("complete live Nuxt 4.0.3 observations, including whole compiled official scripts");
    let cases: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/compiler/vuefes-page-meta-dependencies/dependencies.json"
    ))
    .expect("whole dependency SFC corpus");
    let observations = oracle["observations"]
        .as_array()
        .expect("all actual oracles");
    assert_eq!(observations.len(), 8);
    let mut actual = Vec::new();
    for (fixture, reference) in cases
        .as_array()
        .expect("all cases")
        .iter()
        .zip(observations)
    {
        assert_eq!(reference["source"], fixture["source"]);
        let compiled = compile_metadata(fixture["source"].as_str().expect("whole SFC"));
        assert!(compiled.errors.is_empty(), "{:?}", compiled.errors);
        assert!(compiled.warnings.is_empty(), "{:?}", compiled.warnings);
        assert_eq!(compiled.macro_artifacts.len(), 1);
        let code = compiled.macro_artifacts[0]
            .module_code
            .as_deref()
            .expect("real artifact");
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, code, SourceType::mjs()).parse();
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let statements: Vec<_> = parsed
            .program
            .body
            .iter()
            .map(|statement| {
                let span = statement.span();
                code.get(span.start as usize..span.end as usize)
                    .expect("actual parser statement span")
            })
            .collect();
        actual
            .push(json!({"fixture": fixture, "compileResult": compiled, "statements": statements}));
        if fixture["oracleShadowingLimit"].is_string() {
            // The retained whole oracle explicitly records Nuxt's name-only
            // initializer walk. Resolved references preserve the same metadata
            // while omitting its proven unused imported parameter lookalike.
            assert_eq!(code, fixture["expectedNativeModuleCode"].as_str().unwrap());
        } else {
            let expected: Vec<_> = reference["statements"]
                .as_array()
                .unwrap()
                .iter()
                .map(|statement| statement["source"].as_str().unwrap())
                .collect();
            assert_eq!(statements, expected, "{}", fixture["name"]);
        }
    }
    println!(
        "METADATA_DEPENDENCY_COMPILER={}",
        json!({"oracle": oracle, "actual": actual})
    );
}

#[test]
fn mutual_function_dependencies_remain_finite_and_preserve_both_declarations() {
    let source = "<script setup>\nfunction first() { return second; }\nfunction second() { return first; }\ndefinePageMeta({ middleware: first });\n</script>\n<template><main /></template>";
    let compiled = compile_metadata(source);
    assert!(compiled.errors.is_empty());
    assert_eq!(compiled.macro_artifacts.len(), 1);
    assert_eq!(
        compiled.macro_artifacts[0].module_code.as_deref().unwrap(),
        "function first() { return second; }\nfunction second() { return first; }\nconst __nuxt_page_meta = { middleware: first }\nexport default __nuxt_page_meta\n"
    );
}
