use super::super::pr_github as github;
use super::metadata;
use super::tests::Repo;

#[test]
fn frozen_jsr_channel_and_every_publication_authority_fail_closed() {
    let repo = Repo::new();
    repo.commit(&[
        ("Cargo.toml", "[workspace]\nmembers = [\"pkg\"]\nresolver = \"2\"\n[workspace.package]\nversion = \"1.2.3\"\nedition = \"2024\"\n"),
        ("pkg/Cargo.toml", "[package]\nname = \"vize_law\"\nversion.workspace = true\nedition.workspace = true\n"),
        ("pkg/src/lib.rs", "pub fn law() {}\n"),
        ("npm/law/package.json", "{\"name\":\"vize-law\",\"version\":\"1.2.3\"}\n"),
        ("pnpm-workspace.yaml", "catalogs:\n  native-binaries:\n    \"@vizejs/native-test\": \"1.2.3\"\n"),
        ("tools/moon/cmd/publish_crates/main.mbt", "let published_crates = [\"vize_law\"]\n"),
        (".github/workflows/release.yml", "publish npm/law\n"),
        ("tools/commands/ci/github/release-platforms.rs", "native-test\n"),
        ("tools/moon/cmd/publish_npm_package_dirs/main.mbt", "all native directories\n"),
        ("tools/moon/cmd/publish_npm_package/main.mbt", "publish exact package version\n"),
    ]);
    github::output("cargo", &["generate-lockfile", "--offline"], &repo.work).unwrap();
    let head = repo.commit(&[]);
    let original = metadata::catalog(&head, "1.2.3", &repo.work).unwrap();
    assert!(
        original.get("jsr").is_none(),
        "legacy source keeps its catalog shape"
    );
    github::git(&["reset", "--hard", &head], &repo.work).unwrap();
    let partial = repo.commit(&[(
        ".github/workflows/release.yml",
        "uses: ./.github/workflows/release-jsr.yml\n",
    )]);
    assert!(metadata::catalog(&partial, "1.2.3", &repo.work).is_err());
    let jsr = [
        (
            ".github/workflows/release-jsr.yml",
            "publish and verify JSR\n",
        ),
        (
            "tools/support/release/jsr/prepare.mjs",
            "pin exact npm dependencies\n",
        ),
        (
            "tools/support/release/jsr/consumer.mjs",
            "install public JSR package\n",
        ),
        (
            "jsr/vize/jsr.json",
            "{\"name\":\"@vizejs/vize\",\"exports\":{\".\":\"./mod.ts\",\"./config\":\"./config.ts\",\"./native\":\"./native.ts\",\"./vite\":\"./vite.ts\"}}\n",
        ),
        ("jsr/vize/README.md", "supported Node imports\n"),
        ("LICENSE", "MIT\n"),
        (
            "jsr/vize/channel.json",
            "{\"schema\":\"vize-jsr-channel-v1\",\"enabled\":false}\n",
        ),
        (
            "tools/support/release/jsr/channel.mjs",
            "read exact frozen channel policy\n",
        ),
    ];
    let jsr_head = repo.commit(&jsr);
    let with_jsr = metadata::catalog(&jsr_head, "1.2.3", &repo.work).unwrap();
    assert_eq!(with_jsr["jsr"]["enabled"], false);
    assert_eq!(with_jsr["jsr"]["name"], "@vizejs/vize");
    assert_eq!(with_jsr["jsr"]["version"], "1.2.3");
    for (path, _) in jsr {
        github::git(&["reset", "--hard", &jsr_head], &repo.work).unwrap();
        let changed = repo.commit(&[(path, "changed publishing authority\n")]);
        let observed = metadata::catalog(&changed, "1.2.3", &repo.work);
        if matches!(path, "jsr/vize/channel.json" | "jsr/vize/jsr.json") {
            assert!(observed.is_err(), "malformed required authority: {path}");
        } else {
            let observed = observed.unwrap();
            assert_ne!(with_jsr, observed, "{path}");
            assert_eq!(with_jsr["npm"], observed["npm"], "{path}");
        }
        github::git(&["reset", "--hard", &jsr_head], &repo.work).unwrap();
        github::git(&["rm", path], &repo.work).unwrap();
        let missing = repo.commit(&[]);
        assert!(
            metadata::catalog(&missing, "1.2.3", &repo.work).is_err(),
            "{path}"
        );
    }
    github::git(&["reset", "--hard", &jsr_head], &repo.work).unwrap();
    let enabled = repo.commit(&[(
        "jsr/vize/channel.json",
        "{\"schema\":\"vize-jsr-channel-v1\",\"enabled\":true}\n",
    )]);
    let required = metadata::catalog(&enabled, "1.2.3", &repo.work).unwrap();
    assert_eq!(required["jsr"]["enabled"], true);
    assert_ne!(
        with_jsr, required,
        "required channel cannot disappear from H's catalog"
    );
    for field in [
        "npm",
        "crates",
        "editors",
        "nativeCatalog",
        "cratePublisher",
    ] {
        assert_eq!(
            with_jsr[field], required[field],
            "unchanged original {field}"
        );
    }
    for invalid in [
        "{\"schema\":\"vize-jsr-channel-v1\",\"enabled\":\"true\"}",
        "{\"schema\":\"unknown\",\"enabled\":false}",
        "{\"schema\":\"vize-jsr-channel-v1\",\"enabled\":false,\"skip\":true}",
    ] {
        github::git(&["reset", "--hard", &jsr_head], &repo.work).unwrap();
        let malformed = repo.commit(&[("jsr/vize/channel.json", invalid)]);
        assert!(metadata::catalog(&malformed, "1.2.3", &repo.work).is_err());
    }
}
