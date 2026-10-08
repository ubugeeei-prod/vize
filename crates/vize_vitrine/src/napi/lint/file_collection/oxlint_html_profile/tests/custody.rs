use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use vize_carton::cstr;

use super::super::{HostProfile, Refusal, Selection};
use super::Case;

#[derive(Debug, PartialEq, Eq, serde::Serialize)]
pub(super) enum Entry {
    Directory,
    File(Vec<u8>),
    Symlink(PathBuf),
    Other,
}

pub(super) fn snapshot(root: &Path) -> BTreeMap<PathBuf, Entry> {
    fn visit(root: &Path, path: &Path, entries: &mut BTreeMap<PathBuf, Entry>) {
        let metadata = fs::symlink_metadata(path).unwrap();
        let relative = path.strip_prefix(root).unwrap().to_path_buf();
        let entry = if metadata.is_symlink() {
            Entry::Symlink(fs::read_link(path).unwrap())
        } else if metadata.is_dir() {
            Entry::Directory
        } else if metadata.is_file() {
            Entry::File(fs::read(path).unwrap())
        } else {
            Entry::Other
        };
        entries.insert(relative, entry);
        if metadata.is_dir() {
            for entry in fs::read_dir(path).unwrap() {
                visit(root, &entry.unwrap().path(), entries);
            }
        }
    }
    let mut entries = BTreeMap::new();
    visit(root, root, &mut entries);
    entries
}

pub(super) struct Receipt {
    path: PathBuf,
    evidence: Value,
}

impl Receipt {
    pub(super) fn new(case: &Case, profile: HostProfile) -> Self {
        let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let receipt = std::env::var_os("VIZE_TEST_FIX_HISTORY_ARCHIVE_RECEIPT")
            .map(PathBuf::from)
            .unwrap_or_else(|| repository.join(".artifacts/rust-test-archive/receipt.json"));
        let receipt_bytes = fs::read(&receipt).ok();
        let source_identity = match profile {
            HostProfile::Oxlint178 => "c42d6397eab5b2d5bb2bd6746c57bc2a9cad21bd",
            HostProfile::Oxlint186 => "2ae2939bb2fd98796393658b21556b2a2467e047",
        };
        let evidence = json!({
            "contract": "private internal HTML producer only; wrapper and host HTML output unqualified",
            "host_profile": profile, "host_source": source_identity,
            "ignore": {"version":"0.4.33","checksum":"00b69833ed729dc5aa7d19541d96d6cf8e9137194207a04916d658e43168402f"},
            "compiled_archive_receipt_path": receipt, "compiled_archive_receipt_bytes": receipt_bytes,
            "authored_case": case, "actual_setup": [], "stage":"setup",
            "before": null, "actual": null, "after": null,
        });
        let test_profile = std::env::var("NEXTEST_PROFILE").unwrap_or_else(|_| "full".to_owned());
        let directory = repository
            .join("target/nextest")
            .join(test_profile)
            .join("oxlint-html-profile");
        fs::create_dir_all(&directory).unwrap();
        let filename = cstr!("{}-{profile:?}.json", case.name);
        let receipt = Self {
            path: directory.join(filename.as_str()),
            evidence,
        };
        receipt.persist();
        receipt
    }

    pub(super) fn setup(&mut self, journal: &[Value], success: bool) {
        self.evidence["actual_setup"] = serde_json::to_value(journal).unwrap();
        self.evidence["stage"] = json!(if success { "setup" } else { "setup-failed" });
        self.persist();
    }

    pub(super) fn before(&mut self, entries: &BTreeMap<PathBuf, Entry>) {
        self.evidence["before"] = serde_json::to_value(entries).unwrap();
        self.evidence["stage"] = json!("producer-pending");
        self.persist();
    }

    pub(super) fn actual(&mut self, actual: &Result<Selection, Refusal>) {
        self.evidence["actual"] = serde_json::to_value(actual).unwrap();
        self.evidence["stage"] = json!("producer-observed-before-postread");
        self.persist();
    }

    pub(super) fn after(&mut self, entries: &BTreeMap<PathBuf, Entry>) {
        self.evidence["after"] = serde_json::to_value(entries).unwrap();
        self.evidence["stage"] = json!("custody-observed-before-assertion");
        self.persist();
    }

    fn persist(&self) {
        fs::write(
            &self.path,
            serde_json::to_vec_pretty(&self.evidence).unwrap(),
        )
        .unwrap();
    }
}
