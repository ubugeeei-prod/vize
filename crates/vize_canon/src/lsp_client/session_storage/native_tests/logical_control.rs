//! A real authored-name disk oracle, separate from the old physical policy.

use super::{Value, hash, json};
use std::{fs, path::Path};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Policy {
    Original,
    Disk,
    Logical,
}

impl Policy {
    pub fn label(self) -> &'static str {
        match self {
            Self::Original => "original",
            Self::Disk => "logical-disk-control",
            Self::Logical => "logical-overlay",
        }
    }

    pub fn prepare(self, root: &Path, main: &str) {
        let path = root.join("src/App.vue.ts");
        assert!(!path.exists());
        if self == Self::Disk {
            fs::write(path, main).unwrap();
        }
    }

    pub fn write_dependency(self, root: &Path, text: &str) {
        if self == Self::Disk {
            fs::write(root.join("src/support.ts"), text).unwrap();
        }
    }

    pub fn finish(self, root: &Path, support: &str) {
        if self == Self::Disk {
            fs::remove_file(root.join("src/App.vue.ts")).unwrap();
            fs::write(root.join("src/support.ts"), support).unwrap();
        }
        assert!(!root.join("src/App.vue.ts").exists());
    }
}

pub(super) fn assert_configuration(packet: &serde_json::Value) {
    let expected = Path::new(packet["materializedRoot"].as_str().unwrap())
        .join("tsconfig.json")
        .canonicalize()
        .unwrap();
    for row in packet["rows"].as_array().unwrap() {
        let result = &row["diagnosingConfiguration"];
        let observed = Path::new(
            result["Ok"]
                .as_str()
                .expect("actual LSP configured-project observation required"),
        )
        .canonicalize()
        .unwrap();
        assert_eq!(
            observed, expected,
            "logical documents must retain authored config authority"
        );
    }
}

pub(super) fn run_disk(
    root: &Path,
    native: &Path,
    capture: &Path,
    main: &str,
    support: &str,
) -> serde_json::Value {
    use crate::file_uri::path_to_file_uri;
    let uri = path_to_file_uri(&root.join("src/App.vue.ts"));
    let mut rows = Vec::new();
    for (phase, text) in [
        ("valid", support),
        ("dependency-error", "export const value = 'bad';\n"),
        ("dependency-repair", "export const value = 2;\n"),
        ("dependency-close", support),
    ] {
        fs::write(root.join("src/support.ts"), text).unwrap();
        let packet = crate::lsp_client::editor_lsp::logical_disk_control::run(
            native,
            root,
            &uri,
            main,
            capture,
            phase,
            (2, 32),
            &root.join("src/support.ts"),
        );
        rows.push(packet.unwrap());
    }
    Policy::Disk.finish(root, support);
    serde_json::json!({"policy":"independent-standard-LSP-physical-disk-control",
        "materializedRoot":root,"generatedConfig":serde_json::from_slice::<serde_json::Value>(&fs::read(root.join("tsconfig.json")).unwrap()).unwrap(),
        "rows":rows,"shutdown":"complete"})
}

pub(super) fn inventory(root: &Path) -> Vec<Value> {
    fn visit(root: &Path, path: &Path, rows: &mut Vec<Value>) {
        let mut children: Vec<_> = fs::read_dir(path).unwrap().map(Result::unwrap).collect();
        children.sort_by_key(|entry| entry.file_name());
        for entry in children {
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).unwrap();
            let relative = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            if let Ok(target) = fs::read_link(&path) {
                rows.push(json!({"path":relative,"kind":"link","target":target}));
            } else if metadata.is_dir() {
                rows.push(json!({"path":relative,"kind":"directory"}));
                visit(root, &path, rows);
            } else {
                assert!(metadata.is_file());
                rows.push(json!({"path":relative,"kind":"file","bytes":metadata.len(),"sha256":hash(&fs::read(&path).unwrap()),"readonly":metadata.permissions().readonly(),"modified":metadata.modified().unwrap().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos().to_string()}));
            }
        }
    }
    let mut rows = Vec::new();
    visit(root, root, &mut rows);
    rows
}
