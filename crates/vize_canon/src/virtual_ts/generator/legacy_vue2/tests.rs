use std::{
    path::{Path, PathBuf},
    process::Command,
};

use vize_carton::{
    String, append,
    config::VueVersion,
    corsa_resolver::{CorsaResolveRequest, resolve_corsa_executable},
    cstr,
};

use super::vue_type_helpers;

#[test]
fn v_for_helper_preserves_member_array_aliases_without_losing_object_keys() {
    let Some(tsgo) = resolve_test_tsgo_binary() else {
        return;
    };
    let case_dir =
        std::env::temp_dir().join(cstr!("vize-legacy-vfor-helper-{}", std::process::id()).as_str());
    let _ = std::fs::remove_dir_all(&case_dir);
    std::fs::create_dir_all(&case_dir).unwrap();
    let test_path = case_dir.join("vfor-helper.ts");
    let mut source = String::new("");
    append!(source, "{}\n\n", vue_type_helpers(true, VueVersion::V2));
    source.push_str(
        r#"interface QuestionField {
  questionFieldInputType: string;
  question: string;
  type: string;
}
interface QuestionFormat {
  id: string;
  questionField: QuestionField[];
}

declare const anySource: any;
for (const [answer, i] of __vForList(anySource.questionField)) {
  const keyText = `${i}question`;
  answer.questionFieldInputType;
  void keyText;
}

const obj: { foo: number; bar: number } = { foo: 1, bar: 2 };
for (const [value, key] of __vForList(obj)) {
  const valueCheck: number = value;
  const keyCheck: "foo" | "bar" = key;
  void valueCheck;
  void keyCheck;
}

declare const questionFormat: QuestionFormat;
for (const [answer, i] of __vForList(questionFormat.questionField)) {
  const inputType: string = answer.questionFieldInputType;
  const index: number = i;
  void inputType;
  void index;
}
"#,
    );
    std::fs::write(&test_path, source.as_bytes()).unwrap();

    let output = Command::new(tsgo)
        .args([
            "--ignoreConfig",
            "--noEmit",
            "--strict",
            "--target",
            "ES2022",
        ])
        .arg(&test_path)
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert!(
        output.status.success(),
        "legacy v-for helper should keep any/member-array aliases off the object overload while preserving object key types\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&case_dir);
}

fn resolve_test_tsgo_binary() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("CORSA_PATH")
        && Path::new(&path).exists()
    {
        return Some(PathBuf::from(path));
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)?;
    resolve_corsa_executable(CorsaResolveRequest {
        explicit_path: None,
        project_root: Some(root),
    })
    .ok()
}

#[test]
fn every_embedded_vue_dialect_declares_the_same_structural_props_view() {
    let modern = vue_type_helpers(false, VueVersion::V3);
    let expected = modern
        .lines()
        .find(|line| line.starts_with("type __VizePrettify<T>"))
        .unwrap();
    for (legacy, dialect) in [
        (false, VueVersion::V3),
        (true, VueVersion::V3),
        (false, VueVersion::V2),
        (false, VueVersion::V2_7),
    ] {
        let helpers = vue_type_helpers(legacy, dialect);
        assert_eq!(
            helpers
                .lines()
                .filter(|line| line.starts_with("type __VizePrettify<T>"))
                .collect::<Vec<_>>(),
            [expected]
        );
    }
}

#[test]
fn legacy_structural_props_keep_native_loose_optional_union_and_generic_contracts() {
    let Some(tsgo) = resolve_test_tsgo_binary() else {
        assert!(std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none());
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("legacy-props.ts");
    let mut source = String::from(vue_type_helpers(false, VueVersion::V2_7));
    source.push_str(
        r#"
type Authored = { params: { value: string }; note?: string; flag?: boolean };
declare const props: __VizePrettify<__DefineProps<Authored>>;
const optionalText: string | undefined = props.note;
const optionalBoolean: boolean | undefined = props.flag;
const loose: __VizePrettify<__DefineProps<Authored>> = {
  params: { value: 'original' }, note: undefined, flag: undefined,
};
props.params.value = 'changed';
props.flag = true;
void optionalText;
void optionalBoolean;
void loose;
type Union = { kind: 'text'; value: string } | { kind: 'count'; value: number };
declare const union: __VizePrettify<Union>;
if (union.kind === 'text') union.value.toUpperCase();
else union.value.toFixed();
function generic<T extends { id: string }>(input: { items: T[]; selected?: T }) {
  const view: __VizePrettify<typeof input> = input;
  view.items.map(item => item.id);
  view.selected?.id;
}
void generic;
"#,
    );
    std::fs::write(&path, source.as_bytes()).unwrap();
    let output = Command::new(tsgo)
        .args([
            "--ignoreConfig",
            "--noEmit",
            "--strict",
            "--target",
            "ES2022",
        ])
        .arg(&path)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, b"");
}
