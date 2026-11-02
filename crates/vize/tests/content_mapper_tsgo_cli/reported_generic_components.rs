//! Reported generic aliases and global template refs retain their authored types.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use vize_l0::cstr;

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, workspace_root,
};

const SLOT_INPUT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/generic-alias-slots-11360/input.json"
));
const REF_INPUT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/generic-global-template-ref-11368/input.json"
));

fn input_files(input: &Value, root: &Path) {
    for file in input["files"].as_array().unwrap() {
        let target = root.join(file["path"].as_str().unwrap());
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(target, file["source"].as_str().unwrap()).unwrap();
    }
}

fn observe(tsgo: &Path, root: &Path, input: &Value, identity: Value) -> Value {
    let output = check_project(tsgo, root, "tsconfig.json");
    let files = input["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| {
            let name = file["path"].as_str().unwrap();
            let path = root.join(name);
            let source = std::fs::read_to_string(&path).unwrap();
            let producer = name.ends_with(".vue").then(|| {
                let generated =
                    vize_canon::generate_vue_content_mapper_transform(&path, &source).unwrap();
                json!({
                    "kind": "direct Rust content-mapper API observation",
                    "scope": "not claimed byte-identical to the native host packet",
                    "virtualText": generated.text,
                })
            });
            json!({"path":name,"source":source,"directProducer":producer})
        })
        .collect::<Vec<_>>();
    json!({
        "identity":identity,
        "projectRoot":root,
        "sourceFiles":files,
        "status":output.status.code(),
        "success":output.status.success(),
        "stdoutBytes":&output.stdout,
        "stderrBytes":&output.stderr,
        "stdout":std::str::from_utf8(&output.stdout).unwrap(),
        "stderr":std::str::from_utf8(&output.stderr).unwrap(),
    })
}

fn collect_slots(tsgo: &Path, project: &Path, input: &Value, outcomes: &mut Vec<Value>) {
    let original_child = input["files"][2]["source"].as_str().unwrap();
    let original_probe = input["files"][3]["source"].as_str().unwrap();
    for constrained in [false, true] {
        input_files(input, project);
        let child = if constrained {
            "<script setup lang=\"ts\" generic=\"T extends string\">\ntype Item = { foo: T };\ndefineProps<{ items: Item[] }>();\ndefineSlots<{ default: (props: { item: Item }) => unknown }>();\n</script>\n<template><slot :item=\"{ foo: 'a' } as Item\" /></template>\n"
        } else {
            original_child
        };
        std::fs::write(project.join("src/Child.vue"), child).unwrap();
        let valid = if constrained {
            original_probe
                .replace("{ bar: \"a\", baz: 1 }", "{ foo: \"a\" }")
                .replace("item.baz", "item.foo")
        } else {
            original_probe.into()
        };
        let wrong = if constrained {
            valid.replace("item.foo", "item.foo.bar")
        } else {
            valid.replace("item.baz", "item.nope")
        };
        for (phase, bad) in [
            ("original", false),
            ("wrong", true),
            ("repair", false),
            ("restore", true),
        ] {
            std::fs::write(
                project.join("src/Probe.vue"),
                if bad { &wrong } else { &valid },
            )
            .unwrap();
            outcomes.push(observe(tsgo, project, input, json!({
                "issue":11360,"variant":if constrained {"constrained-no-default-control"} else {"faithful-default-original"},
                "phase":phase,"bad":bad,"line":10,"column":if constrained {19} else {15},
                "expectedCode":2339,"expectedProperty":if constrained {"bar"} else {"nope"},
            })));
        }
    }
}

fn collect_refs(tsgo: &Path, project: &Path, input: &Value, outcomes: &mut Vec<Value>) {
    let original_child = input["files"][2]["source"].as_str().unwrap();
    let original_probe = input["files"][4]["source"].as_str().unwrap();
    for generic in [true, false] {
        input_files(input, project);
        let child = if generic {
            original_child.into()
        } else {
            original_child.replace(" generic=\"T\"", "")
        };
        std::fs::write(project.join("src/Child.vue"), child).unwrap();
        for (phase, replacement, code, column, property) in [
            ("original", "foo()", 0, 14, ""),
            ("wrong", "nope()", 2339, 14, "nope"),
            ("repair", "foo()", 0, 14, ""),
            ("wrong-argument", "foo(1)", 2554, 18, ""),
            ("restore", "nope()", 2339, 14, "nope"),
        ] {
            let source = original_probe.replace("foo()", replacement);
            std::fs::write(project.join("src/Probe.vue"), source).unwrap();
            outcomes.push(observe(tsgo, project, input, json!({
                "issue":11368,"variant":if generic {"faithful-generic-original"} else {"non-generic-control"},
                "phase":phase,"bad":code!=0,"line":5,"column":column,
                "expectedCode":code,"expectedProperty":property,
            })));
        }
    }
}

#[test]
fn reported_generic_slots_and_global_refs_keep_authored_members() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        assert!(
            std::env::var_os("VIZE_REQUIRE_REPORTED_GENERIC_COMPONENTS").is_none(),
            "required reported-generic native provider is missing: {TSGO_ENV}"
        );
        eprintln!("skipping reported generic Content Mapper controls: {TSGO_ENV} is not set");
        return;
    };
    assert!(
        tsgo.is_file(),
        "{TSGO_ENV} must name the actual source-test provider"
    );
    let slots: Value = serde_json::from_str(SLOT_INPUT).unwrap();
    let refs: Value = serde_json::from_str(REF_INPUT).unwrap();
    assert_eq!(slots["issue"], 11360);
    assert_eq!(refs["issue"], 11368);
    assert_eq!(slots["files"].as_array().unwrap().len(), 4);
    assert_eq!(refs["files"].as_array().unwrap().len(), 5);
    let directory = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&directory).unwrap();
    let project = tempfile::Builder::new()
        .prefix("reported-generic-components-")
        .tempdir_in(directory)
        .unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());
    let mut outcomes = Vec::new();
    collect_slots(&tsgo, project.path(), &slots, &mut outcomes);
    // A separate project prevents the ref augmentation from changing slot controls.
    let ref_project = tempfile::Builder::new()
        .prefix("reported-generic-global-refs-")
        .tempdir_in(workspace_root().join("target/vize-tests/tests"))
        .unwrap();
    install_mapper_manifest(ref_project.path());
    install_vue_package(ref_project.path());
    collect_refs(&tsgo, ref_project.path(), &refs, &mut outcomes);
    let capture = json!({
        "schema":"vize.reported-generic-component-source-observations.v1",
        "originalInputs":[slots,refs],"requestedProvider":tsgo,
        "actualVueManifest":std::fs::read_to_string(project.path().join("node_modules/vue/package.json")).unwrap(),
        "actualMapperManifest":std::fs::read_to_string(project.path().join("node_modules/vize/package.json")).unwrap(),
        "scope":"Compiled source CLI Content Mapper with actual selected provider; not installed vize0.638, reporter tsrs tuple or Windows acceptance.",
        "observations":outcomes,
    });
    let evidence = std::env::var_os("VIZE_REPORTED_GENERIC_EVIDENCE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root().join("target/vize-reported-generic-components"));
    std::fs::create_dir_all(&evidence).unwrap();
    std::fs::write(
        evidence.join("observations.json"),
        serde_json::to_vec_pretty(&capture).unwrap(),
    )
    .unwrap();
    eprintln!(
        "11360/11368 complete observations: {}",
        serde_json::to_string_pretty(&capture).unwrap()
    );
    assert_eq!(capture["observations"].as_array().unwrap().len(), 18);
    // Execute every original/wrong/repair/restore state before asserting any.
    for observation in capture["observations"].as_array().unwrap() {
        let identity = &observation["identity"];
        assert_eq!(observation["stderr"], "", "{observation:#}");
        if !identity["bad"].as_bool().unwrap() {
            assert_eq!(observation["status"], 0, "{observation:#}");
            assert_eq!(observation["stdout"], "", "{observation:#}");
        } else {
            assert_eq!(observation["status"], 2, "{observation:#}");
            let stdout = observation["stdout"].as_str().unwrap();
            let lines = stdout.lines().collect::<Vec<_>>();
            assert_eq!(
                lines.len(),
                1,
                "complete diagnostic vector: {observation:#}"
            );
            let prefix = cstr!(
                "src/Probe.vue({},{}): error TS{}:",
                identity["line"],
                identity["column"],
                identity["expectedCode"]
            );
            assert!(stdout.starts_with(prefix.as_str()), "{observation:#}");
            if identity["expectedCode"] == 2339 {
                let property = identity["expectedProperty"].as_str().unwrap();
                assert!(
                    stdout.contains(cstr!("Property '{property}' does not exist").as_str()),
                    "{observation:#}"
                );
            } else {
                assert_eq!(
                    stdout,
                    cstr!("{prefix} Expected 0 arguments, but got 1.\n").as_str(),
                    "{observation:#}"
                );
            }
        }
    }
}
