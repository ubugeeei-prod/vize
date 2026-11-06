//! #11496: `vize check` sees through a vendored component declaration.
#![cfg(test)]
use crate::support::corsa_requirement;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use vize_l0::cstr;

mod checked_report;

const CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/vendored-vue-declaration-11496/tsconfig.json.txt"
));

const FOO: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/vendored-vue-declaration-11496/Foo.vue.txt"
));

/// The reported consumer: a wrong value for a `count: number` prop.
const PROBE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/vendored-vue-declaration-11496/Probe.vue.txt"
));
const PROBE_TAG: &str = r#"<Foo :count="'x'" />"#;

const AUTHORED_VUE_TS_CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/authored-vue-ts-declaration-11539/tsconfig.json.txt"
));

const AUTHORED_VUE_TS_FOO: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/authored-vue-ts-declaration-11539/Foo.vue.txt"
));

const AUTHORED_VUE_TS_PROBE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/authored-vue-ts-declaration-11539/Probe.vue.txt"
));

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap()
}

fn project(dir: &Path, config: &str, files: &[(&str, &str)]) {
    fs::create_dir_all(dir).unwrap();
    fs::write(dir.join("tsconfig.json"), config).unwrap();
    for (name, source) in files {
        let path = dir.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }
    let provider = root().join("tests/node_modules");
    #[cfg(unix)]
    std::os::unix::fs::symlink(provider, dir.join("node_modules")).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(provider, dir.join("node_modules")).unwrap();
}

/// Exit code and combined output.
fn run(command: &mut Command) -> (Option<i32>, vize_l0::String) {
    let output = command.output().unwrap();
    let mut text = vize_l0::String::from(std::str::from_utf8(&output.stdout).unwrap());
    text.push_str(std::str::from_utf8(&output.stderr).unwrap());
    (output.status.code(), text)
}

fn vue_tsc(dir: &Path) -> Command {
    let mut command = Command::new("node");
    command
        .current_dir(dir)
        .arg(root().join("tests/node_modules/vue-tsc/bin/vue-tsc.js"));
    command
}

/// `(line, column, code)` from `vue-tsc --pretty false` and from `vize check`.
fn diagnostics(corsa: &Path, dir: &Path) -> [Vec<(u32, u32, u32)>; 2] {
    diagnostics_with_file_count(corsa, dir, 1)
}

fn diagnostics_with_file_count(
    corsa: &Path,
    dir: &Path,
    expected_file_count: usize,
) -> [Vec<(u32, u32, u32)>; 2] {
    let (reference_code, reference) =
        run(vue_tsc(dir).args(["--noEmit", "-p", "tsconfig.json", "--pretty", "false"]));
    let reference: Vec<(u32, u32, u32)> = reference
        .lines()
        .filter_map(|line| {
            let (site, rest) = line
                .strip_prefix("src/Probe.vue(")?
                .split_once("): error TS")?;
            let (line, column) = site.split_once(',')?;
            let (code, _) = rest.split_once(':')?;
            Some((line.parse().ok()?, column.parse().ok()?, code.parse().ok()?))
        })
        .collect();
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(dir)
        .env("CORSA_PATH", corsa)
        .args(["check", "--format", "json"])
        .output()
        .unwrap();
    let checked =
        checked_report::diagnostic_sites_with_file_count(&output.stdout, expected_file_count);
    // A crashed checker must not pass for a clean one.
    assert_eq!(
        reference_code,
        Some(if reference.is_empty() { 0 } else { 2 })
    );
    assert_eq!(
        output.status.code(),
        Some(if checked.is_empty() { 0 } else { 1 }),
        "{}\n{}",
        std::str::from_utf8(&output.stdout).unwrap(),
        std::str::from_utf8(&output.stderr).unwrap(),
    );
    [reference, checked]
}

#[test]
fn consumers_of_emitted_component_declarations_report_vue_tsc_diagnostics() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let dir = tempfile::Builder::new()
        .prefix("vendored-vue-declaration-11496-")
        .tempdir()
        .unwrap();
    let producer = dir.path().join("producer");
    project(&producer, CONFIG, &[("src/Foo.vue", FOO)]);
    let (code, emitted) = run(Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&producer)
        .env("CORSA_PATH", &corsa)
        .args(["check", "--declaration", "--declaration-dir", "../vize"])
        .arg("src/Foo.vue"));
    assert_eq!(code, Some(0), "{emitted}");
    assert!(dir.path().join("vize/Foo.vue.d.ts").is_file(), "{emitted}");
    let (code, emitted) = run(vue_tsc(&producer).args([
        "-p",
        "tsconfig.json",
        "--declaration",
        "--emitDeclarationOnly",
        "--noEmit",
        "false",
        "--rootDir",
        "src",
        "--outDir",
        "../vue-tsc",
    ]));
    assert_eq!(code, Some(0), "{emitted}");
    assert!(
        dir.path().join("vue-tsc/Foo.vue.d.ts").is_file(),
        "{emitted}"
    );
    let cases: [(&str, &str, Vec<(u32, u32, u32)>); 4] = [
        (
            "valid",
            r#"<Foo :count="1" @change="(value: number) => value" />"#,
            vec![],
        ),
        ("wrong-value", PROBE_TAG, vec![(6, 9, 2322)]),
        ("missing-prop", "<Foo />", vec![(6, 4, 2345)]),
        (
            "wrong-listener",
            r#"<Foo :count="1" @change="(value: string) => value" />"#,
            vec![(6, 20, 2322)],
        ),
    ];
    for emitter in ["vize", "vue-tsc"] {
        for (id, tag, expected) in &cases {
            let consumer = dir.path().join(cstr!("{emitter}-{id}").as_str());
            let probe = PROBE.replace(PROBE_TAG, tag);
            project(&consumer, CONFIG, &[("src/Probe.vue", probe.as_str())]);
            fs::create_dir_all(consumer.join("src/vendor")).unwrap();
            for entry in fs::read_dir(dir.path().join(emitter)).unwrap() {
                let entry = entry.unwrap().path();
                fs::copy(
                    &entry,
                    consumer.join("src/vendor").join(entry.file_name().unwrap()),
                )
                .unwrap();
            }
            let [reference, checked] = diagnostics(&corsa, &consumer);
            assert_eq!(&reference, expected, "vue-tsc {emitter} {id}");
            assert_eq!(&checked, expected, "vize check {emitter} {id}");
        }
    }
}

/// #11539: an explicit authored `.vue.ts` import must use TypeScript's
/// declaration substitution even when the sibling SFC owns a generated mirror
/// with the same spelling.
#[test]
fn authored_vue_ts_imports_do_not_collapse_into_the_sfc_mirror() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let dir = tempfile::Builder::new()
        .prefix("authored-vue-ts-declaration-11539-")
        .tempdir()
        .unwrap();
    let producer = dir.path().join("producer");
    project(
        &producer,
        AUTHORED_VUE_TS_CONFIG,
        &[("src/Foo.vue", AUTHORED_VUE_TS_FOO)],
    );
    let (code, emitted) = run(Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&producer)
        .env("CORSA_PATH", &corsa)
        .args(["check", "--declaration", "--declaration-dir", "../emitted"])
        .arg("src/Foo.vue"));
    assert_eq!(code, Some(0), "{emitted}");

    let consumer = dir.path().join("consumer");
    project(
        &consumer,
        AUTHORED_VUE_TS_CONFIG,
        &[
            ("src/Probe.vue", AUTHORED_VUE_TS_PROBE),
            ("src/Foo.vue", AUTHORED_VUE_TS_FOO),
        ],
    );
    for entry in fs::read_dir(dir.path().join("emitted")).unwrap() {
        let entry = entry.unwrap().path();
        fs::copy(
            &entry,
            consumer.join("src").join(entry.file_name().unwrap()),
        )
        .unwrap();
    }

    let expected = vec![(6, 9, 2322)];
    let [reference, checked] = diagnostics_with_file_count(&corsa, &consumer, 2);
    assert_eq!(reference, expected, "vue-tsc authored .vue.ts");
    assert_eq!(checked, expected, "vize check authored .vue.ts");
}

/// A `.ts` spelling that TypeScript substitutes with a declaration must keep
/// that declaration's own relative imports.
#[test]
fn ts_spelled_declaration_imports_keep_their_relative_dependencies() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let dir = tempfile::Builder::new()
        .prefix("ts-spelled-declaration-11496-")
        .tempdir()
        .unwrap();
    let config = CONFIG.replace(
        "\"strict\": true,",
        "\"strict\": true,\n    \"allowImportingTsExtensions\": true,",
    );
    for (id, value, expected) in [
        ("valid", "const w: number", vec![]),
        ("wrong", "const w: string", vec![(3, 7, 2322)]),
    ] {
        let consumer = dir.path().join(id);
        let probe = cstr!(
            "<script setup lang=\"ts\">\nimport {{ v }} from \"./vendor/types.ts\";\n{value} = v.count;\n</script>\n\n<template>\n  <div>{{{{ w }}}}</div>\n</template>\n"
        );
        project(
            &consumer,
            &config,
            &[
                ("src/Probe.vue", probe.as_str()),
                (
                    "src/vendor/types.d.ts",
                    "import type { Dep } from \"./dep.js\";\nexport declare const v: Dep;\n",
                ),
                (
                    "src/vendor/dep.d.ts",
                    "export type Dep = { count: number };\n",
                ),
            ],
        );
        let [reference, checked] = diagnostics(&corsa, &consumer);
        assert_eq!(reference, expected, "vue-tsc {id}");
        assert_eq!(checked, expected, "vize check {id}");
    }
}
