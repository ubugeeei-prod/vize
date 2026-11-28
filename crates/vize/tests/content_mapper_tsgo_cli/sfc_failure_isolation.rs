//! A Vue SFC that fails to parse, compile or map must not hide an unrelated
//! diagnostic of the same project. A type error in a plain TypeScript module
//! and one in another SFC keep their exact packets beside each failure, with
//! LF and CRLF sources, and a repair leaves only the unrelated errors.

use std::path::{Path, PathBuf};

use vize_l0::String as CompactString;

use super::{TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text};

const MODULE: &str = "export const total: number = 'x';\n";
const MODULE_ERROR: &str =
    "src/module.ts(1,14): error TS2322: Type 'string' is not assignable to type 'number'.";
const OTHER: &str = "<script setup lang=\"ts\">\nconst count: number = 'y';\n</script>\n<template><div>{{ count }}</div></template>\n";
const OTHER_ERROR: &str =
    "src/Other.vue(2,7): error TS2322: Type 'string' is not assignable to type 'number'.";

struct Case {
    name: &'static str,
    source: &'static str,
    /// Every diagnostic reported for `src/Broken.vue`, once each.
    reported: &'static [&'static str],
}

const CASES: &[Case] = &[
    Case {
        name: "unterminated script block",
        source: "<script setup lang=\"ts\">\nconst a: number = 1\n<template><div>{{ a }}</div></template>\n",
        reported: &[
            "src/Broken.vue(1,1): error vize100001: Malformed <script> block: the closing tag is missing.",
        ],
    },
    Case {
        name: "unterminated template block",
        source: "<script setup lang=\"ts\">\nconst a = 1\n</script>\n<template><div>{{ a }}</div>\n",
        reported: &[
            "src/Broken.vue(4,1): error vize100001: Malformed <template> block: the closing tag is missing.",
        ],
    },
    Case {
        name: "duplicate script setup blocks",
        source: "<script setup lang=\"ts\">\nconst a = 1\n</script>\n<script setup lang=\"ts\">\nconst b = 2\n</script>\n<template><div>{{ a }}</div></template>\n",
        reported: &[
            "src/Broken.vue(4,25): error vize100001: SFC can only contain one <script setup> block",
        ],
    },
    Case {
        name: "refused pug include",
        source: "<script setup lang=\"ts\">\nconst a = 1\n</script>\n<template lang=\"pug\">\ninclude other.pug\n</template>\n",
        reported: &[
            "src/Broken.vue(5,1): error vize100001: pug `include` is not supported in Vue templates: it links other pug files at build time",
        ],
    },
    Case {
        name: "repaired",
        source: "<script setup lang=\"ts\">\nconst a = 1\n</script>\n<template><div>{{ a }}</div></template>\n",
        reported: &[],
    },
];

/// A TypeScript syntax error stops the semantic pass of the whole program,
/// in a plain module as in an SFC. The SFC must hide exactly what the same
/// error in a `.ts` file hides, and no more.
const SYNTAX_ERROR_SCRIPT: &str = "const a = ;\n";

fn project(broken: &str, source: &str, newline: &str) -> tempfile::TempDir {
    let project = tempfile::tempdir().unwrap();
    let src = project.path().join("src");
    std::fs::create_dir(&src).unwrap();
    for (name, text) in [
        (broken, source),
        ("Other.vue", OTHER),
        ("module.ts", MODULE),
    ] {
        std::fs::write(src.join(name), text.replace('\n', newline)).unwrap();
    }
    std::fs::write(
        project.path().join("tsconfig.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "compilerOptions": {
                "strict": true, "target": "ES2022", "module": "ESNext",
                "moduleResolution": "Bundler", "skipLibCheck": true, "noEmit": true, "types": []
            },
            "include": ["src/**/*.ts", "src/**/*.vue"],
            "contentMappers": [{ "package": "vize", "extensions": [".vue"] }]
        }))
        .unwrap(),
    )
    .unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());
    project
}

fn sorted(lines: impl IntoIterator<Item = CompactString>) -> Vec<CompactString> {
    let mut lines = lines.into_iter().collect::<Vec<_>>();
    lines.sort();
    lines
}

fn host() -> Option<PathBuf> {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper failure isolation: {TSGO_ENV} is not set");
        return None;
    };
    assert!(
        tsgo.is_file(),
        "{TSGO_ENV} is not a file: {}",
        tsgo.display()
    );
    Some(tsgo)
}

/// Sorted diagnostic lines of one checked project, with its exit status.
fn check(tsgo: &Path, broken: &str, source: &str, newline: &str) -> Vec<CompactString> {
    let project = project(broken, source, newline);
    let output = check_project(tsgo, project.path(), "tsconfig.json");
    let label = vize_l0::cstr!("{broken} {newline:?}\n{}", output_text(&output));
    assert!(output.stderr.is_empty(), "{label}");
    assert_eq!(output.status.code(), Some(2), "{label}");
    sorted(
        std::str::from_utf8(&output.stdout)
            .unwrap()
            .lines()
            .map(|line| CompactString::from(line.replace('\\', "/"))),
    )
}

#[test]
fn a_failing_sfc_never_hides_an_unrelated_project_diagnostic() {
    let Some(tsgo) = host() else { return };
    for case in CASES {
        let expected = sorted(
            case.reported
                .iter()
                .chain([&MODULE_ERROR, &OTHER_ERROR])
                .map(|line| CompactString::from(*line)),
        );
        for newline in ["\n", "\r\n"] {
            assert_eq!(
                check(&tsgo, "Broken.vue", case.source, newline),
                expected,
                "{} {newline:?}",
                case.name
            );
        }
    }
}

#[test]
fn an_sfc_script_syntax_error_hides_only_what_a_module_syntax_error_hides() {
    let Some(tsgo) = host() else { return };
    let sfc = vize_l0::cstr!(
        "<script setup lang=\"ts\">\n{SYNTAX_ERROR_SCRIPT}</script>\n<template><div>{{{{ a }}}}</div></template>\n"
    );
    let module = vize_l0::cstr!("{SYNTAX_ERROR_SCRIPT}export {{}};\n");
    for newline in ["\n", "\r\n"] {
        assert_eq!(
            check(&tsgo, "broken.ts", &module, newline),
            ["src/broken.ts(1,11): error TS1109: Expression expected."],
            "plain module control {newline:?}"
        );
        assert_eq!(
            check(&tsgo, "Broken.vue", &sfc, newline),
            ["src/Broken.vue(2,11): error TS1109: Expression expected."],
            "{newline:?}"
        );
    }
}
