//! A Vue compile error in one SFC must not hide the type errors of the others.
//! The host skips semantic diagnostics for the whole program while any parse
//! diagnostic exists, so mapper diagnostics travel on the semantic pass and
//! keep the code, message and position they had (#12522).

use std::path::{Path, PathBuf};
use std::process::Command;

use vize_l0::String as CompactString;

use super::{TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text};
use crate::support::corsa_requirement;

/// A second host, such as `tsrs`, checked beside [`TSGO_ENV`] when set.
const TSRS_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_TSRS";

const OTHER: &str = "<script setup lang=\"ts\">\nconst count: number = 'y';\n</script>\n<template><div>{{ count }}</div></template>\n";
const OTHER_ERROR: &str =
    "src/B.vue(2,7): error TS2322: Type 'string' is not assignable to type 'number'.";
const SCRIPT_ERROR: &str =
    "src/A.vue(3,7): error TS2322: Type 'string' is not assignable to type 'number'.";

struct Case {
    name: &'static str,
    source: &'static str,
    /// Every diagnostic the mapper must report for `src/A.vue`, once each.
    reported: &'static [&'static str],
    /// Mapper diagnostics `vize check` does not report: a script error while
    /// the template does not parse (#8158).
    mapper_only: &'static [&'static str],
}

const CASES: &[Case] = &[
    Case {
        name: "props destructure default type",
        source: "<script setup lang=\"ts\">\nconst { size = \"big\" } = defineProps<{ size?: number }>()\nconst m: number = \"z\"\n</script>\n<template><span>{{ size }}</span></template>\n",
        reported: &[
            "src/A.vue(2,16): error vize100002: Vue compile error [DEFINE_PROPS_DESTRUCTURE_DEFAULT_TYPE]: Default value of prop \"size\" does not match declared type.",
            SCRIPT_ERROR,
        ],
        mapper_only: &[],
    },
    Case {
        name: "template parse recovery",
        source: "<script setup lang=\"ts\">\nconst m: number = \"z\"\n</script>\n<template><Loc</template>\n",
        reported: &[
            "src/A.vue(4,11): error vize100002: Template parse error: Element is missing end tag.",
            "src/A.vue(4,15): error vize100002: Template parse error: Unexpected end of input inside a tag; inferred the missing tag close so parsing can continue.",
            "src/A.vue(2,7): error TS2322: Type 'string' is not assignable to type 'number'.",
        ],
        mapper_only: &[
            "src/A.vue(2,7): error TS2322: Type 'string' is not assignable to type 'number'.",
        ],
    },
    Case {
        // Unchecked JavaScript stays unchecked: only the compile error shows.
        name: "unchecked javascript macro scope",
        source: "<script setup>\nimport { ref } from \"vue\"\nconst x = ref(1)\ndefineProps({ a: { type: Number, default: x } })\nx.value.foo()\n</script>\n<template><span>{{ a }}</span></template>\n",
        reported: &[
            "src/A.vue(4,43): error vize100002: Vue compile error [SCRIPT_SETUP_MACRO_SCOPE]: `defineProps()` in <script setup> cannot reference locally declared variable `x` because its arguments are hoisted outside setup(). Move the initialization to a normal <script> block or import it from another module.",
        ],
        mapper_only: &[],
    },
    Case {
        // A script that opts out keeps its type errors hidden.
        name: "ts-nocheck script",
        source: "<script setup lang=\"ts\">\n// @ts-nocheck\nconst { size = \"big\" } = defineProps<{ size?: number }>()\nconst m: number = \"z\"\n</script>\n<template><span>{{ size }}</span></template>\n",
        reported: &[
            "src/A.vue(3,16): error vize100002: Vue compile error [DEFINE_PROPS_DESTRUCTURE_DEFAULT_TYPE]: Default value of prop \"size\" does not match declared type.",
        ],
        mapper_only: &[],
    },
    Case {
        name: "repaired",
        source: "<script setup lang=\"ts\">\nconst { size = 1 } = defineProps<{ size?: number }>()\n</script>\n<template><span>{{ size }}</span></template>\n",
        reported: &[],
        mapper_only: &[],
    },
];

fn hosts() -> Vec<PathBuf> {
    let hosts = [TSGO_ENV, TSRS_ENV]
        .into_iter()
        .filter_map(|name| std::env::var_os(name).map(PathBuf::from))
        .collect::<Vec<_>>();
    for host in &hosts {
        assert!(
            host.is_file(),
            "Content Mapper host is not a file: {}",
            host.display()
        );
    }
    hosts
}

fn project(source: &str, newline: &str) -> tempfile::TempDir {
    let project = tempfile::tempdir().unwrap();
    std::fs::create_dir(project.path().join("src")).unwrap();
    std::fs::write(
        project.path().join("src/A.vue"),
        source.replace('\n', newline),
    )
    .unwrap();
    std::fs::write(
        project.path().join("src/B.vue"),
        OTHER.replace('\n', newline),
    )
    .unwrap();
    let options = serde_json::json!({
        "compilerOptions": {
            "strict": true, "target": "ES2022", "module": "ESNext",
            "moduleResolution": "Bundler", "skipLibCheck": true, "noEmit": true, "types": []
        },
        "include": ["src/**/*.vue"]
    });
    std::fs::write(
        project.path().join("tsconfig.base.json"),
        serde_json::to_vec_pretty(&options).unwrap(),
    )
    .unwrap();
    std::fs::write(
        project.path().join("tsconfig.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "extends": "./tsconfig.base.json",
            "contentMappers": [{ "package": "vize", "extensions": [".vue"] }]
        }))
        .unwrap(),
    )
    .unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());
    project
}

fn expected(case: &Case, skip: &[&str]) -> Vec<CompactString> {
    let mut lines = case
        .reported
        .iter()
        .filter(|line| !skip.contains(*line))
        .chain([&OTHER_ERROR])
        .map(|line| CompactString::from(*line))
        .collect::<Vec<_>>();
    lines.sort();
    lines
}

fn mapper_lines(host: &Path, root: &Path) -> Vec<CompactString> {
    let output = check_project(host, root, "tsconfig.json");
    assert!(output.stderr.is_empty(), "{}", output_text(&output));
    assert_eq!(output.status.code(), Some(2), "{}", output_text(&output));
    let mut lines = std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .map(|line| CompactString::from(line.replace('\\', "/")))
        .collect::<Vec<_>>();
    lines.sort();
    lines
}

/// `vize check` lines in the host's spelling: `file(line,col): error code: message`.
fn batch_lines(corsa: &Path, root: &Path) -> Vec<CompactString> {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .env("CORSA_PATH", corsa)
        .args([
            "check",
            "--no-config",
            "--tsconfig",
            "tsconfig.base.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    let result: serde_json::Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", output_text(&output)));
    let mut lines = Vec::new();
    for file in result["files"].as_array().unwrap() {
        for diagnostic in file["diagnostics"].as_array().unwrap() {
            let text = diagnostic.as_str().unwrap();
            let rest = text.strip_prefix("error:").unwrap();
            let (line, rest) = rest.split_once(':').unwrap();
            let (column, message) = rest.split_once(' ').unwrap();
            let message = message.split(" (source: ").next().unwrap();
            let (code, message) = match message.strip_prefix("[TS") {
                Some(coded) => {
                    let (code, message) = coded.split_once("] ").unwrap();
                    (vize_l0::cstr!("TS{code}"), message)
                }
                None => ("vize100002".into(), message),
            };
            lines.push(vize_l0::cstr!(
                "{}({line},{column}): error {code}: {message}",
                file["file"].as_str().unwrap()
            ));
        }
    }
    lines.sort();
    lines
}

#[test]
fn a_compile_error_reports_beside_every_type_error_of_the_project() {
    let hosts = hosts();
    if hosts.is_empty() {
        eprintln!("skipping #12522 Content Mapper hosts: neither {TSGO_ENV} nor {TSRS_ENV} is set");
        return;
    }
    let corsa = corsa_requirement::required_or_skip::<PathBuf>(None);
    for case in CASES {
        for newline in ["\n", "\r\n"] {
            let project = project(case.source, newline);
            for host in &hosts {
                assert_eq!(
                    mapper_lines(host, project.path()),
                    expected(case, &[]),
                    "{} {newline:?} {}",
                    case.name,
                    host.display()
                );
            }
            if let Some(corsa) = &corsa {
                assert_eq!(
                    batch_lines(corsa, project.path()),
                    expected(case, case.mapper_only),
                    "vize check {} {newline:?}",
                    case.name
                );
            }
        }
    }
}

#[test]
fn the_reported_project_shows_the_compile_error_and_the_other_files_type_error() {
    let hosts = hosts();
    if hosts.is_empty() {
        eprintln!("skipping #12522 Content Mapper hosts: neither {TSGO_ENV} nor {TSRS_ENV} is set");
        return;
    }
    let fixture = super::workspace_root()
        .join("tests/_fixtures/differential/typechecker/semantic-compile-errors-12522");
    let project = tempfile::tempdir().unwrap();
    std::fs::create_dir(project.path().join("src")).unwrap();
    for (input, output) in [
        ("A.vue.txt", "src/A.vue"),
        ("B.vue.txt", "src/B.vue"),
        ("tsconfig.json.txt", "tsconfig.json"),
    ] {
        std::fs::copy(fixture.join(input), project.path().join(output)).unwrap();
    }
    install_mapper_manifest(project.path());
    install_vue_package(project.path());
    for host in &hosts {
        assert_eq!(
            mapper_lines(host, project.path()),
            [
                "src/A.vue(2,7): error TS2322: Type 'string' is not assignable to type 'number'.",
                "src/B.vue(2,16): error vize100002: Vue compile error [DEFINE_PROPS_DESTRUCTURE_DEFAULT_TYPE]: Default value of prop \"size\" does not match declared type.",
            ],
            "{}",
            host.display()
        );
    }
}
