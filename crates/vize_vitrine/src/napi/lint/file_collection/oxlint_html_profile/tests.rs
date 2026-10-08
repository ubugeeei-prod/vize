//! Expectations are authored corpus data, never generated from this producer.
#![expect(
    clippy::disallowed_types,
    reason = "the immutable JSON fixture ledger uses serde's std string keys"
)]
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::{HostProfile, Origin, Request, Selection, SourceRole, select};

mod custody;
mod setup;
use setup::{setup, source_bytes};

const CORPUS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lint/oxlint-original-html-profile-7903/"
);
const LEDGER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lint/oxlint-original-html-profile-7903/cases.json"
));

#[derive(Debug, Deserialize)]
struct Corpus {
    profiles: BTreeMap<String, String>,
    source_sha256: BTreeMap<String, String>,
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize, serde::Serialize)]
struct ExpectedRefusal {
    kind: String,
    path: String,
    bytes: Option<String>,
}

#[derive(Debug, Deserialize, serde::Serialize)]
struct Case {
    name: String,
    rationale: String,
    target: String,
    #[serde(default)]
    cwd: String,
    config: Option<String>,
    #[serde(default)]
    no_ignore: bool,
    #[serde(default)]
    cli: Vec<vize_l0::String>,
    custom: Option<String>,
    files: BTreeMap<String, String>,
    #[serde(default)]
    symlinks: BTreeMap<String, String>,
    fifo: Option<String>,
    #[serde(default)]
    force_track: Vec<String>,
    global_exclude: Option<String>,
    #[serde(default)]
    linked_git: bool,
    #[serde(default)]
    no_git: bool,
    #[serde(default)]
    selected: Vec<String>,
    selected_186: Option<Vec<String>>,
    root_decision: Option<String>,
    root_decision_186: Option<String>,
    #[serde(default)]
    authority_dirs: Vec<String>,
    refusal: Option<ExpectedRefusal>,
}

fn assert_selection(
    case: &Case,
    profile: HostProfile,
    root: &Path,
    cwd: &Path,
    config: &Path,
    actual: &Selection,
) {
    let expected_paths = if profile == HostProfile::Oxlint186 {
        case.selected_186.as_ref().unwrap_or(&case.selected)
    } else {
        &case.selected
    };
    let expected_decision = if profile == HostProfile::Oxlint186 {
        case.root_decision_186
            .as_ref()
            .or(case.root_decision.as_ref())
    } else {
        case.root_decision.as_ref()
    }
    .map(String::as_str)
    .unwrap_or("Eligible");
    assert_eq!(
        serde_json::to_value(actual.root_decision).unwrap(),
        expected_decision
    );
    assert_eq!(actual.host, profile);
    assert_eq!(actual.cwd, cwd);
    assert_eq!(actual.repository, root);
    assert_eq!(actual.root_json, config);
    assert_eq!(
        actual.target,
        std::path::absolute(cwd.join(&case.target)).unwrap()
    );
    assert_eq!(actual.literal_target.as_str(), case.target);
    assert_eq!(actual.no_ignore, case.no_ignore);
    assert_eq!(actual.cli_ignore_patterns, case.cli);
    assert_eq!(
        actual.custom_ignore_filename,
        case.custom.as_deref().unwrap_or(".eslintignore")
    );
    let paths = actual
        .originals
        .iter()
        .map(|file| file.cwd_relative.to_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        &paths, expected_paths,
        "{} {profile:?}: whole selected set",
        case.name
    );
    let origin = if root.join(&case.cwd).join(&case.target).is_dir() {
        Origin::DirectoryDiscovery
    } else {
        Origin::ExplicitFile
    };
    for original in &actual.originals {
        assert_eq!(original.path, cwd.join(&original.cwd_relative));
        assert_eq!(original.origin, origin);
        let declared = original.path.strip_prefix(root).unwrap().to_str().unwrap();
        assert_eq!(
            original.bytes,
            source_bytes(case.files.get(declared).unwrap()),
            "declared source custody"
        );
    }
    let custom = case.custom.as_deref().unwrap_or(".eslintignore");
    let mut sources = vec![
        (config.to_path_buf(), SourceRole::RootJson),
        (root.join(".git/info/exclude"), SourceRole::GitInfoExclude),
    ];
    for directory in &case.authority_dirs {
        let directory = std::path::absolute(root.join(directory)).unwrap();
        sources.push((directory.join(".gitignore"), SourceRole::GitIgnore));
        sources.push((directory.join(custom), SourceRole::CustomIgnore));
    }
    sources.sort_unstable();
    let actual_keys = actual
        .sources
        .iter()
        .map(|source| (source.path.clone(), source.role))
        .collect::<Vec<_>>();
    assert_eq!(
        actual_keys, sources,
        "whole declared authority incl absent sources"
    );
    for source in &actual.sources {
        // Whole before/after equality is established first, so these postreads
        // still compare original custody, never an output-derived golden.
        assert_eq!(source.bytes, fs::read(&source.path).ok());
        if source.path == config {
            assert_eq!(
                source.bytes.as_deref(),
                Some(case.config.as_deref().unwrap_or("{}\n").as_bytes())
            );
        } else if let Some(declared) = source
            .path
            .strip_prefix(root)
            .ok()
            .and_then(|path| path.to_str())
            .and_then(|name| case.files.get(name))
        {
            assert_eq!(
                source.bytes.as_deref(),
                Some(source_bytes(declared).as_slice())
            );
        } else if source.role == SourceRole::GitInfoExclude {
            assert_eq!(source.bytes.as_deref(), Some(b"".as_slice()));
        } else {
            assert_eq!(source.bytes, None, "authored absent authority");
        }
    }
}

#[test]
#[cfg(unix)]
fn original_html_profiles_match_independently_authored_corpus() {
    let corpus: Corpus = serde_json::from_str(LEDGER).unwrap();
    for (filename, expected) in &corpus.source_sha256 {
        let actual = Sha256::digest(fs::read(Path::new(CORPUS).join(filename)).unwrap());
        let hex = actual
            .iter()
            .fold(vize_l0::String::default(), |mut hex, byte| {
                hex.push_str(&vize_l0::cstr!("{byte:02x}"));
                hex
            });
        assert_eq!(hex.as_str(), expected, "frozen literal source identity");
    }
    assert_eq!(
        corpus.profiles.get("Oxlint178").unwrap(),
        "c42d6397eab5b2d5bb2bd6746c57bc2a9cad21bd"
    );
    assert_eq!(
        corpus.profiles.get("Oxlint186").unwrap(),
        "2ae2939bb2fd98796393658b21556b2a2467e047"
    );
    for case in corpus.cases {
        for profile in [HostProfile::Oxlint178, HostProfile::Oxlint186] {
            let temporary = tempfile::tempdir().unwrap();
            let root = temporary.path().join("repo");
            let mut journal = Vec::new();
            let mut receipt = custody::Receipt::new(&case, profile);
            let (cwd, config) = setup(&case, &root, &mut journal, &mut receipt);
            let before = custody::snapshot(temporary.path());
            receipt.before(&before);
            let actual = select(Request {
                cwd: &cwd,
                literal_target: &case.target,
                root_json: &config,
                host: profile,
                no_ignore: case.no_ignore,
                cli_ignore_patterns: &case.cli,
                custom_ignore_filename: case.custom.as_deref().unwrap_or(".eslintignore"),
            });
            // Preserve the whole producer observation before any custody postread.
            receipt.actual(&actual);
            let after = custody::snapshot(temporary.path());
            receipt.after(&after);
            assert_eq!(
                before, after,
                "{} {profile:?}: complete owned-tree custody",
                case.name
            );
            match (&case.refusal, actual) {
                (Some(expected), Err(error)) => {
                    assert_eq!(serde_json::to_value(error.kind).unwrap(), expected.kind);
                    let path = error.path.strip_prefix(&root).unwrap_or(&error.path);
                    let path = if path.as_os_str().is_empty() {
                        "."
                    } else {
                        path.to_str().unwrap()
                    };
                    assert_eq!(path, expected.path);
                    assert_eq!(
                        error.original_bytes.as_deref(),
                        expected.bytes.as_ref().map(|bytes| bytes.as_bytes())
                    );
                    assert!(!error.details.is_empty());
                }
                (None, Ok(selection)) => {
                    assert_selection(&case, profile, &root, &cwd, &config, &selection)
                }
                (expected, result) => panic!(
                    "{} {profile:?}: expected {expected:?}, actual {result:?}",
                    case.name
                ),
            }
        }
    }
}
