//! `required` keeps its literal `true` type through `as const`, `<const>`,
//! parentheses, `satisfies` and `as true`, as Vue reads it. Reported with the
//! #13110 props (`required: true as const`): the template saw an optional prop
//! (TS18048) and a parent could omit it. `as boolean` and a widened imported
//! `required: true` stay optional.

use std::{fs, path::PathBuf};

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};

macro_rules! fixture {
    ($path:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/typechecker/runtime-required-wrappers-13110/",
            $path
        ))
    };
}

const FILES: [(&str, &str); 7] = [
    ("tsconfig.json", fixture!("tsconfig.json.txt")),
    ("src/props.ts", fixture!("src/props.ts.txt")),
    ("src/Imported.vue", fixture!("src/Imported.vue.txt")),
    ("src/Wrapped.vue", fixture!("src/Wrapped.vue.txt")),
    ("src/Controls.vue", fixture!("src/Controls.vue.txt")),
    ("src/AsBoolean.vue", fixture!("src/AsBoolean.vue.txt")),
    ("src/Parent.vue", fixture!("src/Parent.vue.txt")),
];

#[test]
fn standard_tsgo_keeps_required_literals_through_value_preserving_wrappers() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("runtime-required-wrappers-")
        .tempdir_in(cases_root)
        .unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());
    for (path, contents) in FILES {
        let path = project.path().join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    // vue-tsc 3.3.11 reports the same rows: both optional controls, and the
    // two required props the parent omits. Imported.vue and Wrapped.vue pass.
    let output = check_project(&tsgo, project.path(), "tsconfig.json");
    assert_eq!(output.status.code(), Some(2), "{}", output_text(&output));
    assert_eq!(output.stderr, b"", "{}", output_text(&output));
    assert_eq!(
        std::str::from_utf8(&output.stdout).unwrap(),
        concat!(
            "src/AsBoolean.vue(6,9): error TS18048: 'label' is possibly 'undefined'.\n",
            "src/Controls.vue(8,9): error TS18048: 'label' is possibly 'undefined'.\n",
            "src/Parent.vue(8,4): error TS2345: Argument of type '{ asConst: string; angleConst: string; parenthesized: string; satisfied: string; }' is not assignable to parameter of type 'ComponentPropsForTemplate<Props, VueTemplatePublicAttrsType & Record<string, unknown>>'.\n",
            "  Property '\"asTrue\"' is missing in type '{ asConst: string; angleConst: string; parenthesized: string; satisfied: string; }' but required in type '{ readonly asConst: string; readonly angleConst: string; readonly parenthesized: string; readonly satisfied: string; readonly asTrue: string; }'.\n",
            "src/Parent.vue(7,4): error TS2345: Argument of type '{}' is not assignable to parameter of type 'ComponentPropsForTemplate<Props, VueTemplatePublicAttrsType & Record<string, unknown>>'.\n",
            "  Property '\"label\"' is missing in type '{}' but required in type '{ readonly label: string; }'.\n",
        ),
        "{}",
        output_text(&output)
    );
}
