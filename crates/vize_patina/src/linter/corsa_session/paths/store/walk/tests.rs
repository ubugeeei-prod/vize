use super::{String, glob_match, push_program_prefix, slash_path, walk_glob};
use std::ffi::OsStr;
use std::path::{Path, Prefix};

fn reference_glob_match(pattern: &str, relative: &str) -> bool {
    let pattern = pattern.trim_start_matches("./");
    let pattern_parts: Vec<&str> = pattern.split('/').filter(|part| !part.is_empty()).collect();
    let name_parts: Vec<&str> = relative
        .split('/')
        .filter(|part| !part.is_empty())
        .collect();
    reference_glob_parts(&pattern_parts, &name_parts)
}

fn reference_glob_parts(pattern: &[&str], name: &[&str]) -> bool {
    let Some((head, pattern_rest)) = pattern.split_first() else {
        return name.is_empty();
    };
    if *head == "**" {
        if reference_glob_parts(pattern_rest, name) {
            return true;
        }
        return match name.split_first() {
            Some((_, name_rest)) => reference_glob_parts(pattern, name_rest),
            None => false,
        };
    }
    let Some((name_head, name_rest)) = name.split_first() else {
        return false;
    };
    super::segment_match(head, name_head) && reference_glob_parts(pattern_rest, name_rest)
}

#[test]
fn glob_match_agrees_with_the_allocating_segment_lists() {
    let patterns = [
        "node_modules",
        "src/**/*.ts",
        "**/*.d.ts",
        "tests/**",
        "./src/**",
        "*",
        "a/**/b",
        "**",
        "src/*",
        "*.d.ts",
        "a//b",
        "././src/**/*.ts",
    ];
    let names = [
        "node_modules",
        "src/node_modules/pkg/index.ts",
        "src/a.ts",
        "src/a.d.ts",
        "src/nested/env.d.ts",
        "a/x/y/b",
        "a/b",
        "a//b",
        "src/file.ts",
        "src/dir/file.ts",
        "tests/a.ts",
        "other.ts",
        "",
        "src",
        "src/a/b",
    ];
    for pattern in patterns {
        for name in names {
            assert_eq!(
                glob_match(pattern, name),
                reference_glob_match(pattern, name),
                "{pattern} vs {name}"
            );
        }
    }
}

#[test]
fn patterns_with_several_globstars_agree_with_the_reference() {
    let patterns = [
        "**/**",
        "**/**/*.ts",
        "**/a/**/b",
        "a/**/**/b",
        "**/x/**",
        "./**/a/**/a/**/b",
        "**/a/**/a/**/a",
    ];
    let names = [
        "",
        "a",
        "b",
        "a/b",
        "a/a/b",
        "a/x/a/y/b",
        "x",
        "q/x/r",
        "a/a/a",
        "a/a",
        "src/dir/file.ts",
        "a/a/a/a/c",
    ];
    for pattern in patterns {
        for name in names {
            assert_eq!(
                glob_match(pattern, name),
                reference_glob_match(pattern, name),
                "{pattern} vs {name}"
            );
        }
    }
}

/// Every split of the name between the globstars fails; each remainder pair
/// is tried once, so this returns at once instead of trying every split.
#[test]
fn a_long_name_that_never_matches_is_rejected_without_retrying_splits() {
    let pattern = ["**/a/".repeat(24).as_str(), "b"].concat();
    let name = ["a/".repeat(96).as_str(), "c"].concat();
    assert!(!glob_match(&pattern, &name));
}

#[test]
fn repeated_globstars_walk_each_directory_once() {
    let root = tempfile::tempdir().unwrap();
    let nested = root.path().join("a/a/a/a");
    std::fs::create_dir_all(&nested).unwrap();
    let file = nested.join("x.ts");
    std::fs::write(&file, "").unwrap();

    let mut out = Vec::new();
    walk_glob(root.path(), &["**", "**", "**", "*.ts"], &mut out);
    assert_eq!(out, vec![file.clone()]);

    let mut out = Vec::new();
    walk_glob(root.path(), &["**", "a", "**", "*.ts"], &mut out);
    assert_eq!(out, vec![file]);
}

fn rendered(kind: Prefix<'_>, raw: &str) -> String {
    let mut rendered = String::new("");
    push_program_prefix(&mut rendered, kind, OsStr::new(raw));
    rendered
}

#[test]
fn verbatim_windows_prefixes_render_as_their_plain_program_roots() {
    // `\\?\C:` became `//?/C:` in Corsa, a different root from the
    // `C:/...` spelling the SFC import graph uses.
    assert_eq!(rendered(Prefix::VerbatimDisk(b'C'), "\\\\?\\C:"), "C:");
    assert_eq!(rendered(Prefix::Disk(b'd'), "d:"), "d:");
    assert_eq!(
        rendered(
            Prefix::VerbatimUNC(OsStr::new("server"), OsStr::new("share")),
            "\\\\?\\UNC\\server\\share"
        ),
        "//server/share"
    );
    assert_eq!(
        rendered(
            Prefix::UNC(OsStr::new("server"), OsStr::new("share")),
            "\\\\server\\share"
        ),
        "//server/share"
    );
    assert_eq!(
        rendered(Prefix::DeviceNS(OsStr::new("pipe")), "\\\\.\\pipe"),
        "\\\\.\\pipe"
    );
}

#[test]
fn slash_paths_never_carry_a_verbatim_prefix() {
    if cfg!(windows) {
        assert_eq!(
            slash_path(Path::new("\\\\?\\C:\\work\\app\\src\\env.d.ts")),
            "C:/work/app/src/env.d.ts"
        );
        assert_eq!(
            slash_path(Path::new("\\\\?\\UNC\\server\\share\\app\\env.d.ts")),
            "//server/share/app/env.d.ts"
        );
        assert_eq!(
            slash_path(Path::new("C:\\work\\app\\src\\env.d.ts")),
            "C:/work/app/src/env.d.ts"
        );
    } else {
        assert_eq!(
            slash_path(Path::new("/work/app/src/env.d.ts")),
            "/work/app/src/env.d.ts"
        );
    }
}
