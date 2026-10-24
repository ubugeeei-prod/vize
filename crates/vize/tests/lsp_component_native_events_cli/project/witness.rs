//! Independently bind one live native mirror to this physical stdio fixture.
#![expect(
    clippy::string_slice,
    reason = "whole filesystem witnesses retain std strings and checked source offsets"
)]

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

const MARKER: &str = "vize-event-fixture";
const PACKAGE: &str = "{\"name\":\"vize-event-fixture\",\"version\":\"0.0.0\",\"private\":true}\n";
const SLOT_NAVIGATION_PRELUDE: &str = include_str!(
    "../../../../../tests/_fixtures/differential/lsp/component-native-events/slot-navigation-prelude.expected.txt"
);
const COMPONENT_NAVIGATION_HELPER: &str = "  type __VizeComponentNavigationProps<P> = __VizeIsAny<P> extends true ? Record<string, unknown> : unknown extends P ? {} : P;\n";

pub(crate) struct Witness {
    pub app_path: PathBuf,
    pub query_position: usize,
    pub historical_query_position: usize,
    pub child_import_utf16_delta: isize,
}

pub(crate) fn create_marker(root: &Path) {
    // An unreferenced bootstrap package exposes only an ownership link. No
    // original source, config, import, native type or generated code is added.
    let package = root.join("node_modules").join(MARKER);
    std::fs::create_dir(&package).unwrap();
    std::fs::write(package.join("package.json"), PACKAGE).unwrap();
}

pub(crate) fn observe(root: &Path, mut record: impl FnMut(Value)) -> Witness {
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
        "vite.config.mjs",
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
    assert_eq!(app.matches(COMPONENT_NAVIGATION_HELPER).count(), 1);
    let construct = "\n  void MySwitch;\n  const { } = undefined as unknown as __VizeComponentNavigationProps<__MySwitch_Props_0>;\n";
    assert_eq!(app.matches(construct).count(), 1);
    let cursor = app.find(construct).unwrap() + "\n  void MySwitch;\n  const { ".len();
    assert!(app.find(&mapping).unwrap() < cursor);
    // Native completion data uses a TypeScript UTF-16 file offset. Derive it
    // from this fixture's complete generated binding pattern, never its result.
    let query_position = app[..cursor].encode_utf16().count();
    receipt["query"] = json!({ "sourceRange": [source_start, source_end], "construct": construct, "byteCursor": cursor, "utf16Position": query_position });
    let (counterfactual, historical_query_position) = historical_counterfactual(&app, construct);
    receipt["navigationPreludeCounterfactual"] = counterfactual;
    let (import_relocation, child_import_utf16_delta) = child_import_relocation(&app, &child_path);
    receipt["childImportRelocation"] = import_relocation;
    record(receipt);
    let parsed: Value = serde_json::from_str(&config).unwrap();
    assert_eq!(
        parsed,
        serde_json::from_str::<Value>(include_str!(
            "../../../../../tests/_fixtures/differential/lsp/component-native-events/generated-tsconfig.expected.json"
        ))
        .unwrap()
    );
    Witness {
        app_path,
        query_position,
        historical_query_position,
        child_import_utf16_delta,
    }
}

fn file(path: &Path) -> Value {
    let source = std::fs::read_to_string(path).unwrap();
    json!({ "path": path, "source": source, "sha256": super::hash(source.as_bytes()) })
}

// The reference mirror was captured under /tmp. A host's canonical temp
// directory can have a different UTF-16 length; authenticate the exact child
// import before accounting for that independent, source-visible relocation.
fn child_import_relocation(app: &str, child: &Path) -> (Value, isize) {
    let before = include_str!(
        "../../../../../tests/_fixtures/differential/lsp/component-native-events/slot-navigation-current-source-counterfactual.txt"
    );
    let line = |source: &str| {
        source
            .lines()
            .filter(|line| line.starts_with("import MySwitch from "))
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    let previous = line(before);
    let current = line(app);
    assert_eq!(previous.len(), 1);
    assert_eq!(current.len(), 1);
    let previous = previous.first().unwrap();
    let current = current.first().unwrap();
    let literal = current
        .strip_prefix("import MySwitch from ")
        .unwrap()
        .strip_suffix(';')
        .unwrap();
    let imported: String = serde_json::from_str(literal).unwrap();
    assert_eq!(
        imported.replace('\\', "/"),
        child.to_string_lossy().replace('\\', "/")
    );
    let delta = isize::try_from(current.encode_utf16().count()).unwrap()
        - isize::try_from(previous.encode_utf16().count()).unwrap();
    (
        json!({ "beforeLine": previous, "currentLine": current, "utf16Delta": delta }),
        delta,
    )
}

#[test]
fn child_import_relocation_preserves_utf16_offsets_and_rejects_a_different_owner() {
    let source = include_str!(
        "../../../../../tests/_fixtures/differential/lsp/component-native-events/slot-navigation-current-source-counterfactual.txt"
    );
    let old = source
        .lines()
        .find(|line| line.starts_with("import MySwitch from "))
        .unwrap();
    let target = Path::new("/tmp/日本/😀/MySwitch.vue.ts");
    let current = source.replacen(
        old,
        &format!(
            "import MySwitch from {};",
            serde_json::to_string(&target.to_str().unwrap()).unwrap()
        ),
        1,
    );
    let (_, delta) = child_import_relocation(&current, target);
    assert_eq!(delta, -114);
    assert!(
        std::panic::catch_unwind(|| child_import_relocation(
            &current,
            Path::new("/tmp/other.vue.ts")
        ))
        .is_err()
    );
}

/// UTF-16 length of the generated slot-navigation prelude that precedes the query.
pub fn slot_navigation_prelude_utf16_len() -> usize {
    SLOT_NAVIGATION_PRELUDE.encode_utf16().count()
}

/// UTF-16 growth from the complete helper at the original test's source
/// baseline (da26dc4b2bb9f02492f8682929d29c4dfc1f7ec9). The older hosted
/// counterfactual remains untouched; it predates other helper changes too.
pub fn public_input_prelude_utf16_delta() -> usize {
    let current = include_str!("../../../../vize_canon/src/virtual_ts/component_prop_helpers.txt");
    let before = include_str!(
        "../../../../../tests/_fixtures/differential/lsp/component-native-events/public-input-prelude-before-9498.txt"
    );
    assert_eq!(
        super::hash(before.as_bytes()),
        "86a95b8a79b0c6dd2d17a79362ec9cd617bbc8a95e361387dc364cda0f2c945d"
    );
    current
        .encode_utf16()
        .count()
        .checked_sub(before.encode_utf16().count())
        .unwrap()
}

/// Keep the original hosted witness bytes and account for #10423's helper
/// growth separately from the component input and navigation declarations.
pub fn kebab_case_prelude_utf16_delta() -> usize {
    let before = include_str!(
        "../../../../../tests/_fixtures/differential/lsp/component-native-events/slot-navigation-before-source-counterfactual.txt"
    ).lines().find(|line| line.starts_with("type __VizeKebabCase<")).unwrap();
    assert_eq!(
        super::hash(before.as_bytes()),
        "252e573afbee269915bdda1cb0725b6cb723b7bba2f8bd1195ec51aca24263b9"
    );
    let current = vize_canon::virtual_ts::SHARED_PREAMBLE_DTS
        .lines()
        .find(|line| line.starts_with("type __VizeKebabCase<"))
        .unwrap();
    current
        .encode_utf16()
        .count()
        .checked_sub(before.encode_utf16().count())
        .unwrap()
}

/// UTF-16 length of the navigation helper added ahead of every prop query.
pub fn component_navigation_helper_utf16_len() -> usize {
    COMPONENT_NAVIGATION_HELPER.encode_utf16().count()
}

// A named source counterfactual preserves the old position; it is not an old backend run.
fn historical_counterfactual(app: &str, construct: &str) -> (Value, usize) {
    assert!(!SLOT_NAVIGATION_PRELUDE.is_empty());
    assert_eq!(
        app.matches(SLOT_NAVIGATION_PRELUDE).count(),
        1,
        "one exact slot-navigation prelude"
    );
    let source = app.replacen(SLOT_NAVIGATION_PRELUDE, "", 1);
    assert_eq!(source.matches(construct).count(), 1);
    let cursor = source.find(construct).unwrap() + "\n  void MySwitch;\n  const { ".len();
    let query_position = source[..cursor].encode_utf16().count();
    let digest = super::hash(source.as_bytes());
    (
        json!({
            "kind": "remove-only-exact-slot-navigation-prelude-source-counterfactual",
            "helperBytesRemoved": SLOT_NAVIGATION_PRELUDE.len(),
            "source": source,
            "sha256": digest,
            "byteCursor": cursor,
            "utf16Position": query_position,
            "historicalBackendExecution": false
        }),
        query_position,
    )
}

#[test]
fn complete_event_source_counterfactual_recovers_original_position() {
    // Authored final source is the whole original hosted mirror with only its known helper changed.
    let current = include_str!(
        "../../../../../tests/_fixtures/differential/lsp/component-native-events/slot-navigation-current-source-counterfactual.txt"
    );
    let original = include_str!(
        "../../../../../tests/_fixtures/differential/lsp/component-native-events/slot-navigation-before-source-counterfactual.txt"
    );
    let construct =
        "\n  void MySwitch;\n  const { } = undefined as unknown as __MySwitch_Props_0;\n";
    let (counterfactual, position) = historical_counterfactual(current, construct);
    assert_eq!(position, 20_850);
    assert_eq!(counterfactual["source"].as_str(), Some(original));
    let cursor = current.find(construct).unwrap() + "\n  void MySwitch;\n  const { ".len();
    assert_eq!(
        current[..cursor].encode_utf16().count(),
        position + slot_navigation_prelude_utf16_len()
    );
    let current_maps = current
        .lines()
        .filter(|line| line.contains("@vize-map"))
        .collect::<Vec<_>>();
    let original_maps = original
        .lines()
        .filter(|line| line.contains("@vize-map"))
        .collect::<Vec<_>>();
    assert_eq!(current_maps, original_maps);
    assert_eq!(
        current_maps,
        vec![
            "  // @vize-map: 14400:14502 -> 0:153",
            "  // @vize-map: expr -> 226:228",
            "  // @vize-map: component -> 201:214",
        ]
    );
}

#[test]
#[should_panic(expected = "one exact slot-navigation prelude")]
fn event_counterfactual_refuses_unknown_prelude() {
    let current = include_str!(
        "../../../../../tests/_fixtures/differential/lsp/component-native-events/slot-navigation-current-source-counterfactual.txt"
    );
    let unknown = current.replace("[keyof NonNullable<__S>]", "[keyof NonNullable<Unknown>]");
    historical_counterfactual(
        &unknown,
        "\n  void MySwitch;\n  const { } = undefined as unknown as __MySwitch_Props_0;\n",
    );
}
