use std::path::PathBuf;

use vize_canon::{BatchDiagnostic, SfcBlockType};

use super::is_suppressed_false_positive;

fn diagnostic(file: PathBuf, code: u32, message: &str) -> BatchDiagnostic {
    BatchDiagnostic {
        file,
        line: 0,
        column: 0,
        message: message.into(),
        code: Some(code),
        severity: 1,
        block_type: None,
    }
}

#[test]
fn suppresses_project_vue_wildcard_component_duplicates() {
    let temp = tempfile::tempdir().unwrap();
    let shim = temp.path().join("ts-shim.d.cts");
    std::fs::write(
        &shim,
        "declare module '*.vue' {\n  import Vue from 'vue';\n  export default Vue;\n}\n",
    )
    .unwrap();

    assert!(is_suppressed_false_positive(&diagnostic(
        shim,
        2300,
        "Duplicate identifier 'component'.",
    )));
}

#[test]
fn suppresses_nuxt_bridge_injection_duplicates_against_any() {
    let temp = tempfile::tempdir().unwrap();
    let shim = temp.path().join("gtag.d.mts");
    std::fs::write(
        &shim,
        "declare module '@nuxt/bridge-schema' {\n  interface Context { $gtag: Gtag.Gtag; }\n}\n",
    )
    .unwrap();

    assert!(is_suppressed_false_positive(&diagnostic(
        shim,
        2717,
        "Subsequent property declarations must have the same type.  Property '$gtag' must be of type 'any', but here has type 'Gtag'.",
    )));
}

#[test]
fn keeps_unrelated_declaration_conflicts_visible() {
    let temp = tempfile::tempdir().unwrap();
    let declaration = temp.path().join("conflict.d.ts");
    std::fs::write(
        &declaration,
        "declare module 'local' {\n  interface Context { value: string; }\n}\n",
    )
    .unwrap();

    assert!(!is_suppressed_false_positive(&diagnostic(
        declaration,
        2717,
        "Subsequent property declarations must have the same type.  Property 'value' must be of type 'number', but here has type 'string'.",
    )));
}

#[test]
fn suppresses_vue_expect_error_on_next_template_node() {
    let temp = tempfile::tempdir().unwrap();
    let component = temp.path().join("App.vue");
    std::fs::write(
        &component,
        "<template>\n  <!-- @vue-expect-error legacy payload -->\n  <Child :value=\"bad\" />\n</template>\n",
    )
    .unwrap();

    let mut diagnostic = diagnostic(component, 2322, "Type 'bad' is not assignable.");
    diagnostic.line = 2;

    assert!(is_suppressed_false_positive(&diagnostic));
}

#[test]
fn keeps_vue_diagnostic_without_adjacent_expect_error_visible() {
    let temp = tempfile::tempdir().unwrap();
    let component = temp.path().join("App.vue");
    std::fs::write(
        &component,
        "<template>\n  <!-- ordinary comment -->\n  <Child :value=\"bad\" />\n</template>\n",
    )
    .unwrap();

    let mut diagnostic = diagnostic(component, 2322, "Type 'bad' is not assignable.");
    diagnostic.line = 2;

    assert!(!is_suppressed_false_positive(&diagnostic));
}

#[test]
fn suppresses_native_truthiness_parity_diagnostic_in_vue_files() {
    let mut diagnostic = diagnostic(
        PathBuf::from("App.vue"),
        2801,
        "This condition will always return true since this 'Paginator' is always defined.",
    );
    diagnostic.block_type = Some(SfcBlockType::Template);

    assert!(is_suppressed_false_positive(&diagnostic));
}

#[test]
fn keeps_unrelated_ts2801_visible() {
    assert!(!is_suppressed_false_positive(&diagnostic(
        PathBuf::from("App.ts"),
        2801,
        "This condition will always return true since this 'Paginator' is always defined.",
    )));
}

#[test]
fn keeps_script_ts2801_visible_in_vue_files() {
    let mut diagnostic = diagnostic(
        PathBuf::from("App.vue"),
        2801,
        "This condition will always return true since this 'service' is always defined.",
    );
    diagnostic.block_type = Some(SfcBlockType::ScriptSetup);

    assert!(!is_suppressed_false_positive(&diagnostic));
}

#[test]
fn suppresses_corsa_recursive_discriminant_array_false_positive() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("construct-data-fetched.ts");
    std::fs::write(
        &source,
        r#"type FetchedTestQuestionComponent = {
  __typename?: "TestContentsQuestionComponent";
  testContentsComponentId: number;
};
type FetchedTestSectionComponent = {
  __typename?: "TestContentsSectionComponent";
  testContentsComponentId: number;
  childTestContentsComponents: FetchedTestComponent[];
};
type FetchedTestComponent =
  | FetchedTestQuestionComponent
  | FetchedTestSectionComponent;
"#,
    )
    .unwrap();

    assert!(is_suppressed_false_positive(&diagnostic(
        source,
        2345,
        recursive_discriminant_array_message(),
    )));
}

#[test]
fn suppresses_generic_recursive_discriminant_arrays() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("construct-data-fetched.ts");
    std::fs::write(
        &source,
        r#"type FetchedTestQuestionComponent = {
  __typename?: "TestContentsQuestionComponent";
};
type FetchedTestSectionComponent = {
  __typename?: "TestContentsSectionComponent";
  childTestContentsComponents: Array<FetchedTestComponent>;
};
type FetchedTestComponent =
  | FetchedTestQuestionComponent
  | FetchedTestSectionComponent;
"#,
    )
    .unwrap();

    assert!(is_suppressed_false_positive(&diagnostic(
        source,
        2345,
        recursive_discriminant_array_message(),
    )));
}

#[test]
fn keeps_recursive_discriminant_message_without_source_shape_visible() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("construct-data-fetched.ts");
    std::fs::write(
        &source,
        r#"type FetchedTestComponent = {
  __typename?: "TestContentsSectionComponent";
  childTestContentsComponents: OtherComponent[];
};
"#,
    )
    .unwrap();

    assert!(!is_suppressed_false_positive(&diagnostic(
        source,
        2345,
        recursive_discriminant_array_message(),
    )));
}

#[test]
fn keeps_declaration_recursive_discriminant_message_visible() {
    let temp = tempfile::tempdir().unwrap();
    let declaration = temp.path().join("construct-data-fetched.d.ts");
    std::fs::write(
        &declaration,
        "type FetchedTestComponent = { childTestContentsComponents: FetchedTestComponent[]; };",
    )
    .unwrap();

    assert!(!is_suppressed_false_positive(&diagnostic(
        declaration,
        2345,
        recursive_discriminant_array_message(),
    )));
}

fn recursive_discriminant_array_message() -> &'static str {
    "Argument of type '({ __typename?: \"TestContentsQuestionComponent\" | undefined; \
testContentsComponentId: number; } | { ...; })[]' is not assignable to parameter of type \
'FetchedTestComponent[]'.\nType '{ __typename?: \"TestContentsQuestionComponent\" | undefined; \
testContentsComponentId: number; } | { ...; }' is not assignable to type \
'FetchedTestComponent'.\nType '{ __typename?: \"TestContentsSectionComponent\" | undefined; \
testContentsComponentId: number; childTestContentsComponents: ({ ...; } | { ...; })[]; }' is \
not assignable to type 'FetchedTestSectionComponent'.\nTypes of property \
'childTestContentsComponents' are incompatible.\nType '({ __typename?: \
\"TestContentsQuestionComponent\" | undefined; testContentsComponentId: number; } | { ...; })[]' \
is not assignable to type 'FetchedTestComponent[]'.\nType '{ __typename?: \
\"TestContentsQuestionComponent\" | undefined; testContentsComponentId: number; } | { ...; }' is \
not assignable to type 'FetchedTestComponent'.\nType '{ __typename?: \
\"TestContentsSectionComponent\" | undefined; }' is not assignable to type \
'FetchedTestComponent'.\nType '{ __typename?: \"TestContentsSectionComponent\" | undefined; }' \
is missing the following properties from type 'FetchedTestSectionComponent': \
testContentsComponentId, testContentsComponentOrder, questionText, descriptionText, and 3 more."
}
