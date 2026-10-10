use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};

use super::{CORPUS, Case, profile};

pub(super) fn source_bytes(value: &str) -> Vec<u8> {
    value.strip_prefix('@').map_or_else(
        || value.as_bytes().to_vec(),
        |filename| fs::read(Path::new(CORPUS).join(filename)).unwrap(),
    )
}

pub(super) fn write(path: &Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

#[derive(Debug, PartialEq, Eq, serde::Serialize)]
pub(super) enum Entry {
    Directory,
    File(Vec<u8>),
    Symlink(PathBuf),
    Other,
}

pub(super) fn snapshot(root: &Path) -> BTreeMap<PathBuf, Entry> {
    fn visit(root: &Path, path: &Path, result: &mut BTreeMap<PathBuf, Entry>) {
        let metadata = fs::symlink_metadata(path).unwrap();
        let entry = if metadata.is_symlink() {
            Entry::Symlink(fs::read_link(path).unwrap())
        } else if metadata.is_dir() {
            Entry::Directory
        } else if metadata.is_file() {
            Entry::File(fs::read(path).unwrap())
        } else {
            Entry::Other
        };
        result.insert(path.strip_prefix(root).unwrap().to_path_buf(), entry);
        if metadata.is_dir() {
            for child in fs::read_dir(path).unwrap() {
                visit(root, &child.unwrap().path(), result);
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    result
}

pub(super) struct Owned {
    pub temporary: tempfile::TempDir,
    pub root: PathBuf,
    pub config: PathBuf,
    receipt_path: PathBuf,
    evidence: Value,
}

impl Owned {
    pub(super) fn setup(case: &Case, host: profile::HostProfile) -> Self {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join("repo");
        let config = root.join(".oxlintrc.json");
        let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let archive = std::env::var_os("VIZE_TEST_FIX_HISTORY_ARCHIVE_RECEIPT")
            .map(PathBuf::from)
            .unwrap_or_else(|| repository.join(".artifacts/rust-test-archive/receipt.json"));
        let profile = std::env::var("NEXTEST_PROFILE").unwrap_or_else(|_| "full".to_owned());
        let directory = repository
            .join("target/nextest")
            .join(profile)
            .join("oxlint-html-operation");
        fs::create_dir_all(&directory).unwrap();
        let mut owned = Self {
            temporary,
            root,
            config,
            receipt_path: directory.join(vize_l0::cstr!("{}-{host:?}.json", case.name).as_str()),
            evidence: json!({
                "contract":"private configured original HTML operation; no public wrapper/renderer/installed qualification",
                "host_profile":host,
                "host_source":match host {
                    profile::HostProfile::Oxlint178=>"c42d6397eab5b2d5bb2bd6746c57bc2a9cad21bd",
                    profile::HostProfile::Oxlint186=>"2ae2939bb2fd98796393658b21556b2a2467e047",
                },
                "compiled_archive_receipt_path":archive,
                "compiled_archive_receipt_bytes":fs::read(&archive).ok(),
                "authored_case":case,
                "setup":[], "stage":"setup", "before":null,"actual":null,"after":null,
            }),
        };
        owned.persist();
        fs::create_dir(&owned.root).unwrap();
        let output = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&owned.root)
            .output();
        let (packet, successful) = match output {
            Ok(output) => (
                json!({"program":"git","argv":["init","-q"],"cwd":owned.root,
                "status":output.status.code(),"success":output.status.success(),"stdout":output.stdout,"stderr":output.stderr}),
                output.status.success(),
            ),
            Err(error) => (
                json!({"program":"git","argv":["init","-q"],"cwd":owned.root,"spawn_error":vize_l0::cstr!("{error}")}),
                false,
            ),
        };
        owned.evidence["setup"] = json!([packet]);
        owned.evidence["stage"] = json!(if successful { "setup" } else { "setup-failed" });
        owned.persist();
        assert!(successful, "owned genuine Git setup failed");
        write(&owned.root.join(".git/info/exclude"), b"");
        for (path, bytes) in &case.files {
            write(&owned.root.join(path), &source_bytes(bytes));
        }
        write(&owned.config, case.root.as_bytes());
        owned
    }

    fn persist(&self) {
        fs::write(
            &self.receipt_path,
            serde_json::to_vec_pretty(&self.evidence).unwrap(),
        )
        .unwrap();
    }

    pub(super) fn observe(&mut self, before: &BTreeMap<PathBuf, Entry>, actual: &Value) {
        self.evidence["before"] = serde_json::to_value(before).unwrap();
        self.evidence["actual"] = actual.clone();
        self.evidence["stage"] = json!("operation-observed-before-postread");
        self.persist();
    }

    pub(super) fn receipt(&mut self, after: &BTreeMap<PathBuf, Entry>) {
        self.evidence["after"] = serde_json::to_value(after).unwrap();
        self.evidence["stage"] = json!("custody-observed-before-assertion");
        self.persist();
    }
}
