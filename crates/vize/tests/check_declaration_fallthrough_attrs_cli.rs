//! #11454: an emitted declaration lets vue-tsc templates pass undeclared attributes.
#![cfg(test)]
use crate::support::corsa_requirement;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use vize_l0::String;

const CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/declaration-fallthrough-attrs-11454/tsconfig.json.txt"
));

const FOO: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/declaration-fallthrough-attrs-11454/Foo.vue.txt"
));

const PROBE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/declaration-fallthrough-attrs-11454/Probe.vue.txt"
));

/// A non-strict vue-tsc program whose `.tsx` file sits beside a `.vue` file.
const MIXED_CONFIG: &str = r#"{
  "compilerOptions": {
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "noEmit": true,
    "skipLibCheck": true,
    "strict": true,
    "target": "ESNext",
    "jsx": "preserve",
    "jsxImportSource": "vue"
  },
  "include": ["src/**/*"]
}
"#;

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap()
}

fn link_provider(project: &Path) {
    let provider = root().join("tests/node_modules");
    #[cfg(unix)]
    std::os::unix::fs::symlink(provider, project.join("node_modules")).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(provider, project.join("node_modules")).unwrap();
}

/// `(id, Foo.vue, Probe.vue, consumer tsconfig, vue-tsc (line, column, code))`.
type Case<'a> = (&'a str, &'a str, String, &'a str, Vec<(u32, u32, u32)>);

/// `(line, column, code)` of each vue-tsc diagnostic in the consumer.
fn consume(corsa: &Path, foo: &str, probe: &str, config: &str) -> Vec<(u32, u32, u32)> {
    let dir = tempfile::Builder::new()
        .prefix("declaration-fallthrough-11454-")
        .tempdir()
        .unwrap();
    let producer = dir.path().join("producer");
    let consumer = dir.path().join("consumer");
    let mixed = config == MIXED_CONFIG;
    let (file, template) = if mixed {
        ("Probe.tsx", PROBE)
    } else {
        ("Probe.vue", probe)
    };
    for (project, file, source, config) in [
        (&producer, "Foo.vue", foo, CONFIG),
        (&consumer, "Probe.vue", template, config),
        (&consumer, file, probe, config),
    ] {
        fs::create_dir_all(project.join("src")).unwrap();
        fs::write(project.join("tsconfig.json"), config).unwrap();
        fs::write(project.join("src").join(file), source).unwrap();
        if !project.join("node_modules").exists() {
            link_provider(project);
        }
    }
    let emitted = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&producer)
        .env("CORSA_PATH", corsa)
        .args(["check", "--declaration", "--declaration-dir", "../out"])
        .arg("src/Foo.vue")
        .output()
        .unwrap();
    assert!(
        emitted.status.success(),
        "{}",
        std::str::from_utf8(&emitted.stdout).unwrap()
    );
    fs::create_dir_all(consumer.join("src/vendor")).unwrap();
    for name in ["Foo.vue.d.ts", "__vize_helpers.d.ts"] {
        fs::copy(
            dir.path().join("out").join(name),
            consumer.join("src/vendor").join(name),
        )
        .unwrap();
    }
    let checked = Command::new("node")
        .current_dir(&consumer)
        .arg(root().join("tests/node_modules/vue-tsc/bin/vue-tsc.js"))
        .args(["--noEmit", "-p", "tsconfig.json", "--pretty", "false"])
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&checked.stdout).unwrap();
    let sites: Vec<(u32, u32, u32)> = stdout
        .lines()
        .filter_map(|line| {
            let (site, rest) = line
                .strip_prefix("src/")?
                .strip_prefix(file)?
                .strip_prefix('(')?
                .split_once("): error TS")?;
            let (line, column) = site.split_once(',')?;
            let (code, _) = rest.split_once(':')?;
            Some((line.parse().ok()?, column.parse().ok()?, code.parse().ok()?))
        })
        .collect();
    assert_eq!(
        checked.status.code(),
        Some(if sites.is_empty() { 0 } else { 2 }),
        "{stdout}"
    );
    sites
}

#[test]
fn emitted_declarations_accept_fallthrough_attributes_in_vue_tsc_templates() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let vue_tsc = root().join("tests/node_modules/vue-tsc/bin/vue-tsc.js");
    assert!(vue_tsc.is_file(), "vue-tsc provider required");
    let two = FOO.replace("bar: number", "bar: number; baz: number");
    let two_without_emits = two.replace("defineEmits<{ change: [] }>();\n", "");
    let emit_only = FOO.replace("defineProps<{ bar: number }>();\n", "");
    let strict = CONFIG.replace(
        "\"include\"",
        "\"vueCompilerOptions\": { \"checkUnknownProps\": true },\n  \"include\"",
    );
    let tsx = "import Foo from \"./vendor/Foo.vue\";\n\nexport const probe = <Foo bar={1} id=\"r\" />;\nexport const owned = <Foo bar={1} class=\"r\" />;\n";
    let cases: [Case; 10] = [
        ("original", FOO, PROBE.into(), CONFIG, vec![]),
        (
            "two-required",
            &two_without_emits,
            PROBE.replace(":bar=\"1\"", ":bar=\"1\" :baz=\"2\"").into(),
            CONFIG,
            vec![],
        ),
        (
            "two-required-with-emit",
            &two,
            PROBE
                .replace(
                    ":bar=\"1\" disabled",
                    ":bar=\"1\" :baz=\"2\" title=\"x\" :custom=\"{ a: 1 }\" @click=\"() => {}\"",
                )
                .into(),
            CONFIG,
            vec![],
        ),
        (
            "emit-only",
            &emit_only,
            PROBE.replace(":bar=\"1\" ", "").into(),
            CONFIG,
            vec![],
        ),
        (
            "wrong-prop",
            FOO,
            PROBE
                .replace(":bar=\"1\" disabled", "bar=\"baz\" disabled")
                .into(),
            CONFIG,
            vec![(6, 8, 2322)],
        ),
        (
            "missing-prop",
            FOO,
            PROBE.replace(":bar=\"1\" ", "").into(),
            CONFIG,
            vec![(6, 4, 2345)],
        ),
        (
            "missing-second-prop",
            &two,
            PROBE.into(),
            CONFIG,
            vec![(6, 4, 2345)],
        ),
        (
            "wrong-listener",
            FOO,
            PROBE
                .replace(" disabled", " onChange=\"wrong\" disabled")
                .into(),
            CONFIG,
            vec![(6, 17, 2322)],
        ),
        (
            "check-unknown-props",
            FOO,
            PROBE.into(),
            &strict,
            vec![(6, 17, 2353)],
        ),
        // vue-tsc's template fallback does not reach TSX in the same program (#11629).
        (
            "tsx-in-vue-tsc-program",
            FOO,
            tsx.into(),
            MIXED_CONFIG,
            vec![(3, 23, 2322)],
        ),
    ];
    for (id, foo, probe, config, expected) in cases {
        assert_eq!(consume(&corsa, foo, &probe, config), expected, "{id}");
    }
}
