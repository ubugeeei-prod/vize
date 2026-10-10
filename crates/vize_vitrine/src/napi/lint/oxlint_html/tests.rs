//! Independently authored operation laws; the frozen selection corpus is unchanged.
#![expect(
    clippy::disallowed_types,
    reason = "immutable JSON corpus and whole owned-tree receipts use serde std strings"
)]

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::{Operation, profile, run, run_with};

mod custody;
mod mutation;
mod packets;

const CORPUS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lint/oxlint-original-html-operation-7903/"
);

#[derive(Debug, Deserialize)]
struct Corpus {
    profiles: BTreeMap<String, String>,
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize, serde::Serialize)]
struct ExpectedFile {
    path: String,
    packet: String,
}

#[derive(Debug, Deserialize, serde::Serialize)]
struct ExpectedRefusal {
    kind: String,
    path: String,
    bytes_ref: String,
}

#[derive(Debug, Deserialize, serde::Serialize)]
struct Case {
    name: String,
    rationale: String,
    target: String,
    root: String,
    expected_root: Option<String>,
    files: BTreeMap<String, String>,
    #[serde(default)]
    cli: Vec<vize_l0::String>,
    #[serde(default)]
    no_ignore: bool,
    selected: Vec<ExpectedFile>,
    selected_186: Option<Vec<ExpectedFile>>,
    root_decision: Option<String>,
    root_decision_186: Option<String>,
    projection: Value,
    refusal: Option<ExpectedRefusal>,
}

fn ledger() -> Corpus {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/lint/oxlint-original-html-operation-7903/cases.json"
    )))
    .unwrap()
}

fn expected_packets() -> BTreeMap<String, Value> {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/lint/oxlint-original-html-operation-7903/expected-packets.json"
    )))
    .unwrap()
}

fn request<'a>(
    case: &'a Case,
    root: &'a Path,
    config: &'a Path,
    host: profile::HostProfile,
) -> profile::Request<'a> {
    profile::Request {
        cwd: root,
        literal_target: &case.target,
        root_json: config,
        host,
        no_ignore: case.no_ignore,
        cli_ignore_patterns: &case.cli,
        custom_ignore_filename: ".eslintignore",
    }
}

fn assert_operation(case: &Case, host: profile::HostProfile, root: &Path, actual: &Operation) {
    let expected = if host == profile::HostProfile::Oxlint186 {
        case.selected_186.as_ref().unwrap_or(&case.selected)
    } else {
        &case.selected
    };
    let root_decision = if host == profile::HostProfile::Oxlint186 {
        case.root_decision_186
            .as_ref()
            .or(case.root_decision.as_ref())
    } else {
        case.root_decision.as_ref()
    };
    assert_eq!(
        serde_json::to_value(actual.selection.root_decision).unwrap(),
        root_decision.map(String::as_str).unwrap_or("Eligible")
    );
    assert_eq!(actual.selection.host, host);
    assert_eq!(actual.selection.cwd, root);
    assert_eq!(actual.selection.repository, root);
    assert_eq!(actual.selection.target, root.join(&case.target));
    assert_eq!(actual.selection.literal_target.as_str(), case.target);
    assert_eq!(actual.selection.root_json, root.join(".oxlintrc.json"));
    assert_eq!(actual.selection.no_ignore, case.no_ignore);
    assert_eq!(actual.selection.cli_ignore_patterns, case.cli);
    assert_eq!(actual.selection.custom_ignore_filename, ".eslintignore");
    assert_eq!(packets::projection(&actual.projection), case.projection);
    let root_source = actual
        .selection
        .sources
        .iter()
        .find(|source| source.role == profile::SourceRole::RootJson)
        .unwrap();
    assert_eq!(root_source.path, root.join(".oxlintrc.json"));
    assert_eq!(root_source.bytes.as_deref(), Some(case.root.as_bytes()));
    assert_eq!(actual.executed_file_count, expected.len());
    assert_eq!(actual.selection.originals.len(), expected.len());
    assert_eq!(actual.files.len(), expected.len());
    let packets = expected_packets();
    for ((original, file), expected) in actual
        .selection
        .originals
        .iter()
        .zip(&actual.files)
        .zip(expected)
    {
        assert_eq!(original.path, root.join(&expected.path));
        assert_eq!(original.cwd_relative, Path::new(&expected.path));
        assert_eq!(file.path, original.path);
        let origin = if case.target == "." {
            profile::Origin::DirectoryDiscovery
        } else {
            profile::Origin::ExplicitFile
        };
        assert_eq!(original.origin, origin);
        assert_eq!(
            original.bytes,
            custody::source_bytes(&case.files[&expected.path])
        );
        assert_eq!(
            file.result.filename.as_str(),
            original.path.to_str().unwrap()
        );
        assert_eq!(
            packets::lint_result(&file.result),
            packets[&expected.packet],
            "{}: complete ordered packet for {}",
            case.name,
            expected.path
        );
    }
}

#[test]
#[cfg(unix)]
fn original_html_operation_matches_authored_native_corpus() {
    let corpus = ledger();
    assert_eq!(
        corpus.profiles["Oxlint178"],
        "c42d6397eab5b2d5bb2bd6746c57bc2a9cad21bd"
    );
    assert_eq!(
        corpus.profiles["Oxlint186"],
        "2ae2939bb2fd98796393658b21556b2a2467e047"
    );
    for case in corpus.cases {
        assert!(!case.rationale.is_empty());
        for host in [
            profile::HostProfile::Oxlint178,
            profile::HostProfile::Oxlint186,
        ] {
            let mut owned = custody::Owned::setup(&case, host);
            let before = custody::snapshot(owned.temporary.path());
            let actual = run(
                request(&case, &owned.root, &owned.config, host),
                case.expected_root
                    .as_deref()
                    .unwrap_or(&case.root)
                    .as_bytes(),
            );
            // Capture the whole native packet before any custody postread/assertion.
            let observation = packets::observation(&actual);
            owned.observe(&before, &observation);
            let after = custody::snapshot(owned.temporary.path());
            owned.receipt(&after);
            assert_eq!(
                before, after,
                "{} {host:?}: whole owned-tree custody",
                case.name
            );
            match (&case.refusal, actual) {
                (None, Ok(operation)) => assert_operation(&case, host, &owned.root, &operation),
                (Some(expected), Err(refusal)) => {
                    assert_eq!(serde_json::to_value(refusal.kind).unwrap(), expected.kind);
                    assert_eq!(refusal.path, owned.root.join(&expected.path));
                    let bytes = if expected.bytes_ref == "@root" {
                        case.root.as_bytes().to_vec()
                    } else {
                        custody::source_bytes(&expected.bytes_ref)
                    };
                    assert_eq!(refusal.original_bytes, Some(bytes));
                    assert!(!refusal.details.is_empty());
                }
                (expected, _) => panic!(
                    "{} {host:?}: expected {expected:?}, actual {observation}",
                    case.name
                ),
            }
        }
    }
}

#[test]
fn operation_sources_and_original_copies_keep_fixed_identity() {
    let ledger: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/lint/oxlint-original-html-operation-7903/sources.json"
    )))
    .unwrap();
    for (name, expected) in ledger["sha256"].as_object().unwrap() {
        let bytes = fs::read(Path::new(CORPUS).join(name)).unwrap();
        let hex =
            Sha256::digest(&bytes)
                .iter()
                .fold(vize_l0::String::default(), |mut value, byte| {
                    value.push_str(&vize_l0::cstr!("{byte:02x}"));
                    value
                });
        assert_eq!(hex.as_str(), expected.as_str().unwrap());
    }
    for (copy, original) in ledger["copies"].as_object().unwrap() {
        assert_eq!(
            fs::read(Path::new(CORPUS).join(copy)).unwrap(),
            fs::read(Path::new(CORPUS).join(original.as_str().unwrap())).unwrap(),
            "exact immutable original copy"
        );
    }
}
