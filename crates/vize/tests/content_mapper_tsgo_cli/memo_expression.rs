//! Builtin memo dependencies remain authored reads and checked expressions.
use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};
use serde_json::json;
use std::path::PathBuf;

#[test]
fn memo_dependencies_check_members_and_preserve_unused_controls() {
    let Some(host) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping Content Mapper conformance: {TSGO_ENV} is not set");
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
