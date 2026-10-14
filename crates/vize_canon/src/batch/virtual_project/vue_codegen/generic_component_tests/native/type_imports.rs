//! Declaration barrels must retain the same SFC types as direct TS imports.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "complete native source and public diagnostic controls use std strings"
)]

use super::{native, packets};
use crate::{BatchTypeChecker, BatchTypeCheckerOptions, BatchTypeCheckerTrait};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, path::Path};
use vize_carton::cstr;

const CORPUS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/declaration-barrel-vue-import/"
);

fn source(name: &str) -> std::string::String {
    fs::read_to_string(Path::new(CORPUS).join(name)).unwrap()
}

#[test]
fn declaration_barrel_and_direct_ts_vue_imports_keep_missing_neighbors_and_type_errors() {
    let Some(native) = native() else { return };
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let provider = repo.join("tests/node_modules").canonicalize().unwrap();
    assert!(provider.join("vue/package.json").is_file());
    let native_bytes = fs::read(&native).unwrap();
    let native_sha256: vize_carton::String = Sha256::digest(&native_bytes)
        .iter()
        .map(|byte| cstr!("{byte:02x}"))
        .collect();
    let mut outcomes = Vec::new();
    for barrel in [false, true] {
        for broken in [false, true] {
            let case = tempfile::tempdir().unwrap();
            let root = case.path().canonicalize().unwrap();
            std::os::unix::fs::symlink(&provider, root.join("node_modules")).unwrap();
            for (path, fixture) in [
                ("tsconfig.json", "tsconfig.json.txt"),
                (".generated/imports.d.ts", "imports.d.ts.txt"),
                ("src/components/Widget.vue", "Widget.vue.txt"),
                ("src/composables/useWidget.ts", "useWidget.ts.txt"),
                ("src/composables/useMissing.ts", "useMissing.ts.txt"),
            ] {
                let path = root.join(path);
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                fs::write(path, source(fixture)).unwrap();
            }
            let app = source("App.vue.txt")
                .replace(
                    "#imports",
                    if barrel {
                        "#imports"
                    } else {
                        "./composables/useWidget"
                    },
                )
                .replace("count: 1", if broken { "count: 'bad'" } else { "count: 1" });
            fs::write(root.join("src/App.vue"), app).unwrap();
            let mut checker = BatchTypeChecker::with_options_and_corsa_path(
                &root,
                BatchTypeCheckerOptions {
                    tsconfig_path: Some(root.join("tsconfig.json")),
                    ..Default::default()
                },
                Some(&native),
            )
            .unwrap();
            checker.set_server_count(Some(1));
            checker.set_checker_count(Some(1));
            checker.scan_project().unwrap();
            let public = checker.check_project();
            let name = cstr!("declaration-barrel-{barrel}-broken-{broken}");
            packets::retain(
                &name,
                &json!({
                    "scope":"independently authored complete TS/SFC project, same concrete import topology as retained Nuxt useToast/useFormField/useContentSearch",
                    "wholeVueManifest":fs::read_to_string(provider.join("vue/package.json")).unwrap(),
                    "providerCanonicalPath":provider.join("vue").canonicalize().unwrap(),
                    "nativeCanonicalPath":native,"nativeExecutableSha256":native_sha256,
                    "nativeExecutableBytes":native_bytes.len(),
                    "wholeAuthoredConfig":source("tsconfig.json.txt"),
                    "wholeAuthoredBarrel":source("imports.d.ts.txt"),
                    "wholePublicResult":cstr!("{public:#?}"),
                }),
            );
            let public =
                public.expect("the normal public batch must complete with all diagnostics");
            let files = checker.virtual_files();
            let paths = [
                root.join("src/App.vue"),
                root.join("src/components/Widget.vue"),
                root.join("src/composables/useWidget.ts"),
                root.join("src/composables/useMissing.ts"),
            ];
            let (mirror, config) = checker.native_project_paths_for_test();
            let (mapped, packet) = packets::diagnose_selected(
                &root,
                &native,
                &files,
                &name,
                "production",
                None,
                (&mirror, &config),
                &paths,
            );
            let authority = &packet["nativeAuthority"];
            assert_eq!(authority["nativeBinary"], json!(native), "{authority}");
            assert_eq!(
                authority["normalizedRequestedOptions"]["strict"], true,
                "{authority}"
            );
            assert_eq!(
                authority["sourceFileNamesResponse"]["status"], "ok",
                "{authority}"
            );
            let loaded = authority["sourceFileNamesResponse"]["wholeResponse"]
                .as_array()
                .unwrap();
            // Retain the failed pre-fix identity proof before checking the law.
            let original_widget_loaded =
                loaded.contains(&json!(root.join("src/composables/useWidget.ts")));
            let mut public_rows = public
                .diagnostics
                .iter()
                .map(|diagnostic| {
                    (
                        diagnostic
                            .file
                            .strip_prefix(&root)
                            .unwrap()
                            .to_string_lossy()
                            .into_owned(),
                        diagnostic.line,
                        diagnostic.column,
                        diagnostic.code,
                        diagnostic.severity,
                        diagnostic.message.to_string(),
                    )
                })
                .collect::<Vec<_>>();
            public_rows.sort();
            outcomes.push((barrel, broken, original_widget_loaded, mapped, public_rows));
        }
    }
    // Compare complete vectors. The missing neighboring SFC must survive both
    // variants; the repair is allowed to clear only the authored TS2322.
    for (barrel, broken, original_loaded, mapped, public) in &outcomes {
        assert!(
            !original_loaded,
            "authored TS leaked through declaration barrel: {outcomes:#?}"
        );
        let mut expected = vec![(
            "src/composables/useMissing.ts".to_string(),
            0,
            33,
            Some(2307),
            1,
            "Cannot find module '../components/Absent.vue' or its corresponding type declarations."
                .to_string(),
        )];
        if *broken {
            expected.push((
                "src/App.vue".to_string(),
                3,
                12,
                Some(2322),
                1,
                "Type 'string' is not assignable to type 'number'.".to_string(),
            ));
        }
        expected.sort();
        assert_eq!(
            public, &expected,
            "barrel={barrel}, broken={broken}, {outcomes:#?}"
        );
        let expected_mapped = expected
            .iter()
            .map(|(file, line, column, code, severity, message)| {
                (
                    file.as_str().into(),
                    *line,
                    *column,
                    *code,
                    Some(i32::from(*severity)),
                    message
                        .replace(
                            "Absent.vue'",
                            "Absent.vue.ts/__vize_missing_vue_import__.js'",
                        )
                        .into(),
                )
            })
            .collect::<Vec<packets::Row>>();
        assert_eq!(
            mapped, &expected_mapped,
            "barrel={barrel}, broken={broken}, {outcomes:#?}"
        );
    }
}
