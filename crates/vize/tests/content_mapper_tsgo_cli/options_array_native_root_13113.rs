//! #13113: an Options API child with string-array `emits` and one native root
//! types a declared listener as `(...args: any[]) => any`, like a fragment
//! root, `defineEmits([...])` and a `.ts` component. Its native root still
//! types undeclared and DOM events. The published SDK used to leave that
//! child's listener contract deferred, so `(a) => ...` was TS7006.

use std::path::{Path, PathBuf};

use super::{
    CompactString, TSGO_ENV, cstr, install_mapper_manifest, install_vue_package, output_text,
    run_tsgo, workspace_root,
};

macro_rules! reported {
    ($path:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/typechecker/options-array-native-root-13113/",
            $path
        ))
    };
}

/// The reporter's four children and parent. `Controls.vue` keeps an
/// undeclared event and DOM events on the native-root child typed by its root.
const SOURCES: [(&str, &str); 6] = [
    (
        "OptionsNativeRoot.vue",
        reported!("OptionsNativeRoot.vue.txt"),
    ),
    (
        "OptionsFragmentRoot.vue",
        reported!("OptionsFragmentRoot.vue.txt"),
    ),
    ("SetupNativeRoot.vue", reported!("SetupNativeRoot.vue.txt")),
    ("TsComponent.ts", reported!("TsComponent.ts.txt")),
    ("Parent.vue", reported!("Parent.vue.txt")),
    ("Controls.vue", reported!("Controls.vue.txt")),
];

const EXPECTED: &str = "src/Controls.vue(11,23): error TS2322: Type '(key: string, path: string[]) => void' is not assignable to type '(payload: Event) => void'.
  Target signature provides too few arguments. Expected 2 or more, but got 1.
src/Controls.vue(13,23): error TS2322: Type '(event: string) => void' is not assignable to type '(payload: KeyboardEvent) => void'.
  Types of parameters 'event' and 'payload' are incompatible.
    Type 'KeyboardEvent' is not assignable to type 'string'.
";

/// The published SDK as installed from npm, answered by the built mapper.
pub(super) fn install_published_sdk(project: &Path) {
    let package = project.join("node_modules/vize");
    let manifest_path = package.join("package.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
    let published: serde_json::Value =
        serde_json::from_str(include_str!("../../../../npm/cli/package.json")).unwrap();
    manifest["exports"] = published["exports"].clone();
    std::fs::write(manifest_path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
    for (name, contents) in [
        (
            "content-mapper-helpers.d.ts",
            include_str!("../../../../npm/cli/content-mapper-helpers.d.ts"),
        ),
        (
            "content-mapper-helpers.d.mts",
            include_str!("../../../../npm/cli/content-mapper-helpers.d.mts"),
        ),
        (
            "content-mapper-helpers.d.cts",
            include_str!("../../../../npm/cli/content-mapper-helpers.d.cts"),
        ),
        (
            "generated/content-mapper-types.d.ts",
            include_str!("../../../../npm/cli/generated/content-mapper-types.d.ts"),
        ),
        (
            "virtual-ts-checker.d.ts",
            include_str!("../../../../npm/cli/virtual-ts-checker.d.ts"),
        ),
        (
            "virtual-ts-checker-strict.d.ts",
            include_str!("../../../../npm/cli/virtual-ts-checker-strict.d.ts"),
        ),
        (
            "virtual-ts-emit-overloads.d.ts",
            include_str!("../../../../npm/cli/virtual-ts-emit-overloads.d.ts"),
        ),
        (
            "virtual-ts-emits.d.ts",
            include_str!("../../../../npm/cli/virtual-ts-emits.d.ts"),
        ),
        (
            "virtual-ts-globals.d.ts",
            include_str!("../../../../npm/cli/virtual-ts-globals.d.ts"),
        ),
        (
            "virtual-ts-listeners.d.ts",
            include_str!("../../../../npm/cli/virtual-ts-listeners.d.ts"),
        ),
        (
            "virtual-ts-patterns.d.ts",
            include_str!("../../../../npm/cli/virtual-ts-patterns.d.ts"),
        ),
        (
            "virtual-ts-refs.d.ts",
            include_str!("../../../../npm/cli/virtual-ts-refs.d.ts"),
        ),
        (
            "virtual-ts-slots.d.ts",
            include_str!("../../../../npm/cli/virtual-ts-slots.d.ts"),
        ),
        (
            "virtual-ts.d.ts",
            include_str!("../../../../npm/cli/virtual-ts.d.ts"),
        ),
        (
            "virtual-ts.d.mts",
            include_str!("../../../../npm/cli/virtual-ts.d.mts"),
        ),
        (
            "virtual-ts.d.cts",
            include_str!("../../../../npm/cli/virtual-ts.d.cts"),
        ),
    ] {
        let path = package.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }
}

#[test]
fn standard_tsgo_types_options_array_emits_on_a_native_root_child() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping Options API array emits on a native root: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let receipt = tempfile::Builder::new()
        .prefix("content-mapper-options-array-native-root-")
        .tempdir_in(&cases_root)
        .unwrap();
    for published_sdk in [false, true] {
        let mode = if published_sdk { "sdk" } else { "embedded" };
        let project = receipt.path().join(mode);
        install_mapper_manifest(&project);
        install_vue_package(&project);
        if published_sdk {
            install_published_sdk(&project);
        }
        let source_root = project.join("src");
        std::fs::create_dir_all(&source_root).unwrap();
        for (name, source) in SOURCES {
            std::fs::write(source_root.join(name), source).unwrap();
        }
        std::fs::write(
            project.join("tsconfig.json"),
            reported!("tsconfig.json.txt"),
        )
        .unwrap();
        let output = run_tsgo(
            &tsgo,
            &project,
            &[
                "--runExternalCode",
                "--noEmit",
                "-p",
                "tsconfig.json",
                "--pretty",
                "false",
                "--listFiles",
            ],
        );
        assert_eq!(output.stderr, b"", "{mode}: {}", output_text(&output));
        let stdout = std::str::from_utf8(&output.stdout).unwrap();
        let sdk = std::fs::canonicalize(project.join("node_modules/vize")).unwrap();
        let checker = sdk.join("virtual-ts-checker.d.ts");
        let loads_checker = stdout
            .lines()
            .any(|line| Path::new(line) == checker.as_path());
        assert_eq!(
            loads_checker,
            published_sdk,
            "{mode}: the projection must read the published SDK exactly when installed: {}",
            output_text(&output)
        );
        let diagnostics = stdout
            .lines()
            .filter(|line| !Path::new(line).is_absolute())
            .fold(CompactString::default(), |text, line| {
                cstr!("{text}{line}\n")
            });
        assert_eq!(
            diagnostics.as_str(),
            EXPECTED,
            "{mode}: {}",
            output_text(&output)
        );
        assert_eq!(
            output.status.code(),
            Some(2),
            "{mode}: {}",
            output_text(&output)
        );
    }
}
