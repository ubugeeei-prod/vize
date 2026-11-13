//! nuxt/ui and reka-ui parity regressions: union-host slot payloads, a
//! library consumed from a nested app, auto-imported template refs, public
//! `defineProps` type names, the slot-outlet option and global components
//! registered through a relative reference directive.
#![cfg(test)]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
use crate::check_canon_recent_issues_cli::{
    link_workspace_vue, resolve_test_corsa_path, symlink_path, unique_case_dir, workspace_root,
    write_tsconfig,
};
use crate::support::corsa_requirement;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use vize_l0::cstr;

const UNION_HOST_FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/union-host-slot-payload/"
);

/// Every diagnostic row of a `vize check --format json` run, in report order,
/// with the raw report for assertion messages.
fn check_diagnostic_rows(project_root: &Path, corsa_path: &Path) -> (Vec<String>, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", "--tsconfig", "tsconfig.json", "--format", "json"])
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap().to_string();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let rows = json["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| file["diagnostics"].as_array().unwrap().iter())
        .filter_map(serde_json::Value::as_str)
        .map(str::to_string)
        .collect();
    (rows, stdout)
}

/// A slot scope on a component picked from a union (nuxt/ui's `InputDate`
/// mounts `computed(() => range ? DateRangeField : DateField)`) receives the
/// props of whichever member renders. Inferring one parameter from the union of
/// both slot functions intersected the payloads, so the `v-else` branch of an
/// `Array.isArray` check narrowed to `never`.
#[test]
fn check_union_host_slot_payload_is_the_union_of_member_payloads() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = unique_case_dir("union-host-slot-payload");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    link_workspace_vue(&project_root).unwrap();
    write_tsconfig(&project_root);
    for name in ["Host.vue", "Single.vue", "Range.vue"] {
        let input = cstr!("{name}.txt");
        let source = std::fs::read(Path::new(UNION_HOST_FIXTURE).join(input.as_str())).unwrap();
        std::fs::write(project_root.join("src").join(name), source).unwrap();
    }

    let (rows, stdout) = check_diagnostic_rows(&project_root, &corsa_path);
    // vue-tsc 3.3.14 reports exactly this one row for the same project.
    assert_eq!(
        rows,
        vec![
            "error:14:66 [TS2339] Property 'toFixed' does not exist on type '{ part: string; }[]'. (source: <template v-else>{{ segments.start.length }} {{ segments.end.toFixed() }}</template>)"
        ],
        "{stdout}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

const NESTED_CONSUMER_FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/nested-consumer-rooted-stub/"
);

/// nuxt/ui keeps its `docs/` app below the library root, with its own
/// `package.json`, and the stub build's `dist/module.d.mts` re-exports the
/// source by a rooted path. A type exported from a library `.vue` must reach
/// the docs app: the docs import is not a self-reference, and the rooted
/// re-export must land on the mirrored source, not the unreadable original.
#[test]
fn check_nested_consumer_reaches_library_vue_types_through_a_rooted_stub() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let library = unique_case_dir("nested-consumer-rooted-stub");
    let _ = std::fs::remove_dir_all(&library);
    for dir in [
        "dist",
        "src/runtime/types",
        "src/runtime/components",
        "docs/src",
    ] {
        std::fs::create_dir_all(library.join(dir)).unwrap();
    }
    link_workspace_vue(&library).unwrap();
    std::fs::create_dir_all(library.join("docs/node_modules/@x")).unwrap();
    symlink_path(&library, &library.join("docs/node_modules/@x/ui")).unwrap();
    let canonical_library = std::fs::canonicalize(&library).unwrap();
    for (input, target) in [
        ("package.json.txt", "package.json"),
        ("module.d.mts.txt", "dist/module.d.mts"),
        ("module.ts.txt", "src/module.ts"),
        ("types-index.ts.txt", "src/runtime/types/index.ts"),
        ("Chip.vue.txt", "src/runtime/components/Chip.vue"),
        ("docs-package.json.txt", "docs/package.json"),
        ("docs-tsconfig.json.txt", "docs/tsconfig.json"),
        ("Use.vue.txt", "docs/src/Use.vue"),
    ] {
        let source = std::fs::read_to_string(Path::new(NESTED_CONSUMER_FIXTURE).join(input))
            .unwrap()
            .replace(
                "__PACKAGE_ROOT__",
                canonical_library.to_str().expect("utf-8 test path"),
            );
        std::fs::write(library.join(target), source).unwrap();
    }

    let (rows, stdout) = check_diagnostic_rows(&library.join("docs"), &corsa_path);
    // vue-tsc 3.3.14 reports exactly this one row for the same project.
    assert_eq!(
        rows,
        vec!["error:4:45 [TS2322] Type 'string' is not assignable to type 'boolean | undefined'."],
        "{stdout}"
    );

    let _ = std::fs::remove_dir_all(&library);
}

const AUTO_IMPORTED_TEMPLATE_REF_FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/auto-imported-template-ref/"
);

/// Nuxt auto-imports `useTemplateRef` as a global. Vue's language tools
/// recognize the composable by name (`composables.useTemplateRef` defaults to
/// `["useTemplateRef", "templateRef"]`), so the ref is the exposed instance of
/// the component that registers it, not `unknown`.
#[test]
fn check_auto_imported_use_template_ref_reads_the_template_registry() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = unique_case_dir("auto-imported-template-ref");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    link_workspace_vue(&project_root).unwrap();
    write_tsconfig(&project_root);
    for name in [
        "Input.vue",
        "Use.vue",
        "auto-imports.d.ts",
        "components.d.ts",
    ] {
        let input = cstr!("{name}.txt");
        let source =
            std::fs::read(Path::new(AUTO_IMPORTED_TEMPLATE_REF_FIXTURE).join(input.as_str()))
                .unwrap();
        std::fs::write(project_root.join("src").join(name), source).unwrap();
    }

    let (rows, stdout) = check_diagnostic_rows(&project_root, &corsa_path);
    // vue-tsc 3.3.14 reports exactly this one row for the same project.
    assert_eq!(
        rows,
        vec![
            "error:5:16 [TS2339] Property 'missing' does not exist on type '{ inputRef: HTMLInputElement | null; }'."
        ],
        "{stdout}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

/// A one-file project from a pinned fixture, with `vue_compiler_options`
/// merged into the shared test tsconfig.
fn create_fixture_case(
    name: &str,
    fixture: &str,
    file: &str,
    vue_compiler_options: Option<serde_json::Value>,
) -> PathBuf {
    let project_root = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    link_workspace_vue(&project_root).unwrap();
    write_tsconfig(&project_root);
    if let Some(options) = vue_compiler_options {
        let path = project_root.join("tsconfig.json");
        let mut config: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        config["vueCompilerOptions"] = options;
        std::fs::write(&path, serde_json::to_vec_pretty(&config).unwrap()).unwrap();
    }
    let input = cstr!("{file}.txt");
    let source = std::fs::read(
        workspace_root()
            .join("tests/_fixtures/differential/typechecker")
            .join(fixture)
            .join(input.as_str()),
    )
    .unwrap();
    std::fs::write(project_root.join("src").join(file), source).unwrap();
    project_root
}

/// The checker names Vize's private copy of Vue's `defineProps` result type;
/// the message must name it the way Vue declares it, as `vue-tsc` does.
#[test]
fn check_define_props_messages_use_vue_public_type_names() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_fixture_case(
        "define-props-public-name",
        "define-props-public-name",
        "Props.vue",
        None,
    );

    let (rows, stdout) = check_diagnostic_rows(&project_root, &corsa_path);
    // vue-tsc 3.3.14 reports exactly this one row for the same project.
    assert_eq!(
        rows,
        vec![
            "error:8:7 [TS2322] Type 'DefineProps<LooseRequired<Props>, \"disabled\">' is not assignable to type 'number'."
        ],
        "{stdout}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

/// A `<slot>` outlet's props are checked against the declared slot by
/// default, as `vue-tsc` does here; `checkSlotOutletProps: false` turns the
/// check off for projects that match `vue-tsc` where it misses it.
#[test]
fn check_slot_outlet_props_follow_the_vue_compiler_option() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    for (option, expected) in [
        (None, 1),
        (Some(serde_json::json!({ "checkSlotOutletProps": true })), 1),
        (
            Some(serde_json::json!({ "checkSlotOutletProps": false })),
            0,
        ),
    ] {
        let project_root = create_fixture_case(
            "slot-outlet-props-option",
            "slot-outlet-props-option",
            "Form.vue",
            option,
        );
        let (rows, stdout) = check_diagnostic_rows(&project_root, &corsa_path);
        let expected_rows: Vec<String> = [
            "error:9:26 [TS2322] Type 'boolean | undefined' is not assignable to type 'boolean'.\nType 'undefined' is not assignable to type 'boolean'. (source: <slot name=\"submit\" :loading=\"loading\" />; binding: 'loading')",
        ]
        .into_iter()
        .take(expected)
        .map(str::to_string)
        .collect();
        assert_eq!(rows, expected_rows, "{stdout}");
        let _ = std::fs::remove_dir_all(&project_root);
    }
}

const RELATIVE_REFERENCE_FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/relative-reference-global-components/"
);

/// reka-ui's `env.d.ts` reaches Histoire's `GlobalComponents` through a
/// relative `/// <reference types>` that leaves the project root. The copy in
/// the virtual project must still name that file, or `<Story>` is `any` and
/// its props go unchecked.
#[test]
fn check_relative_reference_directives_keep_global_components_typed() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let case_root = unique_case_dir("relative-reference-global-components");
    let _ = std::fs::remove_dir_all(&case_root);
    let project_root = case_root.join("app");
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    std::fs::create_dir_all(case_root.join("types")).unwrap();
    link_workspace_vue(&case_root).unwrap();
    write_tsconfig(&project_root);
    for (input, target) in [
        (
            "components.d.ts.txt",
            case_root.join("types/components.d.ts"),
        ),
        ("env.d.ts.txt", project_root.join("src/env.d.ts")),
        (
            "Layout.story.vue.txt",
            project_root.join("src/Layout.story.vue"),
        ),
    ] {
        let source = std::fs::read(Path::new(RELATIVE_REFERENCE_FIXTURE).join(input)).unwrap();
        std::fs::write(target, source).unwrap();
    }

    let (rows, stdout) = check_diagnostic_rows(&project_root, &corsa_path);
    // vue-tsc 3.3.14 reports exactly these rows for the same project.
    assert_eq!(
        rows,
        vec![
            "error:2:54 [TS2353] Object literal may only specify known properties, and 'width' does not exist in type '{ type: \"single\"; iframe?: boolean | undefined; }'. (source: <Story title=\"Splitter\" :layout=\"{ type: 'single', width: '100%' }\" />; binding: 'layout')",
            "error:3:11 [TS2322] Type 'number' is not assignable to type 'string'. (source: <Story :title=\"1\" />; binding: 'title')",
        ],
        "{stdout}"
    );

    let _ = std::fs::remove_dir_all(&case_root);
}
