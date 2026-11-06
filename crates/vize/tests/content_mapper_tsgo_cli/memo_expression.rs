//! Builtin memo dependencies remain authored reads and checked expressions.
use super::{TSGO_ENV, check_project, install_vue_package, output_text, workspace_root};
use serde_json::json;
use std::path::{Path, PathBuf};

const MEMO_TSGO_ENV: &str = "VIZE_TEST_MEMO_CONTENT_MAPPER_TSGO";
const RECEIPT_DIR_ENV: &str = "VIZE_TEST_MEMO_PROCESS_RECEIPT_DIR";

/// A missing host is an unexecuted cohort. `VIZE_TEST_REQUIRE_TSGO` makes that a failure.
fn memo_host() -> Option<PathBuf> {
    select_memo_host(
        std::env::var_os(MEMO_TSGO_ENV).map(PathBuf::from),
        std::env::var_os(TSGO_ENV).map(PathBuf::from),
        std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_some(),
    )
}

fn select_memo_host(
    memo: Option<PathBuf>,
    conformance: Option<PathBuf>,
    required: bool,
) -> Option<PathBuf> {
    let Some(host) = memo.or(conformance) else {
        assert!(
            !required,
            "unexecuted memo cohort: VIZE_TEST_REQUIRE_TSGO is set and both {MEMO_TSGO_ENV} and {TSGO_ENV} are absent"
        );
        eprintln!("unexecuted memo cohort: neither {MEMO_TSGO_ENV} nor {TSGO_ENV} is set");
        return None;
    };
    assert!(
        host.is_file(),
        "selected memo Content Mapper host is not a file: {}",
        host.display()
    );
    eprintln!("memo cohort provider={} require={required}", host.display());
    Some(host)
}

fn install_observed_mapper_manifest(project: &Path, receipt: &Path) {
    let package = project.join("node_modules/vize");
    std::fs::create_dir_all(&package).unwrap();
    let node = std::env::var_os("VIZE_TEST_NODE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("node"));
    let proxy = workspace_root().join("tools/support/compat/content-mapper/process-receipt.mjs");
    assert!(
        proxy.is_file(),
        "missing mapper process proxy: {}",
        proxy.display()
    );
    let manifest = json!({
        "name": "vize",
        "private": true,
        "typescript": {
            "contentMapper": {
                "exec": [node, proxy, receipt, env!("CARGO_BIN_EXE_vize"), "content-mapper"],
                "compilerOptions": ["checkJs", "noUnusedLocals", "rootDir"],
                "dynamicConfig": true,
            },
        },
    });
    std::fs::write(
        package.join("package.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}

fn assert_complete_mapper_receipt(receipt: &Path, output: &std::process::Output) {
    let bytes = std::fs::read(receipt).unwrap_or_else(|error| {
        panic!(
            "missing mapper process receipt {}: {error}\n{}",
            receipt.display(),
            output_text(output)
        )
    });
    let packet: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let context = format!(
        "{}\n{}",
        String::from_utf8_lossy(&bytes),
        output_text(output)
    );
    assert_eq!(
        packet["schema"], "vize.contentMapperProcessReceipt.v1",
        "{context}"
    );
    assert_eq!(
        packet["command"]["file"],
        env!("CARGO_BIN_EXE_vize"),
        "{context}"
    );
    assert_eq!(
        packet["command"]["args"],
        json!(["content-mapper"]),
        "{context}"
    );
    assert_eq!(packet["spawnError"], serde_json::Value::Null, "{context}");
    assert_eq!(packet["exit"]["code"], 0, "{context}");
    assert_eq!(
        packet["exit"]["signal"],
        serde_json::Value::Null,
        "{context}"
    );
    for name in ["stdin", "stdout", "stderr"] {
        let stream = &packet["streams"][name];
        assert_eq!(stream["complete"], true, "{name}: {context}");
        assert_eq!(
            stream["capturedBytes"], stream["bytes"],
            "{name}: {context}"
        );
        assert_eq!(
            stream["sha256"].as_str().map(str::len),
            Some(64),
            "{name}: {context}"
        );
    }
    assert!(
        packet["streams"]["stdin"]["bytes"].as_u64().unwrap_or(0) > 0,
        "{context}"
    );
    assert!(
        packet["streams"]["stdout"]["bytes"].as_u64().unwrap_or(0) > 0,
        "{context}"
    );
    assert_eq!(packet["streams"]["stderr"]["bytes"], 0, "{context}");
}

#[test]
fn memo_host_prefers_its_explicit_provider_over_the_conformance_provider() {
    let memo = tempfile::NamedTempFile::new().unwrap();
    let conformance = tempfile::NamedTempFile::new().unwrap();
    assert_eq!(
        select_memo_host(
            Some(memo.path().to_path_buf()),
            Some(conformance.path().to_path_buf()),
            true,
        ),
        Some(memo.path().to_path_buf())
    );
}

#[test]
fn memo_host_keeps_the_existing_conformance_provider_as_a_fallback() {
    let conformance = tempfile::NamedTempFile::new().unwrap();
    assert_eq!(
        select_memo_host(None, Some(conformance.path().to_path_buf()), true),
        Some(conformance.path().to_path_buf())
    );
}

#[test]
fn optional_memo_without_either_provider_remains_unexecuted() {
    assert_eq!(select_memo_host(None, None, false), None);
}

#[test]
#[should_panic(expected = "unexecuted memo cohort: VIZE_TEST_REQUIRE_TSGO is set")]
fn required_memo_without_either_provider_fails_closed() {
    let _ = select_memo_host(None, None, true);
}

#[test]
#[should_panic(expected = "selected memo Content Mapper host is not a file")]
fn invalid_explicit_memo_provider_does_not_fall_back_to_another_host() {
    let root = tempfile::tempdir().unwrap();
    let conformance = tempfile::NamedTempFile::new().unwrap();
    let _ = select_memo_host(
        Some(root.path().join("missing-tsgo")),
        Some(conformance.path().to_path_buf()),
        true,
    );
}

#[test]
fn memo_dependencies_check_members_and_preserve_unused_controls() {
    let Some(host) = memo_host() else {
        return;
    };
    let fixture =
        workspace_root().join("tests/_fixtures/differential/typechecker/memo-expression-11733");
    let retained_receipts = std::env::var_os(RECEIPT_DIR_ENV).map(PathBuf::from);
    if let Some(directory) = &retained_receipts {
        std::fs::create_dir_all(directory).unwrap();
    }
    let mut cell = 0;
    for no_unused in [false, true] {
        for newline in ["\n", "\r\n"] {
            let project = tempfile::tempdir().unwrap();
            let newline_name = if newline == "\n" { "lf" } else { "crlf" };
            let receipt_name = format!("cell-{cell}-{newline_name}-unused-{no_unused}.json");
            let receipt = retained_receipts.as_ref().map_or_else(
                || project.path().join(&receipt_name),
                |root| root.join(&receipt_name),
            );
            install_observed_mapper_manifest(project.path(), &receipt);
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
            assert_complete_mapper_receipt(&receipt, &output);
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
            cell += 1;
        }
    }
}
