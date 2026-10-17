//! Real Git creates the metadata; expectations never come from the selector.
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::super::{HostProfile, RefusalKind, Request, RootDecision, SourceRole, select};
use super::custody;

fn git(cwd: &Path, args: &[&str]) -> Vec<u8> {
    let mut command = Command::new("git");
    for (key, _) in std::env::vars_os() {
        if key.as_encoded_bytes().starts_with(b"GIT_") {
            command.env_remove(key);
        }
    }
    let output = command
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .args(["-c", "core.hooksPath=/dev/null"])
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap();
    eprintln!("git {args:?}: {output:?}");
    assert!(output.status.success());
    output.stdout
}

fn write(path: &Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

fn init(root: &Path) {
    fs::create_dir(root).unwrap();
    git(root, &["init", "-q"]);
    git(
        root,
        &[
            "-c",
            "user.name=Vize native fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "--allow-empty",
            "-qm",
            "fixture",
        ],
    );
}

fn git_path(root: &Path, option: &str) -> PathBuf {
    let bytes = git(root, &["rev-parse", "--path-format=absolute", option]);
    fs::canonicalize(std::str::from_utf8(&bytes).unwrap().trim()).unwrap()
}

fn selected(
    root: &Path,
    target: &str,
    host: HostProfile,
    cli: &[vize_l0::String],
) -> super::super::Selection {
    select(Request {
        cwd: root,
        literal_target: target,
        root_json: &root.join(".oxlintrc.json"),
        host,
        no_ignore: false,
        cli_ignore_patterns: cli,
        custom_ignore_filename: ".eslintignore",
    })
    .unwrap()
}

#[test]
#[cfg(unix)]
fn linked_worktree_captures_common_exclude_and_keeps_ignore_precedence() {
    for relative_gitdir in [false, true] {
        let temporary = tempfile::tempdir().unwrap();
        let base = fs::canonicalize(temporary.path()).unwrap();
        let main = base.join("main");
        init(&main);
        let root = base.join("linked");
        git(
            &main,
            &[
                "worktree",
                "add",
                "--detach",
                root.to_str().unwrap(),
                "HEAD",
            ],
        );
        let git_dir = git_path(&root, "--absolute-git-dir");
        let common = git_path(&root, "--git-common-dir");
        assert_eq!(common, main.join(".git"));
        if relative_gitdir {
            write(
                &root.join(".git"),
                b"gitdir: ../main/.git/worktrees/linked\n",
            );
            assert_eq!(git_path(&root, "--absolute-git-dir"), git_dir);
            let mut bytes = common.as_os_str().as_encoded_bytes().to_vec();
            bytes.push(b'\n');
            write(&git_dir.join("commondir"), &bytes);
            assert_eq!(git_path(&root, "--git-common-dir"), common);
        }
        write(
            &common.join("info/exclude"),
            b"src/Drop.html\nsrc/Reopen.html\nsrc/Custom.html\n",
        );
        write(
            &root.join(".gitignore"),
            b"!src/Reopen.html\nsrc/Custom.html\n",
        );
        write(&root.join("src/.eslintignore"), b"!Custom.html\n");
        write(
            &root.join(".oxlintrc.json"),
            b"{\"ignorePatterns\":[\"src/Config.html\"]}\n",
        );
        for name in [
            "Keep.html",
            "Drop.html",
            "Reopen.html",
            "Custom.html",
            "Cli.html",
            "Config.html",
        ] {
            write(
                &root.join("src").join(name),
                b"<div>native owned original</div>\n",
            );
        }
        let metadata = fs::read(root.join(".git")).unwrap();
        let commondir = fs::read(git_dir.join("commondir")).unwrap();
        let before = custody::snapshot(&base);
        for host in [
            HostProfile::Oxlint178,
            HostProfile::Oxlint181,
            HostProfile::Oxlint186,
        ] {
            let actual = selected(&root, "src", host, &["src/Cli.html".into()]);
            let after = custody::snapshot(&base);
            assert_eq!(before, after);
            assert_eq!(actual.root_decision, RootDecision::Eligible);
            assert_eq!(actual.repository, root);
            let paths = actual
                .originals
                .iter()
                .map(|file| file.cwd_relative.as_path())
                .collect::<Vec<_>>();
            assert_eq!(
                paths,
                [
                    Path::new("src/Custom.html"),
                    Path::new("src/Keep.html"),
                    Path::new("src/Reopen.html")
                ]
            );
            for original in &actual.originals {
                assert_eq!(original.bytes, b"<div>native owned original</div>\n");
            }
            let mut expected = vec![
                (
                    root.join(".git"),
                    SourceRole::GitMetadata,
                    Some(metadata.clone()),
                ),
                (
                    git_dir.join("commondir"),
                    SourceRole::GitMetadata,
                    Some(commondir.clone()),
                ),
                (
                    common.join("info/exclude"),
                    SourceRole::GitInfoExclude,
                    Some(b"src/Drop.html\nsrc/Reopen.html\nsrc/Custom.html\n".to_vec()),
                ),
                (
                    root.join(".oxlintrc.json"),
                    SourceRole::RootJson,
                    Some(b"{\"ignorePatterns\":[\"src/Config.html\"]}\n".to_vec()),
                ),
                (
                    root.join(".gitignore"),
                    SourceRole::GitIgnore,
                    Some(b"!src/Reopen.html\nsrc/Custom.html\n".to_vec()),
                ),
                (root.join(".eslintignore"), SourceRole::CustomIgnore, None),
                (root.join("src/.gitignore"), SourceRole::GitIgnore, None),
                (
                    root.join("src/.eslintignore"),
                    SourceRole::CustomIgnore,
                    Some(b"!Custom.html\n".to_vec()),
                ),
            ];
            expected.sort_unstable();
            let actual_sources = actual
                .sources
                .into_iter()
                .map(|source| (source.path, source.role, source.bytes))
                .collect::<Vec<_>>();
            assert_eq!(
                actual_sources, expected,
                "complete authority including absent sources"
            );
            let no_ignore = select(Request {
                cwd: &root,
                literal_target: "src",
                root_json: &root.join(".oxlintrc.json"),
                host,
                no_ignore: true,
                cli_ignore_patterns: &["src/Cli.html".into()],
                custom_ignore_filename: ".eslintignore",
            })
            .unwrap();
            assert_eq!(before, custody::snapshot(&base));
            assert_eq!(
                no_ignore
                    .originals
                    .iter()
                    .map(|file| file.cwd_relative.as_path())
                    .collect::<Vec<_>>(),
                [
                    Path::new("src/Cli.html"),
                    Path::new("src/Keep.html"),
                    Path::new("src/Reopen.html")
                ],
                "no-ignore disables CLI/custom matching but retains Git and root-config filtering"
            );
        }
        let excluded = selected(&root, "src/Drop.html", HostProfile::Oxlint178, &[]);
        assert_eq!(excluded.root_decision, RootDecision::VcsExcluded);
        assert!(excluded.originals.is_empty());
        // The fixed reader still accepts the deeper .gitignore directory
        // whitelist before consulting the lower-precedence common exclude.
        write(&common.join("info/exclude"), b"sub/generated/\n");
        write(&root.join("sub/.gitignore"), b"!generated/\n");
        write(
            &root.join("sub/generated/pkg/Keep.html"),
            b"<div>deeper whitelist</div>\n",
        );
        let actual = selected(&root, "sub/generated/pkg", HostProfile::Oxlint186, &[]);
        assert_eq!(actual.root_decision, RootDecision::Eligible);
        assert_eq!(actual.originals.len(), 1);
        assert_eq!(
            actual.originals[0].cwd_relative,
            Path::new("sub/generated/pkg/Keep.html")
        );
    }
}

#[test]
#[cfg(unix)]
fn submodule_style_relative_gitfile_without_commondir_uses_its_gitdir() {
    let temporary = tempfile::tempdir().unwrap();
    let base = fs::canonicalize(temporary.path()).unwrap();
    let root = base.join("submodule");
    init(&root);
    let git_dir = base.join("submodule-admin");
    fs::rename(root.join(".git"), &git_dir).unwrap();
    write(&root.join(".git"), b"gitdir: ../submodule-admin\n");
    assert_eq!(git_path(&root, "--absolute-git-dir"), git_dir);
    assert_eq!(git_path(&root, "--git-common-dir"), git_dir);
    write(&git_dir.join("info/exclude"), b"src/Drop.html\n");
    write(&root.join(".oxlintrc.json"), b"{}\n");
    write(&root.join("src/Keep.html"), b"<div>keep</div>\n");
    write(&root.join("src/Drop.html"), b"<div>drop</div>\n");
    let before = custody::snapshot(&base);
    for host in [
        HostProfile::Oxlint178,
        HostProfile::Oxlint181,
        HostProfile::Oxlint186,
    ] {
        let actual = selected(&root, "src", host, &[]);
        assert_eq!(before, custody::snapshot(&base));
        assert_eq!(actual.originals.len(), 1);
        assert_eq!(actual.originals[0].cwd_relative, Path::new("src/Keep.html"));
        assert!(
            actual
                .sources
                .iter()
                .any(|source| source.path == git_dir.join("commondir")
                    && source.role == SourceRole::GitMetadata
                    && source.bytes.is_none())
        );
        assert!(
            actual
                .sources
                .iter()
                .any(|source| source.path == git_dir.join("info/exclude")
                    && source.role == SourceRole::GitInfoExclude
                    && source.bytes.as_deref() == Some(b"src/Drop.html\n".as_slice()))
        );
    }
}

#[test]
#[cfg(unix)]
fn git_metadata_symlinks_and_malformed_commondir_fail_closed() {
    let temporary = tempfile::tempdir().unwrap();
    let base = fs::canonicalize(temporary.path()).unwrap();
    let root = base.join("repo");
    init(&root);
    let admin = base.join("admin");
    fs::rename(root.join(".git"), &admin).unwrap();
    write(&root.join(".git"), b"gitdir: ../admin\n");
    write(&root.join(".oxlintrc.json"), b"{}\n");
    write(&root.join("Keep.html"), b"<div>keep</div>\n");
    for (bytes, expected) in [
        (b"\n".as_slice(), RefusalKind::LinkedGitMetadata),
        (
            b"../admin\nextra\n".as_slice(),
            RefusalKind::LinkedGitMetadata,
        ),
        (b"\xff\n".as_slice(), RefusalKind::SourceEncoding),
    ] {
        write(&admin.join("commondir"), bytes);
        let error = select(Request {
            cwd: &root,
            literal_target: ".",
            root_json: &root.join(".oxlintrc.json"),
            host: HostProfile::Oxlint186,
            no_ignore: false,
            cli_ignore_patterns: &[],
            custom_ignore_filename: ".eslintignore",
        })
        .unwrap_err();
        assert_eq!(error.kind, expected);
        assert_eq!(error.path, admin.join("commondir"));
    }
    fs::remove_file(admin.join("commondir")).unwrap();
    std::os::unix::fs::symlink(&admin, base.join("admin-link")).unwrap();
    write(&root.join(".git"), b"gitdir: ../admin-link\n");
    let error = select(Request {
        cwd: &root,
        literal_target: ".",
        root_json: &root.join(".oxlintrc.json"),
        host: HostProfile::Oxlint186,
        no_ignore: false,
        cli_ignore_patterns: &[],
        custom_ignore_filename: ".eslintignore",
    })
    .unwrap_err();
    assert_eq!(error.kind, RefusalKind::Symlink);
    assert_eq!(error.path, base.join("admin-link"));
}
