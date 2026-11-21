//! Compare batched diagnostics against the original projection's public mapping.
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use std::{fs, path::Path};
use vize_l0::{
    Allocator, Span,
    config::{VueDialect, VueVersion},
    line_index::utf16_len,
};
use vize_l1::{container::vue::DescriptorOptions, embed::Lang};
use vize_l1_to_l2::{
    framework::vue::jsx::{ModuleOptions, VIZE, lower_module},
    native_file::lower_sfc_native,
};
use vize_l2::component::Exec;
use vize_l4::{
    targets::ts::{
        DiagnosticMapper, MappingError, project_jsx_module, project_program,
        project_program_no_links,
        vue::{project_vue, project_vue_no_links},
    },
    write::EmitDocument,
};

mod profile;
use super::support as program;

#[derive(Default, Debug)]
struct Counts {
    sources: usize,
    invalid_utf8: usize,
    vue: usize,
    jsx: usize,
    programs: usize,
    requests: usize,
    mapped: usize,
    refused: usize,
    timing: bool,
}

fn compare<'p, M: Fn(Span) -> Result<Span, MappingError>>(
    doc: &EmitDocument,
    make: impl Fn() -> DiagnosticMapper<'p, M>,
    original: impl Fn(u32, u32) -> Result<Span, MappingError>,
    counts: &mut Counts,
) {
    let mut mapper = make();
    let text = doc.as_str();
    let units = utf16_len(text) as u32;
    let mut points = vec![0, 1, units / 2, units.saturating_sub(1), units, u32::MAX];
    if units <= 64 {
        points.extend(0..=units);
    } else {
        points.extend((0..=units).step_by((units / 16) as usize));
    }
    // Sample both ends of original mapped links, where anchors, nesting and
    // adjacent generated punctuation have distinct mapping outcomes.
    for link in doc.links().iter().step_by((doc.links().len() / 8).max(1)) {
        for byte in [link.generated.start, link.generated.end] {
            if let Some(prefix) = text.get(..byte as usize) {
                let point = utf16_len(prefix) as u32;
                points.extend([point.saturating_sub(1), point, point.saturating_add(1)]);
            }
        }
    }
    points.sort_unstable();
    points.dedup();
    if counts.timing {
        profile::original_batch(doc, &points, &make, &original, counts.sources);
    }
    for &start in points.iter().chain(points.iter().rev()) {
        for length in [0, 1, 2, 7, units.saturating_sub(start), u32::MAX] {
            let expected = original(start, length);
            assert_eq!(
                mapper.map_utf16(start, length),
                expected,
                "{start}/{length}"
            );
            counts.requests += 1;
            if expected.is_ok() {
                counts.mapped += 1;
            } else {
                counts.refused += 1;
            }
        }
    }
}

fn observe(source: &str, extension: &str, counts: &mut Counts) {
    let arena = Allocator::default();
    if extension == "vue" {
        let obs = lower_sfc_native(
            &arena,
            source,
            DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: Default::default(),
            },
        );
        // Each projection consumes its own admission proof for the same observation.
        let Some((mapped, unmapped)) = obs.admitted().zip(obs.admitted()) else {
            return;
        };
        let (Ok(on), Ok(off)) = (project_vue(mapped), project_vue_no_links(unmapped)) else {
            return;
        };
        counts.vue += 1;
        for projection in [&on, &off] {
            compare(
                projection.document(),
                || projection.diagnostic_mapper(),
                |a, b| projection.map_utf16(a, b),
                counts,
            );
        }
    } else if matches!(extension, "jsx" | "tsx") {
        let language = if extension == "tsx" {
            SourceType::tsx()
        } else {
            SourceType::jsx()
        };
        let parsed = Parser::new(&arena, source, language.with_module(true)).parse();
        if !parsed.diagnostics.is_empty() {
            return;
        }
        let scope = SemanticBuilder::new()
            .build(&parsed.program)
            .semantic
            .into_scoping();
        let options = ModuleOptions {
            dialect: &VIZE,
            default_exec: Exec::ReRender,
        };
        let Ok(roots) =
            lower_module(&arena, source, &parsed.program, Some(&scope), options).into_components()
        else {
            return;
        };
        let Ok(projection) = project_jsx_module(&parsed.program, &roots) else {
            return;
        };
        counts.jsx += 1;
        compare(
            projection.document(),
            || projection.diagnostic_mapper(),
            |a, b| projection.map_utf16(a, b),
            counts,
        );
    } else {
        if program::whole(source).is_none() {
            return;
        }
        let lang = if matches!(extension, "ts" | "cts" | "mts") {
            Lang::Ts
        } else {
            Lang::Js
        };
        let Some(file) = program::file(&arena, source, lang) else {
            return;
        };
        let (Ok(on), Ok(off)) = (project_program(&file), project_program_no_links(&file)) else {
            return;
        };
        counts.programs += 1;
        for projection in [&on, &off] {
            compare(
                projection.document(),
                || projection.diagnostic_mapper(),
                |a, b| projection.map_utf16(a, b),
                counts,
            );
        }
    }
}

#[test]
fn maps_program_sfc_and_nested_jsx_diagnostics_in_both_directions() {
    let mut counts = Counts::default();
    observe(
        "/*🎨*/ const 雪: string = 'é';\r\n雪.length",
        "ts",
        &mut counts,
    );
    observe(
        "<script setup>const label = '😀';</script>\n<template><p>{{ label }}</p></template>",
        "vue",
        &mut counts,
    );
    observe(
        "/*😀*/ const view = <div title={'雪'}><span>{'é'}</span></div>;",
        "tsx",
        &mut counts,
    );
    assert_eq!(
        (counts.programs, counts.vue, counts.jsx),
        (1, 1, 1),
        "{counts:?}"
    );
    assert!(counts.mapped > 0 && counts.refused > 0);
}

#[test]
#[ignore = "requires a NUL-delimited manifest of original Vue, JSX and script sources"]
fn original_projected_diagnostics() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let input = fs::read(std::env::var_os("VIZE_DIAGNOSTIC_INPUTS").unwrap()).unwrap();
    let mut counts = Counts {
        timing: std::env::var_os("VIZE_DIAGNOSTIC_TIMINGS").is_some(),
        ..Counts::default()
    };
    for name in input.split(|&b| b == 0).filter(|p| !p.is_empty()) {
        let path = core::str::from_utf8(name).unwrap();
        let actual = fs::read(root.join(path)).unwrap();
        counts.sources += 1;
        let Ok(text) = core::str::from_utf8(&actual) else {
            counts.invalid_utf8 += 1;
            continue;
        };
        let syntax = path.strip_suffix(".txt").unwrap_or(path);
        if let Some((_, extension)) = syntax.rsplit_once('.') {
            observe(text, extension, &mut counts);
        }
    }
    assert!(counts.requests > 0 && counts.mapped > 0 && counts.refused > 0);
    eprintln!("{counts:?}");
}
