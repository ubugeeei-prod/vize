#![cfg(test)]
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "whole native and actual CLI RPC oracles"
)]

#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;
#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "support/inlay_native_oracle.rs"]
mod oracle;
#[path = "support/lsp_vue_project.rs"]
mod support;

use oracle::{NativeOracle, normalize_locations};
use serde_json::{Value, json};
use std::path::Path;
use support::Fixture;

const ORIGINAL: &str =
    include_str!("../../../tests/_fixtures/differential/lsp/computed-inlay-hints/App.vue.txt");
const CONFIG: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/computed-inlay-hints/tsconfig.json.txt"
);
const VIZE_CONFIG: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/computed-inlay-hints/vize.config.json.txt"
);
const STOCK_SCRIPT_DIAGNOSTICS: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/computed-inlay-hints/native-script-diagnostics.json"
);

fn workspace() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
}

fn vue_dependency() -> std::path::PathBuf {
    let store = workspace().join("node_modules/.pnpm");
    std::fs::read_dir(store)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("vue@3.5.41")
        })
        .map(|entry| entry.path().join("node_modules/vue"))
        .find(|path| path.join("package.json").is_file())
        .expect("the frozen workspace lock must supply reported Vue 3.5.41")
        .canonicalize()
        .unwrap()
}

fn range(source: &str) -> Value {
    let normalized = source.replace("\r\n", "\n");
    let line = normalized.bytes().filter(|byte| *byte == b'\n').count();
    let character = normalized
        .rsplit('\n')
        .next()
        .unwrap()
        .encode_utf16()
        .count();
    json!({"start":{"line":0,"character":0},"end":{"line":line,"character":character}})
}

fn script(source: &str) -> String {
    let start = source.find('>').unwrap() + 1;
    let end = source.find("</script>").unwrap();
    source[start..end].to_string()
}

fn capture(name: &str, sources: &[(&str, &str)], native: &Value, actual: &Value, runtime: &Path) {
    let Some(root) = std::env::var_os("VIZE_INLAY_HINT_CAPTURE") else {
        return;
    };
    let root = Path::new(&root).join(name);
    std::fs::create_dir_all(&root).unwrap();
    for (name, bytes) in sources {
        std::fs::write(root.join(name), bytes).unwrap();
    }
    std::fs::write(root.join("whole-vectors.json"),serde_json::to_vec_pretty(&json!({"sourceSha":std::env::var("SOURCE_SHA").ok(),"nativeBinary":runtime,"cliBinary":env!("CARGO_BIN_EXE_vize"),"native":native,"actual":actual})).unwrap()).unwrap();
}

#[test]
fn original_computed_types_match_complete_stock_native_hints_after_edits_and_ranges() {
    let Some(runtime) = corsa_requirement::required_or_skip::<std::path::PathBuf>(None) else {
        return;
    };
    let vue = vue_dependency();
    let package: Value =
        serde_json::from_slice(&std::fs::read(vue.join("package.json")).unwrap()).unwrap();
    assert_eq!(package["version"], "3.5.41");
    for newline in ["\n", "\r\n"] {
        let source = ORIGINAL.replace('\n', newline);
        let files = [("tsconfig.json", CONFIG), ("vize.config.json", VIZE_CONFIG)];
        let mut fixture =
            Fixture::new_with_pinned_vue_project(&source, "src/App.vue", &files, &vue);
        let mut native = NativeOracle::new(&runtime, &vue, CONFIG, "ts");
        assert_eq!(fixture.open(&source), json!([]));
        for (version, current) in [
            source.clone(),
            source.replace("\"on\" : \"off\"", "\"ready\" : \"waiting\""),
            source.clone(),
        ]
        .into_iter()
        .enumerate()
        {
            if version > 0 {
                assert_eq!(fixture.change(&current, version as i64 + 1), json!([]));
            }
            let original_script = script(&current);
            let expected = native.hints(
                &original_script,
                "typescript",
                range(&original_script),
                serde_json::from_str(STOCK_SCRIPT_DIAGNOSTICS).unwrap(),
            );
            assert_eq!(
                expected.as_array().unwrap().len(),
                3,
                "stock native must prove all three original bindings"
            );
            let actual = fixture.request_with(
                "textDocument/inlayHint",
                &current,
                "const on",
                json!({"range":range(&current)}),
            );
            capture(
                &format!("original-{}-{}", newline.len(), version),
                &[
                    ("App.vue", &current),
                    ("tsconfig.json", CONFIG),
                    ("vize.config.json", VIZE_CONFIG),
                ],
                &expected,
                &actual,
                &runtime,
            );
            let mut normalized_expected = expected.clone();
            let mut normalized_actual = actual.clone();
            native.project_authored_locations(&mut normalized_expected, &fixture.uri);
            normalize_locations(&mut normalized_expected);
            normalize_locations(&mut normalized_actual);
            assert_eq!(
                normalized_actual, normalized_expected,
                "complete checker hints including structured alias locations"
            );
            let width = current.lines().nth(4).unwrap().encode_utf16().count();
            let requested =
                json!({"start":{"line":4,"character":0},"end":{"line":4,"character":width}});
            let ranged = fixture.request_with(
                "textDocument/inlayHint",
                &current,
                "const on",
                json!({"range":requested}),
            );
            let expected_range: Vec<_> = actual
                .as_array()
                .unwrap()
                .iter()
                .filter(|hint| hint["position"]["line"] == 4)
                .cloned()
                .collect();
            assert_eq!(ranged, json!(expected_range));
        }
        native.shutdown();
        fixture.shutdown();
    }
}

#[test]
fn plain_ts_and_js_keep_checker_generic_alias_types_and_utf16_ranges() {
    let Some(runtime) = corsa_requirement::required_or_skip::<std::path::PathBuf>(None) else {
        return;
    };
    let vue = vue_dependency();
    for (extension, language, source) in [
        (
            "ts",
            "typescript",
            "import { computed } from 'vue';\r\ntype Row = { id: number };\r\n/*😀*/ const café = computed(() => new Map<string, Row[]>([['rows', [{id:1}]]]));\r\nvoid café.value;\r\n",
        ),
        (
            "js",
            "javascript",
            "import { computed } from 'vue';\r\n/*😀*/ const café = computed(() => ['Ada', 'Linus']);\r\nvoid café.value;\r\n",
        ),
    ] {
        let mut config: Value = serde_json::from_str(CONFIG).unwrap();
        config["compilerOptions"]["allowJs"] = json!(true);
        config["compilerOptions"]["checkJs"] = json!(true);
        config["compilerOptions"]["target"] = json!("ES2022");
        config["compilerOptions"]["module"] = json!("ESNext");
        config["include"] = json!(["src/**/*"]);
        let config = config.to_string();
        let files = [("tsconfig.json", config.as_str())];
        let mut fixture =
            Fixture::new_with_pinned_vue_project(ORIGINAL, "src/App.vue", &files, &vue);
        let uri = fixture.write_file(&format!("src/Plain.{extension}"), source);
        assert_eq!(fixture.open_file_as(&uri, source, language), json!([]));
        let mut native = NativeOracle::new(&runtime, &vue, &config, extension);
        let expected = native.hints(
            source,
            language,
            range(source),
            json!({"kind":"full","items":[]}),
        );
        assert_eq!(expected.as_array().unwrap().len(), 1);
        let actual = fixture.request_file_with(
            "textDocument/inlayHint",
            &uri,
            source,
            "café",
            json!({"range":range(source)}),
        );
        capture(
            &format!("plain-{extension}"),
            &[
                ("Plain.source", source),
                ("tsconfig.json", &config),
                ("App.vue", ORIGINAL),
                ("vize.config.json", &fixture.read_file("vize.config.json")),
            ],
            &expected,
            &actual,
            &runtime,
        );
        let mut expected = expected.clone();
        let mut actual = actual.clone();
        native.project_authored_locations(&mut expected, &uri);
        normalize_locations(&mut expected);
        normalize_locations(&mut actual);
        assert_eq!(actual, expected, "whole {language} alias type and metadata");
        assert_eq!(
            actual[0]["position"],
            json!({"line":if extension=="ts"{2}else{1},"character":17})
        );
        native.shutdown();
        fixture.shutdown();
    }
}
