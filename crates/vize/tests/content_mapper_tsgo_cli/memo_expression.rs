//! Builtin memo dependencies remain authored reads and checked expressions.
use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};
use serde_json::json;
use std::path::PathBuf;

/// A missing host is an unexecuted cohort. `VIZE_TEST_REQUIRE_TSGO` makes that a failure.
fn memo_host() -> Option<PathBuf> {
    let required = std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_some();
    let Some(host) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        assert!(
            !required,
            "unexecuted memo cohort: VIZE_TEST_REQUIRE_TSGO is set and {TSGO_ENV} is absent"
        );
        eprintln!("unexecuted memo cohort: {TSGO_ENV} is not set");
        return None;
    };
    assert!(
        host.is_file(),
        "{TSGO_ENV} is not a file: {}",
        host.display()
    );
    eprintln!("memo cohort provider={} require={required}", host.display());
    Some(host)
}

#[test]
fn memo_dependencies_check_members_and_preserve_unused_controls() {
    let Some(host) = memo_host() else {
        return;
    };
    let fixture =
        workspace_root().join("tests/_fixtures/differential/typechecker/memo-expression-11733");
    for no_unused in [false, true] {
        for newline in ["\n", "\r\n"] {
            let project = tempfile::tempdir().unwrap();
            install_mapper_manifest(project.path());
            install_vue_package(project.path());
            std::fs::create_dir(project.path().join("src")).unwrap();
            for name in ["Memo.vue", "Missing.vue", "Unused.vue", "util.ts"] {
                let text = std::fs::read_to_string(fixture.join(format!("{name}.txt"))).unwrap();
                std::fs::write(
                    project.path().join("src").join(name),
                    text.replace('\n', newline),
                )
                .unwrap();
            }
            std::fs::write(
                project.path().join("tsconfig.json"),
                serde_json::to_vec_pretty(&json!({
                    "compilerOptions": { "strict": true, "noEmit": true, "skipLibCheck": true,
                        "target": "ES2022", "module": "ESNext", "moduleResolution": "bundler",
                        "types": [], "noUnusedLocals": no_unused },
                    "contentMappers": [{"package": "vize", "extensions": [".vue"]}],
                    "include": ["src"]
                }))
                .unwrap(),
            )
            .unwrap();
            let output = check_project(&host, project.path(), "tsconfig.json");
            eprintln!(
                "memo cell newline={} noUnusedLocals={} status={} provider={}",
                if newline == "\n" { "lf" } else { "crlf" },
                no_unused,
                output.status,
                host.display()
            );
            let mut actual = std::str::from_utf8(&output.stdout)
                .unwrap()
                .lines()
                .filter_map(|line| {
                    let (at, message) = line.split_once(": error ")?;
                    let (code, _) = message.split_once(':')?;
                    Some(format!("{} {code}", at.replace('\\', "/")))
                })
                .collect::<Vec<_>>();
            actual.sort();
            let expected = if no_unused {
                vec![
                    "src/Missing.vue(6,75) TS2339",
                    "src/Unused.vue(2,10) TS6133",
                ]
            } else {
                vec!["src/Missing.vue(6,75) TS2339"]
            };
            assert_eq!(actual, expected, "{}", output_text(&output));
            assert!(output.stderr.is_empty(), "{}", output_text(&output));
            assert!(!output.status.success());
        }
    }
}
