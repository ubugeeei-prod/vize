//! Bind native data to this fixture's complete generated source before RPC.
#![expect(
    clippy::string_slice,
    reason = "checked source spans in independent witnesses"
)]

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::source_digest;

const MARKER: &str = "vize-data-aria-fixture";
const PACKAGE: &str = "{\"name\":\"vize-data-aria-fixture\",\"private\":true}\n";

pub(super) struct Observed {
    pub path: PathBuf,
    pub position: usize,
}

pub(super) fn create_marker(root: &Path) {
    let marker = root.join("node_modules").join(MARKER);
    std::fs::create_dir(&marker).unwrap();
    std::fs::write(marker.join("package.json"), PACKAGE).unwrap();
}

pub(super) fn observe(root: &Path, source: &str, capture: Option<&Path>) -> Observed {
    let root = root.canonicalize().unwrap();
    let marker = root.join("node_modules").join(MARKER);
    assert_eq!(
        std::fs::read_to_string(marker.join("package.json")).unwrap(),
        PACKAGE
    );
    let sessions = std::env::temp_dir()
        .canonicalize()
        .unwrap()
        .join("vize-canon/editor/sessions");
    let mut mirrors = Vec::new();
    for session in std::fs::read_dir(sessions).unwrap().flatten() {
        let Ok(projects) = std::fs::read_dir(session.path().join("projects")) else {
            continue;
        };
        for project in projects.flatten() {
            let link = project.path().join("node_modules").join(MARKER);
            if std::fs::read_link(&link).is_ok_and(|target| target == marker) {
                mirrors.push(project.path());
            }
        }
    }
    assert_eq!(mirrors.len(), 1, "own fixture mirror: {mirrors:?}");
    let mirror = mirrors.pop().unwrap();
    let path = mirror.join("Owner.vue.ts");
    assert!(
        std::fs::symlink_metadata(&path)
            .unwrap()
            .file_type()
            .is_file()
    );
    assert_eq!(path.canonicalize().unwrap(), path);
    let generated = std::fs::read_to_string(&path).unwrap();
    let child = std::fs::read_to_string(mirror.join("Declared.vue.ts")).unwrap();
    let config = std::fs::read_to_string(mirror.join("tsconfig.json")).unwrap();
    let construct =
        "\n  void Declared;\n  const { } = undefined as unknown as __Declared_Props_0;\n";
    let tag_start = source.find("<Declared").unwrap();
    let tag_end = tag_start + source[tag_start..].find('>').unwrap() + 1;
    let mapping = format!("// @vize-map: component -> {tag_start}:{tag_end}\n");
    let cursor = generated
        .find(construct)
        .map(|start| start + "\n  void Declared;\n  const { ".len());
    let receipt: Value = json!({
        "physicalRoot":root,"source":source,"sourceSha256":source_digest(source),
        "generated":[{"path":path,"source":generated,"sha256":source_digest(&generated)},
            {"path":mirror.join("Declared.vue.ts"),"source":child,"sha256":source_digest(&child)},
            {"path":mirror.join("tsconfig.json"),"source":config,"sha256":source_digest(&config)}],
        "mapping":mapping,"construct":construct,"byteCursor":cursor
    });
    if let Some(capture) = capture {
        let name = format!("data-aria-mirror-{}.json", source_digest(source));
        std::fs::write(
            capture.join(name),
            serde_json::to_vec_pretty(&receipt).unwrap(),
        )
        .unwrap();
    }
    assert_eq!(generated.matches(construct).count(), 1, "{receipt:#}");
    assert_eq!(generated.matches(&mapping).count(), 1, "{receipt:#}");
    let cursor = cursor.unwrap();
    assert!(generated.find(&mapping).unwrap() < cursor);
    // No observed completion item supplies these expected paths or positions.
    Observed {
        path,
        position: generated[..cursor].encode_utf16().count(),
    }
}
