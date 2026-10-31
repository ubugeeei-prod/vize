//! A real authored-name disk oracle, separate from the old physical policy.

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
