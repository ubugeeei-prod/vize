//! Owned filesystem buffers must survive the real builders, not a later swap.

#![expect(
    clippy::disallowed_types,
    reason = "exercise std filesystem read buffers"
)]
#![expect(
    clippy::disallowed_macros,
    reason = "compare complete debug observations"
)]

use std::borrow::Cow;
use std::fmt::Debug;
use std::fs;

use vize_carton::String as CompactString;

use super::super::build::{RegisteredFile, build_registered_file, retain_source_content};
use super::VirtualProject;

fn assert_debug_equal(left: &impl Debug, right: &impl Debug) {
    assert_eq!(format!("{left:?}"), format!("{right:?}"));
}

fn assert_registered_equal(left: &RegisteredFile, right: &RegisteredFile) {
    // Exhaustive destructuring makes a new output field require an explicit check.
    let RegisteredFile {
        diagnostic_route,
        file,
        extra_virtual_files,
        original_content,
        pre_rewrite_code,
        passthrough_files,
        diagnostics,
        unchecked_javascript,
        typed_router_import,
        empty_module_exports,
        module_exports,
        star_projection,
    } = left;
    assert_eq!(diagnostic_route, &right.diagnostic_route);
    assert_debug_equal(file, &right.file);
    assert_debug_equal(extra_virtual_files, &right.extra_virtual_files);
    assert_eq!(original_content, &right.original_content);
    assert_eq!(pre_rewrite_code, &right.pre_rewrite_code);
    assert_eq!(passthrough_files, &right.passthrough_files);
    assert_debug_equal(diagnostics, &right.diagnostics);
    assert_eq!(unchecked_javascript, &right.unchecked_javascript);
    assert_eq!(typed_router_import, &right.typed_router_import);
    assert_eq!(empty_module_exports, &right.empty_module_exports);
    assert_eq!(module_exports, &right.module_exports);
    let star = |value: &Option<super::super::star_exports::StarProjection>| {
        value
            .as_ref()
            .map(|value| format!("{:?}", (&value.code, &value.source_map, &value.stars)))
    };
    assert_eq!(star(star_projection), star(&right.star_projection));
}

fn project_observation(project: &VirtualProject) -> std::string::String {
    let mut originals = project.original_contents.iter().collect::<Vec<_>>();
    originals.sort_by_key(|(path, _)| *path);
    let mut pre_rewrite = project.pre_rewrite_code.iter().collect::<Vec<_>>();
    pre_rewrite.sort_by_key(|(path, _)| *path);
    let mut passthrough = project.passthrough_files.iter().collect::<Vec<_>>();
    passthrough.sort_by_key(|(path, _)| *path);
    format!(
        "{:?}",
        (
            project.virtual_files_sorted(),
            originals,
            pre_rewrite,
            passthrough,
            project.diagnostics(),
        )
    )
}

fn source_for(extension: &str, comment_repetitions: usize) -> std::string::String {
    let comment = "雪🌸 authored bytes\r\n".repeat(comment_repetitions);
    match extension {
        "vue" => format!(
            "<script setup lang=\"ts\">\r\nconst label: string = '雪🌸'\r\n</script>\r\n<template><div>{{{{ label }}}}</div></template>\r\n<!--{comment}-->\r\n"
        ),
        "tsx" | "jsx" => format!(
            "/*{comment}*/\r\nexport default function App() {{ return <div>{{'雪🌸'}}</div> }}\r\n"
        ),
        "d.ts" => format!("/*{comment}*/\r\nexport declare const label: string;\r\n"),
        _ => format!("/*{comment}*/\r\nexport const label = '雪🌸';\r\n"),
    }
}

#[test]
fn owned_source_conversion_reuses_long_buffers_and_inlines_short_capacity() {
    for length in [64, 64 * 1024, 4 * 1024 * 1024] {
        let mut source = std::string::String::with_capacity(length + 4096);
        source.extend(std::iter::repeat_n('x', length));
        let (pointer, capacity) = (source.as_ptr(), source.capacity());
        let retained = retain_source_content(Cow::Owned(source));
        assert_eq!(retained.as_ptr(), pointer);
        assert_eq!(retained.len(), length);
        assert_eq!(retained.capacity(), capacity);
        assert!(retained.bytes().all(|byte| byte == b'x'));
    }
    for text in ["", "a", "雪🌸"] {
        let mut source = std::string::String::with_capacity(1024 * 1024);
        source.push_str(text);
        let retained = retain_source_content(Cow::Owned(source));
        assert_eq!(retained.as_str(), text);
        assert_eq!(retained.capacity(), CompactString::new(text).capacity());
        assert!(retained.capacity() < 1024 * 1024);
    }
}

#[test]
fn owned_source_real_builders_retain_large_input_allocation_and_whole_outputs() {
    let fixture = tempfile::TempDir::new().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(fixture.path());
    for (extension, jsx) in [
        ("vue", false),
        ("ts", false),
        ("d.ts", false),
        ("mts", false),
        ("tsx", false),
        ("tsx", true),
        ("jsx", true),
    ] {
        for repetitions in [1, 4096, 65536] {
            let path = root.join(format!("App.{extension}"));
            fs::write(&path, source_for(extension, repetitions)).unwrap();
            let source = fs::read_to_string(&path).unwrap();
            let (pointer, length, capacity) = (source.as_ptr(), source.len(), source.capacity());
            let mut project = VirtualProject::new(&root).unwrap();
            project.set_jsx_typecheck(jsx);
            let borrowed = build_registered_file(
                &path,
                Cow::Borrowed(&source),
                project.path_build_context(&path, None),
            )
            .unwrap();
            // This is the old constructor's real duplicate while input is alive.
            assert_ne!(borrowed.original_content.as_ptr(), pointer);
            let owned = build_registered_file(
                &path,
                Cow::Owned(source),
                project.path_build_context(&path, None),
            )
            .unwrap();
            assert_eq!(owned.original_content.as_ptr(), pointer);
            assert_eq!(owned.original_content.len(), length);
            assert_eq!(owned.original_content.capacity(), capacity);
            assert_registered_equal(&borrowed, &owned);
            project.absorb_registered_file(owned).unwrap();
            let retained = project.original_contents.values().next().unwrap();
            assert_eq!(retained.as_ptr(), pointer);
            assert_eq!(retained.len(), length);
            eprintln!(
                "owned-read-buffer extension={extension} jsx={jsx} source_bytes={length} capacity={capacity} retained_input_pointer=true borrowed_original_duplicate_bytes={length}"
            );
        }
    }
}

#[test]
fn owned_source_all_four_filesystem_read_routes_match_borrowed_revision() {
    let fixture = tempfile::TempDir::new().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(fixture.path());
    let sources = ["vue", "ts", "d.ts", "tsx"].map(|extension| {
        (
            root.join(format!("App.{extension}")),
            source_for(extension, 32),
        )
    });
    let paths = sources
        .iter()
        .map(|(path, _)| path.clone())
        .collect::<Vec<_>>();
    for (path, content) in &sources {
        fs::write(path, content).unwrap();
    }
    for jsx in [false, true] {
        let mut borrowed = VirtualProject::new(&root).unwrap();
        borrowed.set_jsx_typecheck(jsx);
        borrowed
            .register_paths_with_content(
                sources
                    .iter()
                    .map(|(path, text)| (path.as_path(), text.as_str())),
            )
            .unwrap();
        let expected = project_observation(&borrowed);
        for route in 0..3 {
            let mut project = VirtualProject::new(&root).unwrap();
            project.set_jsx_typecheck(jsx);
            match route {
                0 => {
                    for path in &paths {
                        project.register_path(path).unwrap();
                    }
                }
                1 => project.register_paths(&paths).unwrap(),
                _ => assert!(project.register_canonical_paths_lenient(&paths)),
            }
            assert_eq!(project_observation(&project), expected);
        }
    }
    for extension in ["ts", "d.ts", "tsx"] {
        let path = root.join(format!("App.{extension}"));
        let content = fs::read_to_string(&path).unwrap();
        let mut borrowed = VirtualProject::new(&root).unwrap();
        let source_type = super::source_type_for_path(&path).unwrap();
        borrowed
            .register_script_file(&path, &content, source_type)
            .unwrap();
        let mut owned = VirtualProject::new(&root).unwrap();
        owned.register_ts_file(&path).unwrap();
        assert_eq!(project_observation(&owned), project_observation(&borrowed));
    }
}

#[test]
fn owned_source_borrowed_unsaved_text_and_replacement_remain_independent_of_disk() {
    let fixture = tempfile::TempDir::new().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(fixture.path());
    for extension in ["vue", "ts", "d.ts"] {
        let path = root.join(format!("App.{extension}"));
        fs::write(&path, source_for(extension, 1)).unwrap();
        let mut project = VirtualProject::new(&root).unwrap();
        project.register_path(&path).unwrap();
        let mut unsaved = source_for(extension, 5);
        let expected = unsaved.clone();
        match extension {
            "vue" => project.register_vue_file(&path, &unsaved).unwrap(),
            "d.ts" => project.register_declaration_file(&path, &unsaved).unwrap(),
            _ => project.register_path_with_content(&path, &unsaved).unwrap(),
        }
        unsaved.clear();
        let file = project.find_by_original(&path).unwrap();
        assert_eq!(
            project.original_contents[&file.virtual_path].as_str(),
            expected
        );
    }
}

#[test]
fn owned_source_failure_and_lenient_registration_preserve_borrowed_results() {
    let fixture = tempfile::TempDir::new().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(fixture.path());
    let mut project = VirtualProject::new(&root).unwrap();
    for (name, content) in [
        ("Broken.vue", "<script setup lang=\"ts\">const =</script>"),
        ("Broken.tsx", "export default () => <div>"),
        ("Unsupported.txt", "雪🌸"),
    ] {
        let path = root.join(name);
        let borrowed = build_registered_file(
            &path,
            Cow::Borrowed(content),
            project.path_build_context(&path, None),
        );
        let owned = build_registered_file(
            &path,
            Cow::Owned(content.into()),
            project.path_build_context(&path, None),
        );
        match (borrowed, owned) {
            (Ok(left), Ok(right)) => assert_registered_equal(&left, &right),
            (Err(left), Err(right)) => assert_debug_equal(&left, &right),
            _ => panic!("ownership changed build acceptance"),
        }
    }
    let valid = root.join("Valid.ts");
    fs::write(&valid, "export const value = 1;\r\n").unwrap();
    let missing = root.join("Missing.ts");
    assert!(project.register_canonical_paths_lenient(&[missing.clone(), valid.clone()]));
    let mut expected = VirtualProject::new(&root).unwrap();
    expected
        .register_path_with_content(&valid, "export const value = 1;\r\n")
        .unwrap();
    assert_eq!(
        project_observation(&project),
        project_observation(&expected)
    );
    assert!(project.register_path(&missing).is_err());
    assert!(project.register_ts_file(&missing).is_err());
    assert_eq!(
        project_observation(&project),
        project_observation(&expected)
    );
}
