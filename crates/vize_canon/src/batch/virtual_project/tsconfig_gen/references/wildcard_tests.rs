//! Expected selections come from `tsc` 6.0.3 `getPatternFromSpec`, checked
//! against tsgo 7.0.2 `--listFilesOnly`. They agree on every case except
//! `""`, where tsgo is the program builder Vize runs.

use std::path::{Path, PathBuf};

use super::wildcard::{TsconfigWildcard, TsconfigWildcardUsage};

const ROOT: &str = "/vize-wildcard-fixture/p";

const FILES: [&str; 22] = [
    "lib.ts",
    "node_modules/n.ts",
    "src/!x.ts",
    "src/.dot/q.ts",
    "src/.hidden.ts",
    "src/.ts",
    "src/[id].vue",
    "src/[id]/p.ts",
    "src/a.min.js/x.ts",
    "src/a.ts",
    "src/d.ts",
    "src/i.ts",
    "src/i.vue",
    "src/node_modules/m.ts",
    "src/pages/p.ts",
    "src/sub/z.ts",
    "src/w/pages/p2.ts",
    "src/x.min.js",
    "src/y.js",
    "src/z.MIN.JS",
    "src/{a,b}.ts",
    "src/Node_Modules/u.ts",
];

fn select(specs: &[&str], usage: TsconfigWildcardUsage, case_sensitive: bool) -> Vec<&'static str> {
    let specs = specs
        .iter()
        .filter_map(|spec| TsconfigWildcard::new(Path::new(ROOT), spec, usage))
        .collect::<Vec<_>>();
    FILES
        .into_iter()
        .filter(|file| {
            let path = Path::new(ROOT).join(file);
            specs.iter().any(|spec| spec.matches(&path, case_sensitive))
        })
        .collect()
}

fn included(specs: &[&str]) -> Vec<&'static str> {
    select(specs, TsconfigWildcardUsage::Include, true)
}

fn excluded(specs: &[&str]) -> Vec<&'static str> {
    select(specs, TsconfigWildcardUsage::Exclude, true)
}

#[test]
fn only_star_and_question_are_wildcards() {
    assert_eq!(included(&["src/[id].vue"]), ["src/[id].vue"]);
    assert_eq!(included(&["src/{a,b}.ts"]), ["src/{a,b}.ts"]);
    assert_eq!(included(&["src/!x.ts"]), ["src/!x.ts"]);
    assert_eq!(included(&["src/[id]"]), ["src/[id]/p.ts"]);
    assert_eq!(excluded(&["src/[id]"]), ["src/[id]/p.ts"]);
    assert_eq!(excluded(&["src/[id].vue"]), ["src/[id].vue"]);
}

#[test]
fn include_wildcards_skip_dots_package_folders_and_min_js() {
    assert_eq!(
        included(&["src/*"]),
        [
            "src/!x.ts",
            "src/[id].vue",
            "src/a.ts",
            "src/d.ts",
            "src/i.ts",
            "src/i.vue",
            "src/y.js",
            "src/z.MIN.JS",
            "src/{a,b}.ts",
        ]
    );
    assert_eq!(
        included(&["src/?.ts"]),
        ["src/a.ts", "src/d.ts", "src/i.ts"]
    );
    assert_eq!(
        included(&["src/*.ts"]),
        [
            "src/!x.ts",
            "src/.ts",
            "src/a.ts",
            "src/d.ts",
            "src/i.ts",
            "src/{a,b}.ts"
        ],
        "an empty leading `*` still lets `.ts` match"
    );
    assert_eq!(
        included(&["src/.*"]),
        ["src/.hidden.ts", "src/.ts"],
        "a literal leading dot selects dot files"
    );
    assert_eq!(
        included(&["src/*/m.ts", "src/*/u.ts"]),
        ["src/Node_Modules/u.ts"]
    );
    assert_eq!(included(&["*/node_modules/*"]), ["src/node_modules/m.ts"]);
    assert_eq!(included(&["src/*/x.ts"]), ["src/a.min.js/x.ts"]);
    assert_eq!(included(&["src/*.min.js"]), ["src/x.min.js"]);
    assert_eq!(
        included(&["src/**/*.ts"]),
        [
            "src/!x.ts",
            "src/.ts",
            "src/[id]/p.ts",
            "src/a.min.js/x.ts",
            "src/a.ts",
            "src/d.ts",
            "src/i.ts",
            "src/pages/p.ts",
            "src/sub/z.ts",
            "src/w/pages/p2.ts",
            "src/{a,b}.ts",
            "src/Node_Modules/u.ts",
        ]
    );
}

#[test]
fn star_matches_an_empty_leading_name() {
    let spec =
        TsconfigWildcard::new(Path::new(ROOT), "src/*.ts", TsconfigWildcardUsage::Include).unwrap();
    assert!(spec.matches(&Path::new(ROOT).join("src/.ts"), true));
    assert!(!spec.matches(&Path::new(ROOT).join("src/.hidden.ts"), true));
}

#[test]
fn case_insensitive_hosts_fold_names_and_package_folders() {
    let insensitive = |specs: &[&str]| select(specs, TsconfigWildcardUsage::Include, false);
    assert_eq!(
        insensitive(&["SRC/?.TS"]),
        ["src/a.ts", "src/d.ts", "src/i.ts"]
    );
    assert_eq!(insensitive(&["src/*/u.ts"]), Vec::<&str>::new());
    assert_eq!(
        insensitive(&["src/*.js"]),
        ["src/y.js"],
        "`.MIN.JS` is guarded too"
    );
    assert_eq!(included(&["src/*.JS"]), ["src/z.MIN.JS"]);
}

#[test]
fn implicit_globs_follow_the_last_segment() {
    assert_eq!(included(&["src/*/pages"]), ["src/w/pages/p2.ts"]);
    assert_eq!(included(&["src/sub/../d.ts"]), ["src/d.ts"]);
    assert_eq!(included(&["src/*/"]), included(&["src/*"]));
    let dotted = Path::new("/vize-wildcard-fixture/my.app");
    for spec in [".", "./", ""] {
        let spec = TsconfigWildcard::new(dotted, spec, TsconfigWildcardUsage::Include).unwrap();
        assert!(!spec.matches(&dotted.join("a.ts"), true));
    }
    assert_eq!(included(&["."]).len(), 16);
    assert_eq!(
        included(&[""]),
        included(&["."]),
        "tsgo reads `\"\"` as `.`"
    );
}

#[test]
fn excludes_cover_dots_package_folders_and_descendants() {
    assert_eq!(excluded(&["src/sub"]), ["src/sub/z.ts"]);
    assert_eq!(
        excluded(&["**/node_modules"]),
        ["node_modules/n.ts", "src/node_modules/m.ts"]
    );
    assert_eq!(excluded(&["src/.dot"]), ["src/.dot/q.ts"]);
    assert_eq!(excluded(&["src/*"]).len(), 20);
    assert_eq!(excluded(&["src/**"]), excluded(&["src/*"]));
    assert_eq!(excluded(&["*"]).len(), FILES.len());
}

#[test]
fn rejected_specs_match_nothing() {
    for (spec, usage) in [
        ("src/**", TsconfigWildcardUsage::Include),
        ("src/**/", TsconfigWildcardUsage::Include),
        ("**", TsconfigWildcardUsage::Include),
        ("src/**/../lib.ts", TsconfigWildcardUsage::Include),
        ("src/**/../lib.ts", TsconfigWildcardUsage::Exclude),
    ] {
        assert!(
            TsconfigWildcard::new(Path::new(ROOT), spec, usage).is_none(),
            "{spec}"
        );
    }
}

#[test]
fn bases_stop_at_the_first_wildcard() {
    let base = |spec: &str| {
        TsconfigWildcard::new(Path::new(ROOT), spec, TsconfigWildcardUsage::Include)
            .map(|spec| spec.base().to_path_buf())
    };
    let root = PathBuf::from(ROOT);
    assert_eq!(base("src/pages/**/*.vue"), Some(root.join("src/pages")));
    assert_eq!(base("src/pages/[id].vue"), Some(root.join("src/pages")));
    assert_eq!(base("src"), Some(root.join("src")));
    assert_eq!(
        base("../shared/*.ts"),
        Some(PathBuf::from("/vize-wildcard-fixture/shared"))
    );
    assert_eq!(base(r"src\*.ts"), Some(root.join("src")));
    let hidden = TsconfigWildcard::new(
        Path::new(ROOT),
        "../.vitepress/theme/**/*",
        TsconfigWildcardUsage::Include,
    )
    .unwrap();
    assert_eq!(hidden.literal_directories(), Path::new(".vitepress/theme"));
}
