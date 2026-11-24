//! What holds for every component: a requested map never changes the module
//! or its diagnostics, and later rewrites of the module keep the template
//! mappings.

use super::{
    BROKEN, CONSTRUCTS, IMPORT_ONLY, INLINE, OPTIONS_API, SEPARATE, SSR, Shape, TEMPLATE_ONLY,
    compile, resolve, template_segments, try_compile,
};

use crate::types::{SfcCompileResult, SfcError};

#[test]
fn a_standalone_module_keeps_the_template_mappings() {
    // `mode: "function"` rewrites the module line by line after assembly.
    let shape = Shape {
        standalone: true,
        ..INLINE
    };
    let result = compile(BROKEN, shape);
    assert_eq!(
        template_segments(&result, BROKEN),
        [
            r#"16:17 "Child, _hois" -> 8:3 "Child label=""#,
            r#"17:25 "p\", null, _t" -> 9:3 "p>{{ trip.ti""#,
            r#"17:52 "_unref(trip)" -> 9:8 "trip.title }""#,
            r#"17:65 "title), 1 /*" -> 9:13 "title }}</p>""#,
        ],
        "{}",
        result.code
    );
}

#[test]
fn stripping_typescript_at_the_boundary_keeps_template_mappings() {
    // The module keeps TypeScript (`$event: any`, `id: number`) until the
    // emitter boundary re-prints it as JavaScript and carries the map along.
    let typed = compile(
        CONSTRUCTS,
        Shape {
            typescript: true,
            ..INLINE
        },
    );
    let (code, _, map) = crate::module_shape::finalize_module_output_with_map(
        typed.code.clone(),
        false,
        typed.map.clone(),
    );
    assert_ne!(code, typed.code, "the fixture must need the strip");
    let stripped = SfcCompileResult { code, map, ..typed };
    assert_eq!(
        [
            resolve(&stripped, "title.value,"),
            resolve(&stripped, "items.value,"),
            resolve(&stripped, "row.name"),
        ],
        [Some((13, 19)), Some((17, 34)), Some((23, 38))]
    );
}

#[test]
fn a_requested_map_never_changes_the_module() {
    let javascript = CONSTRUCTS
        .replace(" lang=\"ts\"", "")
        .replace("id: number", "id");
    let sources = [
        BROKEN,
        IMPORT_ONLY,
        CONSTRUCTS,
        javascript.as_str(),
        OPTIONS_API,
        TEMPLATE_ONLY,
    ];
    let diagnostics = |result: &SfcCompileResult| {
        let render = |errors: &[SfcError]| -> Vec<String> {
            errors
                .iter()
                .map(|error| format!("{:?} {}", error.code, error.message))
                .collect()
        };
        (render(&result.errors), render(&result.warnings))
    };
    for source in sources {
        for (inline, ssr, typescript) in [
            (false, false, false),
            (false, false, true),
            (false, true, false),
            (false, true, true),
            (true, false, false),
            (true, false, true),
            (true, true, false),
            (true, true, true),
        ] {
            let shape = Shape {
                inline,
                ssr,
                typescript,
                standalone: false,
            };
            let plain = try_compile(source, shape, false).expect("plain compile");
            let mapped = try_compile(source, shape, true).expect("mapped compile");
            assert_eq!(plain.map, None, "{shape:?}\n{source}");
            assert_eq!(mapped.code, plain.code, "{shape:?}\n{source}");
            assert_eq!(
                diagnostics(&mapped),
                diagnostics(&plain),
                "{shape:?}\n{source}"
            );
            assert_ne!(
                template_segments(&mapped, source),
                [""; 0],
                "{shape:?}\n{source}"
            );
        }
    }
}

#[test]
fn a_template_error_is_located_the_same_with_or_without_a_map() {
    // `v-else` without a preceding `v-if` is a fatal template error.
    let source = "<script setup>\nconst ok = true\n</script>\n\n<template>\n  <p>{{ ok }}</p>\n  <b v-else>no</b>\n</template>\n";
    for shape in [INLINE, SEPARATE, SSR] {
        let located = |map: bool| {
            let result = try_compile(source, shape, map).expect("the script still compiles");
            result
                .errors
                .iter()
                .map(|error| {
                    let loc = error.loc.as_ref().expect("a template error is located");
                    format!(
                        "{:?} {}:{}-{}:{}",
                        error.code, loc.start_line, loc.start_column, loc.end_line, loc.end_column
                    )
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(located(true), located(false), "{shape:?}");
        assert_eq!(
            located(true),
            [r#"Some("TEMPLATE_ERROR") 5:11-8:1"#],
            "{shape:?}"
        );
    }
}

#[test]
fn carriage_returns_in_the_template_do_not_shift_the_mappings() {
    // The render function is cut out of the template module line by line. A
    // `\r\n` inside an expression must not move the segments that follow it:
    // the second `trip.title` is on line 10, at column 18.
    let source = BROKEN
        .replace(
            "{{ trip.title }}",
            "{{ trip\n      .title }}{{ trip.title }}",
        )
        .replace('\n', "\r\n");
    let client = compile(&source, SEPARATE);
    assert_eq!(
        template_segments(&client, &source),
        [
            r#"4:0 "function _sf" -> 7:10 "\r⏎  <Child l""#,
            r#"6:17 "$setup[\"Chil" -> 8:3 "Child label=""#,
            r#"6:36 "label: \"befo" -> 8:9 "label=\"befor""#,
            r#"6:44 "before\" }),⏎" -> 8:16 "before\" />\r⏎""#,
            r#"7:25 "p\", null, _t" -> 9:3 "p>{{ trip\r⏎ ""#,
            r#"7:52 "$setup.trip⏎" -> 9:8 "trip\r⏎      ""#,
            r#"8:7 "title) + _to" -> 10:7 "title }}{{ t""#,
            r#"8:33 "$setup.trip." -> 10:18 "trip.title }""#,
            r#"8:45 "title), 1 /*" -> 10:23 "title }}</p>""#,
        ],
        "{}",
        client.code
    );
    let server = compile(&source, SSR);
    assert_eq!(
        template_segments(&server, &source),
        [
            r#"4:0 "function ssr" -> 7:10 "\r⏎  <Child l""#,
            r#"6:28 "$setup.Child" -> 8:3 "Child label=""#,
            r#"7:10 "p>${_ssrInte" -> 9:3 "p>{{ trip\r⏎ ""#,
            r#"7:30 "$setup.trip⏎" -> 9:8 "trip\r⏎      ""#,
            r#"8:7 "title)}${_ss" -> 10:7 "title }}{{ t""#,
            r#"8:32 "$setup.trip." -> 10:18 "trip.title }""#,
            r#"8:44 "title)}</p><" -> 10:23 "title }}</p>""#,
        ],
        "{}",
        server.code
    );
    let inline = compile(&source, INLINE);
    assert_eq!(
        template_segments(&inline, &source),
        [
            r#"16:17 "Child, _hois" -> 8:3 "Child label=""#,
            r#"17:25 "p\", null, _t" -> 9:3 "p>{{ trip\r⏎ ""#,
            r#"17:52 "_unref(trip)" -> 9:8 "trip\r⏎      ""#,
            r#"18:7 "title) + _to" -> 10:7 "title }}{{ t""#,
            r#"18:33 "_unref(trip)" -> 10:18 "trip.title }""#,
            r#"18:46 "title), 1 /*" -> 10:23 "title }}</p>""#,
        ],
        "{}",
        inline.code
    );
}
