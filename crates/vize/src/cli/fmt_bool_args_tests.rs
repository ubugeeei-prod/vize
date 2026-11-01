#![expect(clippy::disallowed_types, reason = "actual CLI parser fixture strings")]
#![expect(
    clippy::disallowed_macros,
    reason = "actual CLI parser fixture strings"
)]

use super::{normalize, optional_bool};
use crate::cli::{Cli, Commands};
use clap::{CommandFactory, Parser, error::ErrorKind};
use std::ffi::OsString;

fn normalized(args: &[&str]) -> Vec<OsString> {
    normalize(args.iter().map(OsString::from))
}

#[test]
fn original_clap_grammar_explains_false_becoming_a_pattern() {
    let old = Cli::try_parse_from(["vize", "fmt", "a.vue", "--semi", "false"]).unwrap();
    let Some(Commands::Fmt(old)) = old.command else {
        panic!("formatter")
    };
    assert_eq!(old.semi, Some(true));
    assert_eq!(old.patterns, ["a.vue", "false"]);
    let new =
        Cli::try_parse_from(normalized(&["vize", "fmt", "a.vue", "--semi", "false"])).unwrap();
    let Some(Commands::Fmt(new)) = new.command else {
        panic!("formatter")
    };
    assert_eq!(new.semi, Some(false));
    assert_eq!(new.patterns, ["a.vue"]);
}

#[test]
fn every_declared_optional_bool_accepts_both_literals_in_both_formatter_names() {
    let mut cli = Cli::command();
    cli.build();
    let command = cli.find_subcommand("fmt").unwrap();
    let names: Vec<_> = command
        .get_arguments()
        .filter(|arg| optional_bool(arg))
        .map(|arg| arg.get_long().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "single-quote",
            "jsx-single-quote",
            "bracket-spacing",
            "bracket-same-line",
            "use-tabs",
            "semi",
            "sort-attributes",
            "single-attribute-per-line",
            "vue-indent-script-and-style",
            "normalize-directive-shorthands",
            "sort-blocks",
            "self-close-void-elements",
            "normalize-css-quotes",
            "normalize-inline-styles",
            "normalize-v-for-parens",
            "reindent-preprocessor-styles"
        ]
    );
    for name in names {
        for alias in ["fmt", "glyph"] {
            for value in ["true", "false"] {
                let flag = format!("--{name}");
                let joined = format!("{flag}={value}");
                let actual = normalized(&["vize", alias, "a.vue", &flag, value]);
                assert_eq!(
                    actual,
                    ["vize", alias, "a.vue", &joined].map(OsString::from)
                );
                let matches = command
                    .clone()
                    .try_get_matches_from(actual.iter().skip(1))
                    .unwrap();
                let id = command
                    .get_arguments()
                    .find(|arg| arg.get_long() == Some(name))
                    .unwrap()
                    .get_id();
                assert_eq!(
                    matches.get_one::<bool>(id.as_str()),
                    Some(&(value == "true"))
                );
                assert_eq!(
                    matches
                        .get_many::<String>("patterns")
                        .unwrap()
                        .map(String::as_str)
                        .collect::<Vec<_>>(),
                    ["a.vue"]
                );
            }
        }
    }
}

#[test]
fn bare_flags_files_other_commands_and_terminators_stay_literal() {
    for args in [
        vec!["vize", "fmt", "--semi", "a.vue"],
        vec!["vize", "fmt", "--semi", "--", "false"],
        vec!["vize", "fmt", "--", "--semi", "false"],
        vec!["vize", "lint", "--semi", "false"],
        vec!["vize", "--", "fmt", "--semi", "false"],
        vec!["vize", "fmt", "--unknown", "--semi", "false"],
        vec!["vize", "fmt", "--semi=invalid"],
    ] {
        assert_eq!(
            normalized(&args),
            args.iter().map(OsString::from).collect::<Vec<_>>()
        );
    }
    let cli = Cli::try_parse_from(normalized(&["vize", "fmt", "--semi", "a.vue"])).unwrap();
    let Some(Commands::Fmt(args)) = cli.command else {
        panic!("formatter")
    };
    assert_eq!(args.semi, Some(true));
    assert_eq!(args.patterns, ["a.vue"]);
    let cli = Cli::try_parse_from(normalized(&["vize", "fmt", "--", "false"])).unwrap();
    let Some(Commands::Fmt(args)) = cli.command else {
        panic!("formatter")
    };
    assert_eq!(args.semi, None);
    assert_eq!(args.patterns, ["false"]);
}

#[test]
fn required_values_are_opaque_including_short_clusters_and_flag_like_paths() {
    for prefix in [
        vec!["--config", "--semi"],
        vec!["-c", "--semi"],
        vec!["-wc", "--semi"],
        vec!["--config", "false"],
        vec!["--config=--semi"],
        vec!["-cfalse"],
    ] {
        let mut args = vec!["vize", "fmt"];
        args.extend(prefix.iter().copied());
        args.extend(["false", "--bracket-spacing", "false", "a.vue"]);
        let mut want = vec!["vize", "fmt"];
        want.extend(prefix.iter().copied());
        want.extend(["false", "--bracket-spacing=false", "a.vue"]);
        assert_eq!(
            normalized(&args),
            want.iter().map(OsString::from).collect::<Vec<_>>()
        );
    }
    assert_eq!(
        normalized(&["vize", "fmt", "--config", "--", "--semi", "false"]),
        ["vize", "fmt", "--config", "--", "--semi", "false"].map(OsString::from)
    );
    let mut cli = Cli::command();
    cli.build();
    for arg in cli
        .find_subcommand("fmt")
        .unwrap()
        .get_arguments()
        .filter(|arg| super::required_value(arg))
    {
        let Some(name) = arg.get_long() else { continue };
        let flag = format!("--{name}");
        assert_eq!(
            normalized(&["vize", "fmt", &flag, "--semi", "false"]),
            ["vize", "fmt", &flag, "--semi", "false"].map(OsString::from)
        );
    }
}

#[test]
fn equals_spaced_mixtures_and_duplicate_conflicts_keep_clap_precedence() {
    let args = normalized(&[
        "vize",
        "fmt",
        "a.vue",
        "--semi=false",
        "--bracket-spacing",
        "false",
        "--sort-attributes",
        "true",
    ]);
    let Some(Commands::Fmt(args)) = Cli::try_parse_from(args).unwrap().command else {
        panic!("formatter")
    };
    assert_eq!(
        (args.semi, args.bracket_spacing, args.sort_attributes),
        (Some(false), Some(false), Some(true))
    );
    for (spaced, equals) in [
        (
            vec!["--semi", "false", "--no-semi"],
            vec!["--semi=false", "--no-semi"],
        ),
        (
            vec!["--no-semi", "--semi", "false"],
            vec!["--no-semi", "--semi=false"],
        ),
        (
            vec!["--semi=true", "--semi", "false"],
            vec!["--semi=true", "--semi=false"],
        ),
        (
            vec!["--semi", "false", "--semi=true"],
            vec!["--semi=false", "--semi=true"],
        ),
    ] {
        let spaced: Vec<_> = ["vize", "fmt", "a.vue"].into_iter().chain(spaced).collect();
        let equals: Vec<_> = ["vize", "fmt", "a.vue"].into_iter().chain(equals).collect();
        let before = Cli::try_parse_from(equals).err().unwrap().kind();
        assert_eq!(before, ErrorKind::ArgumentConflict);
        assert_eq!(
            Cli::try_parse_from(normalized(&spaced))
                .err()
                .unwrap()
                .kind(),
            before
        );
    }
}

#[cfg(unix)]
#[test]
fn non_utf8_arguments_are_not_reinterpreted() {
    use std::os::unix::ffi::OsStringExt;
    let args = vec![
        OsString::from("vize"),
        OsString::from("fmt"),
        OsString::from_vec(vec![0xff]),
        OsString::from("--semi"),
        OsString::from("false"),
    ];
    assert_eq!(normalize(args.clone()), args);
}
