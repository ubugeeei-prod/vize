use super::GenericParameters;

#[test]
fn parser_owned_boundaries_preserve_full_variants_and_rows() {
    for (raw, names, defaults, aliases, fallback) in [
        (
            "const T extends 'a,b'",
            "T",
            "const T extends 'a,b' = any",
            "T extends 'a,b' = any",
            "'a,b'",
        ),
        (
            "T extends `a,${string}`, U = 'x,y'",
            "T, U",
            "T extends `a,${string}` = any, U = 'x,y'",
            "T extends `a,${string}` = any, U = 'x,y'",
            "`a,${string}`, unknown",
        ),
        (
            "const F extends (left: string, right: number) => [string, number], U",
            "F, U",
            "const F extends (left: string, right: number) => [string, number] = any, U = any",
            "F extends (left: string, right: number) => [string, number] = any, U = any",
            "(left: string, right: number) => [string, number], unknown",
        ),
        (
            "状態 extends Record<'a,b', [string, number]> = { 'a,b': [string, number] }, 値 = 状態",
            "状態, 値",
            "状態 extends Record<'a,b', [string, number]> = { 'a,b': [string, number] }, 値 = 状態",
            "状態 extends Record<'a,b', [string, number]> = { 'a,b': [string, number] }, 値 = 状態",
            "Record<'a,b', [string, number]>, unknown",
        ),
        (
            "T extends 'T', U extends T",
            "T, U",
            "T extends 'T' = any, U extends T = any",
            "T extends 'T' = any, U extends T = any",
            "'T', any",
        ),
        (
            "T, F extends <T>(value: T) => T",
            "T, F",
            "T = any, F extends <T>(value: T) => T = any",
            "T = any, F extends <T>(value: T) => T = any",
            "unknown, <T>(value: T) => T",
        ),
    ] {
        for newline in ["\n", "\r\n"] {
            let raw = raw.replace(", U", &vize_carton::cstr!(",{newline}U"));
            let plan = GenericParameters::parse(&raw);
            assert_eq!(plan.diagnostics, vec![], "{raw}");
            assert_eq!(plan.names, names, "{raw}");
            assert_eq!(plan.declaration, defaults, "{raw}");
            assert_eq!(plan.alias_declaration, aliases, "{raw}");
            assert_eq!(plan.fallback, fallback, "{raw}");
            for (rendered, rows) in plan.declaration_variants() {
                for (generated, original) in rows {
                    assert_eq!(
                        rendered.get(generated.clone()),
                        raw.get(original.clone()),
                        "{raw}: {rendered}"
                    );
                }
            }
        }
    }
}

#[test]
fn comments_and_authored_defaults_are_not_inferred_from_spelling() {
    let raw = "/* head , */ const /* modifier , */ T extends 'a=b,c' /* tail , */, // gap ,\n U = any // final ,";
    let plan = GenericParameters::parse(raw);
    assert_eq!(plan.diagnostics, vec![]);
    assert_eq!(plan.names, "T, U");
    assert_eq!(
        plan.declaration,
        "/* head , */ const /* modifier , */ T extends 'a=b,c' = any /* tail , */, // gap ,\n U = any // final ,\n"
    );
    assert_eq!(
        plan.alias_declaration,
        "/* head , */ /* modifier , */ T extends 'a=b,c' = any /* tail , */, // gap ,\n U = any // final ,\n"
    );
    assert_eq!(
        plan.alias_authored,
        "/* head , */ /* modifier , */ T extends 'a=b,c' /* tail , */, // gap ,\n U = any // final ,\n"
    );
    assert_eq!(plan.authored(), vize_carton::cstr!("{raw}\n"));
    assert_eq!(plan.fallback, "'a=b,c', unknown");
    assert!(plan.has_repeated_fallback_constraint());
    for (rendered, rows) in plan.declaration_variants() {
        for (generated, original) in rows {
            assert_eq!(rendered.get(generated.clone()), raw.get(original.clone()));
        }
    }
}

#[test]
fn malformed_parameters_keep_the_entire_authored_signature() {
    for raw in [
        "T extends",
        "T = 'a,b",
        "T extends (a: string, b: number => string",
        "T,, U",
        "const",
    ] {
        let plan = GenericParameters::parse(raw);
        assert!(!plan.diagnostics.is_empty(), "{raw}");
        assert_eq!(plan.raw(), raw);
        for (rendered, rows) in plan.declaration_variants() {
            assert_eq!(rendered, raw);
            assert_eq!(rows, &[(0..raw.len(), 0..raw.len())]);
        }
        for diagnostic in plan.diagnostics {
            assert!(diagnostic.start <= diagnostic.end);
            assert!(diagnostic.end as usize <= raw.len());
        }
    }
}

#[test]
fn a_valid_synthetic_program_cannot_escape_the_parameter_boundary() {
    for raw in [
        "T>() {} function other<U",
        "T>() {} const accepted = 1; function other<U",
    ] {
        let plan = GenericParameters::parse(raw);
        assert!(plan.escaped_wrapper);
        assert_eq!(
            plan.diagnostics,
            vec![crate::script_parse::ScriptParseDiagnostic {
                message: "Generic attribute must contain one complete type-parameter list".into(),
                start: 0,
                end: raw.len() as u32,
            }]
        );
        for (rendered, rows) in plan.declaration_variants() {
            assert_eq!(rendered, raw);
            assert_eq!(rows, &[(0..raw.len(), 0..raw.len())]);
        }
    }
}

#[test]
fn source_generation_prepares_one_plan_and_refuses_invalid_authored_syntax() {
    for newline in ["\n", "\r\n"] {
        let valid = "<!-- 日本 🧭 -->\n<script setup lang=\"ts\" generic=\"const T extends 'a,b'\">\nconst props = defineProps<{ value: T }>()\ndefineEmits<{ change: [value: T] }>()\ndefineSlots<{ default: (value: T) => unknown }>()\n</script>\n<template>{{ props.value }}</template>".replace('\n', newline);
        let options =
            crate::sfc_typecheck::SfcTypeCheckOptions::new("src/Comma.vue").with_virtual_ts();
        let before = super::preparation_count(false);
        let result = crate::sfc_typecheck::type_check_sfc(&valid, &options);
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        assert!(result.virtual_ts.is_some());
        assert_eq!(super::preparation_count(false) - before, 1);
        for invalid in [
            "T extends",
            "T>() {} function other<U",
            "const\nT",
            "const\r\nT",
        ] {
            let broken = valid.replace("const T extends 'a,b'", invalid);
            let before = super::preparation_count(false);
            let result = crate::sfc_typecheck::type_check_sfc(&broken, &options);
            assert!(!result.diagnostics.is_empty(), "{invalid}");
            assert!(result.virtual_ts.is_none(), "{invalid}");
            assert_eq!(super::preparation_count(false) - before, 1);
            let start = broken.find(invalid).unwrap() as u32;
            for diagnostic in &result.diagnostics {
                assert_eq!(diagnostic.code.as_deref(), Some("script-parse-error"));
                assert!(
                    diagnostic.start >= start && diagnostic.end <= start + invalid.len() as u32
                );
            }
        }
    }
}

#[cfg(feature = "native")]
#[test]
fn all_mapper_options_reuse_one_prepared_plan() {
    let source = "<script setup lang=\"ts\" generic=\"const T extends 'a,b'\">defineProps<{ value: T }>()</script><template>{{ value }}</template>";
    for bits in 0..16u8 {
        let options = crate::ContentMapperTransformOptions::default()
            .with_options_api(bits & 1 != 0)
            .with_preserve_unused_diagnostics(bits & 2 != 0)
            .with_check_javascript(bits & 4 != 0)
            .with_external_component_attrs_module(bits & 8 != 0);
        let before = super::preparation_count(false);
        let result = crate::generate_vue_content_mapper_transform_with_options(
            std::path::Path::new("src/Comma.vue"),
            source,
            options,
        )
        .unwrap();
        assert!(result.diagnostics.is_empty());
        assert!(!result.text.is_empty());
        assert_eq!(super::preparation_count(false) - before, 1);
    }
}
