//! The TSRX extension domain borrows the original host predicate without renaming paths.
use super::super::{HostProfile, RefusalKind, Request, select_tsrx};
use std::{fs, path::Path};

#[test]
fn tsrx_uses_original_ignore_domains_and_retains_every_authority() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    fs::create_dir(root.join(".git")).unwrap();
    fs::create_dir(root.join("src")).unwrap();
    let config = root.join(".oxlintrc.json");
    let bytes = br#"{"ignorePatterns":["src/Config.tsrx"]}"#;
    fs::write(&config, bytes).unwrap();
    fs::write(root.join(".gitignore"), "src/Vcs.tsrx\n").unwrap();
    fs::write(root.join(".eslintignore"), "src/Custom.tsrx\n").unwrap();
    for name in [
        "Keep.tsrx",
        "Vcs.tsrx",
        "Custom.tsrx",
        "Cli.tsrx",
        "Config.tsrx",
        "Other.vue",
    ] {
        fs::write(root.join("src").join(name), format!("original:{name}\n")).unwrap();
    }
    let patterns = ["src/Cli.tsrx".into()];
    for host in [
        HostProfile::Oxlint178,
        HostProfile::Oxlint181,
        HostProfile::Oxlint186,
    ] {
        let selected = select_tsrx(
            Request {
                cwd: root,
                literal_target: "src",
                root_json: &config,
                host,
                no_ignore: false,
                cli_ignore_patterns: &patterns,
                custom_ignore_filename: ".eslintignore",
            },
            bytes,
        )
        .unwrap();
        assert_eq!(selected.originals.len(), 1);
        assert_eq!(
            selected.originals[0].cwd_relative,
            Path::new("src/Keep.tsrx")
        );
        assert_eq!(selected.originals[0].bytes, b"original:Keep.tsrx\n");
        for file in [".oxlintrc.json", ".gitignore", ".eslintignore"] {
            assert!(
                selected
                    .sources
                    .iter()
                    .any(|source| source.path == root.join(file)
                        && source.bytes == Some(fs::read(root.join(file)).unwrap()))
            );
        }
        // Overrides refer to original .tsrx paths. They are explicitly refused
        // until the owning host can provide their exact effective config.
        let overridden = br#"{"overrides":[{"files":["**/*.tsrx"],"rules":{}}]}"#;
        fs::write(&config, overridden).unwrap();
        let refusal = select_tsrx(
            Request {
                cwd: root,
                literal_target: "src",
                root_json: &config,
                host,
                no_ignore: false,
                cli_ignore_patterns: &patterns,
                custom_ignore_filename: ".eslintignore",
            },
            overridden,
        )
        .unwrap_err();
        assert_eq!(refusal.kind, RefusalKind::ConfigOverrides);
        fs::write(&config, bytes).unwrap();
    }
}
