//! Opt-in causal collector; success is not native product qualification.
#![cfg(test)]

mod counterfactual;
mod public_cli;

use serde_json::{Value, json};
use std::{fs, path::Path};

const CONTRACT: &str =
    include_str!("../../../tests/_fixtures/patina-symbol-owner-11147/contracts.ts.txt");
const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/native-boolean-tsrs-11147/App.vue.txt"
);
const INLINE: &str =
    include_str!("../../../tests/_fixtures/patina-symbol-owner-11147/Inline.vue.txt");
const FOREIGN: &str =
    include_str!("../../../tests/_fixtures/patina-symbol-owner-11147/Foreign.vue.txt");

pub(super) fn record(dir: &Path, name: &str, value: &Value) {
    fs::write(
        dir.join(name),
        serde_json::to_vec_pretty(value).expect("serialize complete evidence"),
    )
    .expect("retain complete evidence");
}

#[test]
#[ignore = "manual bounded native ownership collector, not ordinary qualification"]
fn same_connection_foreign_callable_probe() {
    assert_eq!(std::env::var("VIZE_SYMBOL_OWNER_REPRO").as_deref(), Ok("1"));
    let provider =
        fs::canonicalize(std::env::var_os("CORSA_PATH").expect("exact provider must be explicit"))
            .expect("canonical provider");
    let label = std::env::var("VIZE_SYMBOL_OWNER_PROVIDER_LABEL").expect("provider label");
    assert!(matches!(label.as_str(), "stable" | "tsrs"));
    let package_root = if label == "stable" {
        provider.parent().unwrap().parent().unwrap()
    } else {
        provider.parent().unwrap()
    };
    let package: Value =
        serde_json::from_slice(&fs::read(package_root.join("package.json")).unwrap()).unwrap();
    let (name, version) = if label == "stable" {
        ("@typescript/typescript-linux-x64", "7.0.2")
    } else {
        ("@tsrs/linux-x64-gnu", "0.20.1-ts7.1.0-beta")
    };
    assert_eq!(package["name"], name);
    assert_eq!(package["version"], version);
    let evidence = std::path::PathBuf::from(
        std::env::var_os("VIZE_SYMBOL_OWNER_EVIDENCE_DIR").expect("evidence directory"),
    );
    fs::create_dir_all(&evidence).expect("evidence directory");
    record(
        &evidence,
        "collector.json",
        &json!({
            "scope":"causal instrumentation, not bug resolution or ordinary native qualification",
            "source":std::env::var("GITHUB_SHA").expect("Actions source"),
            "provider":provider,"label":label,"providerPackage":package,
            "cli":env!("CARGO_BIN_EXE_vize"),
            "packetScope":"actual complete SDK-adapted API JSON; not pre-adapter wire bytes",
            "projectBound":"two code files per isolated variant; no parallel/edit widening",
        }),
    );
    let mut failed_cases = Vec::new();
    for (name, prefix) in [
        (
            "inline",
            "type Formatter = (value: number | null) => string;\n",
        ),
        ("foreign", "import type { Formatter } from './contracts';\n"),
    ] {
        if std::panic::catch_unwind(|| counterfactual::collect(&evidence, &provider, name, prefix))
            .is_err()
        {
            failed_cases.push(format!("native-{name}"));
        }
    }
    for (name, source) in [
        ("original", ORIGINAL),
        ("inline", INLINE),
        ("foreign", FOREIGN),
    ] {
        if std::panic::catch_unwind(|| public_cli::collect(&evidence, &provider, name, source))
            .is_err()
        {
            failed_cases.push(format!("cli-{name}"));
        }
    }
    record(
        &evidence,
        "completion.json",
        &json!({
            "failedCases":failed_cases,
            "scope":"collector completion is not bug resolution or ordinary native qualification",
            "panicDetails":"complete original panic output is retained in the provider test log",
        }),
    );
    assert!(
        failed_cases.is_empty(),
        "failed causal collection cases: {failed_cases:?}"
    );
}
