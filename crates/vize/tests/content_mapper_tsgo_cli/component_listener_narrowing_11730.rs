//! Authored branch conditions survive component-listener closure boundaries.

use std::{fs, path::PathBuf};

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};

const FIXTURE: &str = "tests/_fixtures/differential/typechecker/component-listener-narrowing-11730";
const SHADOW_FIXTURE: &str = "tests/_fixtures/differential/typechecker/listener-shadow-guard-11749";

fn diagnostics(output: &std::process::Output) -> Vec<(String, u32)> {
    std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .filter_map(|line| line.split_once(": error TS"))
        .map(|(position, message)| {
            let (code, _) = message.split_once(':').unwrap();
            (position.to_owned(), code.parse().unwrap())
        })
        .collect()
}

fn position(file: &str, source: &str, needle: &str, offset: usize) -> String {
    let byte = source.find(needle).unwrap() + offset;
    let prefix = &source[..byte];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = prefix.rsplit('\n').next().unwrap().encode_utf16().count() + 1;
    format!("{file}({line},{column})")
}

#[test]
fn component_listener_branches_keep_narrowing_and_authored_body_errors() {
    let Some(native) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        assert_ne!(
            std::env::var("VIZE_TEST_REQUIRE_COMPONENT_LISTENER_NARROWING").as_deref(),
            Ok("1"),
            "required component-listener narrowing host is missing"
        );
        eprintln!("Content Mapper provider not selected; no semantic qualification");
        return;
    };
    let fixture = workspace_root().join(FIXTURE);
    let row = fs::read_to_string(fixture.join("Row.vue.txt")).unwrap();
    let union = fs::read_to_string(fixture.join("Union.vue.txt")).unwrap();
    let nullable = fs::read_to_string(fixture.join("Nullable.vue.txt")).unwrap();
    let config = fs::read(fixture.join("tsconfig.json.txt")).unwrap();
    let wrong_union = union.replace("row.item.slug", "row.item.missing");
    let wrong_nullable = nullable.replace("draft.a = n", "draft.missing = n");
    let wrong_payload = row.replace("v: string", "v: number");
    let multiline = "<script setup lang=\"ts\">\nimport Row from './Row.vue';\ntype Item = {kind:'header';label:string}|{kind:'entry';slug:string};\ndefineProps<{items:Item[]}>();\nconst open=(slug:string,value:string)=>void[slug,value];\n</script>\n<template>\n<template v-for=\"item in items\">\n<Row v-if=\"\n item.kind !== 'header' &amp;&amp;\n // preserved comment\n item.slug.length > 0\n\" @act=\"open(item.slug, $event)\" />\n</template>\n</template>";
    let wrong_multiline = multiline.replace("$event", "123");
    let slot_capture = nullable.replace(
        "<Row v-for=\"n in ['x', 'y']\" :key=\"n\" @act=\"draft.a = n\" />",
        "<template v-for=\"n in ['x', 'y']\" :key=\"n\"><slot :value=\"draft.a\" /><Row @act=\"draft.a = n\" /></template>",
    );
    let wrong_slot_capture = slot_capture.replace(":value=\"draft.a\"", ":value=\"draft.missing\"");
    let explicit_callback = union.replace(
        "open(row.item.slug, $event)",
        "(event: string) => open(row.item.slug, event)",
    );
    assert_ne!(wrong_union, union);
    assert_ne!(wrong_nullable, nullable);
    assert_ne!(wrong_payload, row);
    assert_ne!(slot_capture, nullable);
    assert_ne!(wrong_slot_capture, slot_capture);
    assert_ne!(explicit_callback, union);
    let cases = [
        (
            "reported",
            row.clone(),
            union.clone(),
            nullable.clone(),
            Vec::new(),
        ),
        (
            "wrong-union-member",
            row.clone(),
            wrong_union.clone(),
            nullable.clone(),
            vec![(
                position("Union.vue", &wrong_union, "row.item.missing", 9),
                2339,
            )],
        ),
        (
            "wrong-nullable-member",
            row.clone(),
            union.clone(),
            wrong_nullable.clone(),
            vec![(
                position("Nullable.vue", &wrong_nullable, "draft.missing", 6),
                2339,
            )],
        ),
        (
            "wrong-listener-payload",
            wrong_payload,
            union.clone(),
            nullable.clone(),
            vec![(position("Union.vue", &union, "$event", 0), 2345)],
        ),
        (
            "multiline-generated-guard",
            row.clone(),
            multiline.to_owned(),
            nullable.clone(),
            Vec::new(),
        ),
        (
            "multiline-authored-error",
            row.clone(),
            wrong_multiline.clone(),
            nullable.clone(),
            vec![(position("Union.vue", &wrong_multiline, "123", 0), 2345)],
        ),
        (
            "slot-capturing-callback",
            row.clone(),
            union.clone(),
            slot_capture,
            Vec::new(),
        ),
        (
            "slot-capturing-authored-error",
            row.clone(),
            union.clone(),
            wrong_slot_capture.clone(),
            vec![(
                position("Nullable.vue", &wrong_slot_capture, "draft.missing", 6),
                2339,
            )],
        ),
        (
            "authored-callback-stays-deferred",
            row.clone(),
            explicit_callback.clone(),
            nullable.clone(),
            vec![(
                position("Union.vue", &explicit_callback, "row.item.slug", 9),
                2339,
            )],
        ),
    ];
    let projects = workspace_root().join("target/vize-tests/tests");
    fs::create_dir_all(&projects).unwrap();
    let mut observed = Vec::new();
    for (phase, row, union, nullable, expected) in cases {
        let project = tempfile::Builder::new()
            .prefix("component-listener-narrowing-11730-")
            .tempdir_in(&projects)
            .unwrap();
        install_mapper_manifest(project.path());
        install_vue_package(project.path());
        for (name, source) in [
            ("Row.vue", row),
            ("Union.vue", union),
            ("Nullable.vue", nullable),
        ] {
            fs::write(project.path().join(name), source).unwrap();
        }
        fs::write(project.path().join("tsconfig.json"), &config).unwrap();
        let output = check_project(&native, project.path(), "tsconfig.json");
        println!(
            "component-listener-narrowing-11730 {phase}: {}",
            output_text(&output)
        );
        observed.push((phase, output, expected));
    }
    for (phase, output, expected) in observed {
        assert_eq!(
            output.status.code(),
            Some(if expected.is_empty() { 0 } else { 2 }),
            "{phase}: {}",
            output_text(&output)
        );
        assert_eq!(
            diagnostics(&output),
            expected,
            "{phase}: {}",
            output_text(&output)
        );
        if expected.is_empty() {
            assert_eq!(output.stdout, b"", "{phase}: {}", output_text(&output));
        }
        assert_eq!(output.stderr, b"", "{phase}: {}", output_text(&output));
    }
}

#[test]
fn shadowed_listener_guard_keeps_binding_identity() {
    let Some(native) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("Content Mapper provider not selected; no semantic qualification");
        return;
    };
    let fixture = workspace_root().join(SHADOW_FIXTURE);
    let row = fs::read_to_string(fixture.join("Row.vue.txt")).unwrap();
    let config = fs::read(fixture.join("tsconfig.json.txt")).unwrap();
    let shadowed = fs::read_to_string(fixture.join("Shadowed.vue.txt")).unwrap();
    let callable_control = fs::read_to_string(fixture.join("Callable.vue.txt")).unwrap();

    for (phase, main, expected) in [
        (
            "shadowed",
            shadowed.clone(),
            vec![(position("Main.vue", &shadowed, "@act", 1), 2322)],
        ),
        ("callable-control", callable_control, Vec::new()),
    ] {
        let parent = workspace_root().join("target/vize-tests/tests");
        fs::create_dir_all(&parent).unwrap();
        let project = tempfile::Builder::new()
            .prefix("listener-shadow-guard-11749-")
            .tempdir_in(parent)
            .unwrap();
        install_mapper_manifest(project.path());
        install_vue_package(project.path());
        fs::write(project.path().join("Row.vue"), &row).unwrap();
        fs::write(project.path().join("Main.vue"), &main).unwrap();
        fs::write(project.path().join("tsconfig.json"), &config).unwrap();
        let mut output = check_project(&native, project.path(), "tsconfig.json");
        // A freshly started external host can lose its first mapper handshake.
        // Retry only that transport diagnostic; the identical semantic result
        // below must still match exactly.
        if output.stdout.windows(7).any(|window| window == b"TS18069") {
            output = check_project(&native, project.path(), "tsconfig.json");
        }
        assert_eq!(
            output.status.code(),
            Some(if expected.is_empty() { 0 } else { 2 }),
            "{phase}: {}",
            output_text(&output)
        );
        assert_eq!(
            diagnostics(&output),
            expected,
            "{phase}: {}",
            output_text(&output)
        );
        assert_eq!(output.stderr, b"", "{phase}: {}", output_text(&output));
    }
}
