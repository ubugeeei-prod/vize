//! Only separately registered, authenticated physical inputs are additive.
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

const REGISTRY: &str = include_str!("../fixtures/projection-carriers-8757.json");
const PREFIXES: [&str; 2] = [
    "differential/lint/attrs-expression-ranges-8090/sources",
    "differential/lint/transition-custom-child-6881/sources",
];
const SVG_REGISTRY: &str = include_str!("../fixtures/projection-carriers-svg-9200.json");
const SVG_PREFIXES: [&str; 1] = ["differential/lint/native-svg-content"];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Registry {
    version: u8,
    source: String,
    entries: Vec<Entry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    path: String,
    bytes: usize,
    sha256: String,
    mode: String,
}

pub struct Carriers {
    root: PathBuf,
    entries: BTreeMap<PathBuf, Entry>,
    observed: BTreeSet<PathBuf>,
}

impl Carriers {
    pub fn new(root: &Path, files: &[PathBuf]) -> Self {
        Self::from_registry(
            root,
            files,
            REGISTRY,
            "d95c151c18d85c8fdd95541291bc614e44790101",
            49,
            &PREFIXES,
        )
    }

    pub fn svg(root: &Path, files: &[PathBuf]) -> Self {
        Self::from_registry(
            root,
            files,
            SVG_REGISTRY,
            "be62f9e2ae434bd52f776340615436e31d9b102a",
            2,
            &SVG_PREFIXES,
        )
    }

    fn from_registry(
        root: &Path,
        files: &[PathBuf],
        registry: &str,
        source: &str,
        count: usize,
        prefixes: &[&str],
    ) -> Self {
        let registry: Registry = serde_json::from_str(registry).expect("fixed carrier registry");
        assert_eq!(registry.version, 1);
        assert_eq!(registry.source, source);
        assert_eq!(registry.entries.len(), count);
        let mut entries = BTreeMap::new();
        for entry in registry.entries {
            let relative = Path::new(&entry.path);
            assert!(
                relative
                    .components()
                    .all(|part| matches!(part, std::path::Component::Normal(_)))
            );
            assert!(
                prefixes
                    .iter()
                    .any(|prefix| relative.parent() == Some(Path::new(prefix))),
                "carrier parent must equal a registered source directory"
            );
            assert_eq!(
                relative.extension().and_then(|value| value.to_str()),
                Some("vue")
            );
            assert_eq!(entry.mode, "100644");
            assert!(
                entries.insert(root.join(relative), entry).is_none(),
                "duplicate carrier path"
            );
        }
        let actual: BTreeSet<_> = files
            .iter()
            .filter(|file| {
                prefixes
                    .iter()
                    .any(|prefix| file.starts_with(root.join(prefix)))
            })
            .cloned()
            .collect();
        assert_eq!(
            actual,
            entries.keys().cloned().collect(),
            "carrier census drift"
        );
        Self {
            root: root.to_path_buf(),
            entries,
            observed: BTreeSet::new(),
        }
    }

    pub fn source(&mut self, path: &Path, source: &str) -> bool {
        let Some(entry) = self.entries.get(path) else {
            return false;
        };
        let metadata = std::fs::symlink_metadata(path).expect("carrier physical metadata");
        assert!(
            metadata.is_file(),
            "carrier must remain a physical regular file"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(metadata.permissions().mode() & 0o7777, 0o644);
        }
        assert_eq!(source.len(), entry.bytes);
        assert_eq!(
            Sha256::digest(source.as_bytes())
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>(),
            entry.sha256
        );
        assert!(
            self.observed.insert(path.to_path_buf()),
            "carrier was visited twice"
        );
        true
    }

    pub fn record(&self, path: &Path, diagnostic: Value) {
        if let Some(entry) = self.entries.get(path) {
            eprintln!(
                "projection carrier observation={}",
                serde_json::json!({
                    "path": path.strip_prefix(&self.root).expect("carrier relative path"),
                    "sourceSha256": entry.sha256,
                    "observation": diagnostic,
                })
            );
        }
    }

    pub fn finish(&self) {
        assert_eq!(
            self.observed,
            self.entries.keys().cloned().collect(),
            "carrier source observations incomplete"
        );
    }
}

#[cfg(test)]
mod controls {
    use super::*;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    fn fixture() -> (tempfile::TempDir, Vec<PathBuf>) {
        fixture_from(REGISTRY)
    }

    fn fixture_from(registry: &str) -> (tempfile::TempDir, Vec<PathBuf>) {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/_fixtures");
        let registry: Registry = serde_json::from_str(registry).expect("fixture registry");
        let temporary = tempfile::tempdir().expect("private carrier fixture");
        let mut files = Vec::new();
        for entry in registry.entries {
            let file = temporary.path().join(&entry.path);
            std::fs::create_dir_all(file.parent().expect("fixture parent")).expect("create parent");
            std::fs::copy(source.join(entry.path), &file).expect("copy whole fixture");
            files.push(file);
        }
        (temporary, files)
    }

    #[test]
    fn svg_originals_remain_complete_and_reject_missing_or_changed_inputs() {
        let (temporary, files) = fixture_from(SVG_REGISTRY);
        let mut carriers = Carriers::svg(temporary.path(), &files);
        for file in &files {
            let source = std::fs::read_to_string(file).expect("whole SVG fixture source");
            assert!(carriers.source(file, &source));
        }
        carriers.finish();
        assert!(catch_unwind(|| Carriers::svg(temporary.path(), &files[..1])).is_err());
        let mut changed = std::fs::read(&files[0]).expect("original SVG fixture");
        changed[0] ^= 1;
        std::fs::write(&files[0], changed).expect("same-size physical corruption");
        let source = std::fs::read_to_string(&files[0]).expect("changed SVG fixture source");
        let mut carriers = Carriers::svg(temporary.path(), &files);
        assert!(catch_unwind(AssertUnwindSafe(|| carriers.source(&files[0], &source))).is_err());
    }

    #[test]
    fn registered_complete_sources_pass_and_same_size_body_mutation_refuses() {
        let (temporary, files) = fixture();
        let mut carriers = Carriers::new(temporary.path(), &files);
        for file in &files {
            let source = std::fs::read_to_string(file).expect("whole fixture source");
            assert!(carriers.source(file, &source));
        }
        carriers.finish();
        let mut changed = std::fs::read(&files[0]).expect("authored source");
        changed[0] ^= 1;
        std::fs::write(&files[0], &changed).expect("write same-size physical corruption");
        let source = std::fs::read_to_string(&files[0]).expect("same-size authored corruption");
        let mut carriers = Carriers::new(temporary.path(), &files);
        assert!(catch_unwind(AssertUnwindSafe(|| carriers.source(&files[0], &source))).is_err());
    }

    #[test]
    fn missing_unknown_duplicate_and_unvisited_carriers_refuse() {
        let (temporary, files) = fixture();
        let mut missing = files.clone();
        missing.pop();
        assert!(catch_unwind(|| Carriers::new(temporary.path(), &missing)).is_err());
        let mut unknown = files.clone();
        unknown.push(temporary.path().join(PREFIXES[0]).join("Unknown.vue"));
        assert!(catch_unwind(|| Carriers::new(temporary.path(), &unknown)).is_err());
        let mut carriers = Carriers::new(temporary.path(), &files);
        assert!(catch_unwind(|| carriers.finish()).is_err());
        let source = std::fs::read_to_string(&files[0]).expect("original fixture source");
        carriers.source(&files[0], &source);
        assert!(catch_unwind(AssertUnwindSafe(|| carriers.source(&files[0], &source))).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn same_body_symlink_and_changed_physical_mode_refuse() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let (temporary, files) = fixture();
        let source = std::fs::read_to_string(&files[0]).expect("original fixture source");
        std::fs::set_permissions(&files[0], std::fs::Permissions::from_mode(0o600))
            .expect("change mode");
        let mut carriers = Carriers::new(temporary.path(), &files);
        assert!(catch_unwind(AssertUnwindSafe(|| carriers.source(&files[0], &source))).is_err());
        std::fs::set_permissions(&files[0], std::fs::Permissions::from_mode(0o644))
            .expect("restore private fixture mode");
        let original = temporary.path().join("whole-original.vue");
        std::fs::rename(&files[0], &original).expect("move private fixture");
        symlink(original, &files[0]).expect("author private symlink");
        let mut carriers = Carriers::new(temporary.path(), &files);
        assert!(catch_unwind(AssertUnwindSafe(|| carriers.source(&files[0], &source))).is_err());
    }
}
