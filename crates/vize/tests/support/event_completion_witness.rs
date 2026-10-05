//! Independently bind one live native mirror to this physical stdio fixture.
#![expect(
    clippy::string_slice,
    reason = "whole filesystem witnesses retain std strings and checked source offsets"
)]

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

const MARKER: &str = "vize-event-fixture";
const PACKAGE: &str = "{\"name\":\"vize-event-fixture\",\"version\":\"0.0.0\",\"private\":true}\n";

pub(super) struct Witness {
    pub app_path: PathBuf,
    pub query_position: usize,
}

pub(super) fn create_marker(root: &Path) {
    // An unreferenced bootstrap package exposes only an ownership link. No
    // original source, config, import, native type or generated code is added.
    let package = root.join("node_modules").join(MARKER);
    std::fs::create_dir(&package).unwrap();
    std::fs::write(package.join("package.json"), PACKAGE).unwrap();
}

pub(super) fn observe(root: &Path, mut record: impl FnMut(Value)) -> Witness {
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
    let mut matching = Vec::new();
    for session in std::fs::read_dir(&sessions).unwrap().flatten() {
        if !session.file_type().is_ok_and(|kind| kind.is_dir()) {
            continue;
        }
        let Ok(projects) = std::fs::read_dir(session.path().join("projects")) else {
            continue;
        };
        for project in projects.flatten() {
            if !project.file_type().is_ok_and(|kind| kind.is_dir()) {
                continue;
            }
            let link = project.path().join("node_modules").join(MARKER);
            if std::fs::read_link(&link).is_ok_and(|target| target == marker) {
                matching.push((project.path(), link));
            }
        }
    }
    assert_eq!(matching.len(), 1, "own fixture mirror: {matching:?}");
    let (mirror, link) = matching.pop().unwrap();
    let app_path = mirror.join("src/App.vue.ts");
    let child_path = mirror.join("src/MySwitch.vue.ts");
    let config_path = mirror.join("tsconfig.json");
    for path in [&app_path, &child_path, &config_path] {
        assert!(
            std::fs::symlink_metadata(path)
                .unwrap()
                .file_type()
                .is_file()
        );
        assert_eq!(&path.canonicalize().unwrap(), path);
    }
    let app = std::fs::read_to_string(&app_path).unwrap();
    let child = std::fs::read_to_string(&child_path).unwrap();
    let config = std::fs::read_to_string(&config_path).unwrap();
    let source = std::fs::read_to_string(root.join("src/App.vue")).unwrap();
    let authored: Vec<_> = [
        "src/App.vue",
        "src/MySwitch.vue",
        "tsconfig.json",
        "vize.config.json",
        "node_modules/vize-event-fixture/package.json",
    ]
    .into_iter()
    .map(|relative| file(&root.join(relative)))
    .collect();
    let mut receipt = json!({
        "physicalFixtureRoot": root,
        "markerLink": { "path": link, "target": std::fs::read_link(&link).unwrap() },
        "authored": authored,
        "generated": [
            { "path": app_path, "source": app, "sha256": super::hash(app.as_bytes()) },
            { "path": child_path, "source": child, "sha256": super::hash(child.as_bytes()) },
            { "path": config_path, "source": config, "sha256": super::hash(config.as_bytes()) }
        ]
    });
    // Persist whole generated sources even if an anchor/option assertion fails.
    record(receipt.clone());
    let tag = "<MySwitch  />";
    assert_eq!(source.matches(tag).count(), 1);
    let source_start = source.find(tag).unwrap();
    let source_end = source_start + tag.len();
    let mapping = format!("// @vize-map: component -> {source_start}:{source_end}\n");
    assert_eq!(app.matches(&mapping).count(), 1);
    let construct =
        "\n  void MySwitch;\n  const { } = undefined as unknown as __MySwitch_Props_0;\n";
    assert_eq!(app.matches(construct).count(), 1);
    let cursor = app.find(construct).unwrap() + "\n  void MySwitch;\n  const { ".len();
    assert!(app.find(&mapping).unwrap() < cursor);
    // Native completion data uses a TypeScript UTF-16 file offset. Derive it
    // from this fixture's complete generated binding pattern, never its result.
    let query_position = app[..cursor].encode_utf16().count();
    receipt["query"] = json!({ "sourceRange": [source_start, source_end], "construct": construct, "byteCursor": cursor, "utf16Position": query_position });
    record(receipt);
    let parsed: Value = serde_json::from_str(&config).unwrap();
    assert_eq!(
        parsed,
        serde_json::from_str::<Value>(include_str!(
            "../../../../tests/_fixtures/differential/lsp/component-native-events/generated-tsconfig.expected.json"
        ))
        .unwrap()
    );
    Witness {
        app_path,
        query_position,
    }
}

fn file(path: &Path) -> Value {
    let source = std::fs::read_to_string(path).unwrap();
    json!({ "path": path, "source": source, "sha256": super::hash(source.as_bytes()) })
}
