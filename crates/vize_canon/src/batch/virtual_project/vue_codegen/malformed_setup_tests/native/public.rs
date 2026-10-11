//! Public mirror/config acceptance is distinct from the authored-tree LSP control.

use super::{CorsaProjectClient, fs, install, json, native, path_to_file_uri, retain, sources};
use crate::{BatchTypeChecker, BatchTypeCheckerOptions, BatchTypeCheckerTrait};

#[test]
fn original_public_projects_keep_only_their_complete_authored_syntax_vectors() {
    let Some(native) = native() else { return };
    for issue in ["5071", "5823"] {
        let temporary = tempfile::tempdir().unwrap();
        let root = install(temporary.path(), issue);
        let authored = sources(issue);
        for (name, source) in &authored {
            fs::write(root.join(name), source).unwrap();
        }
        let mut checker = BatchTypeChecker::with_options_and_corsa_path(
            &root,
            BatchTypeCheckerOptions::default(),
            Some(&native),
        )
        .unwrap();
        checker.set_server_count(Some(1));
        checker.set_checker_count(Some(1));
        checker.scan_project().unwrap();
        let result = checker.check_project();
        let (mirror, config) = checker.native_project_paths_for_test();
        let files = checker.virtual_files();
        let canonical = root.canonicalize().unwrap();
        let uris = files
            .iter()
            .filter(|file| {
                file.original_path.starts_with(&canonical)
                    && file
                        .original_path
                        .extension()
                        .is_some_and(|extension| extension == "vue")
            })
            .map(|file| path_to_file_uri(&file.virtual_path))
            .collect::<Vec<_>>();
        let mut client = CorsaProjectClient::new_for_workspace_config(
            Some(native.to_str().unwrap()),
            &mirror,
            &config,
        )
        .unwrap();
        client.begin_batch_test_receipt(true);
        let raw = client.request_diagnostics_batch(&uris).unwrap();
        let authority = client.batch_test_receipt(uris.first().unwrap());
        let cli = std::process::Command::new(&native)
            .args(["--checkers", "1", "--pretty", "false", "--project"])
            .arg(&config)
            .current_dir(&mirror)
            .output()
            .unwrap();
        let packet = json!({
            "wholeAuthoredFiles":authored,
            "wholeAuthoredConfig":fs::read_to_string(root.join("tsconfig.json")).unwrap(),
            "wholeAuthoredBaseConfig":fs::read_to_string(root.parent().unwrap().parent().unwrap().join("tsconfig.base.json")).unwrap(),
            "publicSourceRoot":root,"publicMirrorRoot":mirror,
            "publicGeneratedConfigPath":config,
            "wholeGeneratedConfig":fs::read_to_string(&config).unwrap(),
            "wholeGeneratedPackage":fs::read_to_string(mirror.join("package.json")).ok(),
            "wholePublicResult":vize_carton::cstr!("{result:#?}"),
            "wholeGeneratedFiles":files.iter().map(|file| json!({
                "originalPath":file.original_path,"virtualPath":file.virtual_path,
                "wholeCode":file.content.as_str(),"wholeSourceMap":vize_carton::cstr!("{:#?}",file.source_map),
            })).collect::<Vec<_>>(),
            "generatedProjectControlAuthority":authority,
            "wholeGeneratedProjectControlRawVector":raw,
            "wholeNativeCliStdout":std::string::String::from_utf8(cli.stdout).unwrap(),
            "wholeNativeCliStderr":std::string::String::from_utf8(cli.stderr).unwrap(),
            "nativeCliExit":cli.status.code(),
            "authorityScope":"public result is from the real Batch checker; additional same-generated-config raw diagnostics are attested by their own returning client",
        });
        retain(&root, "public-generated-project", &packet);
        assert!(!mirror.starts_with(&canonical), "{packet}");
        assert_eq!(authority["selectedConfig"], json!(config), "{packet}");
        assert_eq!(
            authority["project"]["compilerOptions"], authority["normalizedRequestedOptions"],
            "{packet}"
        );
        assert!(authority["project"].is_object(), "{packet}");
        assert!(
            authority["normalizedRequestedOptions"].is_object(),
            "{packet}"
        );
        assert!(authority["snapshotProjects"].is_array(), "{packet}");
        assert!(
            matches!(
                authority["mode"].as_str(),
                Some("project-api" | "editor-lsp")
            ),
            "{packet}"
        );
        let result = result.expect("the original configured public checker must complete");
        let mut actual = result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                (
                    diagnostic
                        .file
                        .strip_prefix(&canonical)
                        .unwrap()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap(),
                    diagnostic.line,
                    diagnostic.column,
                    diagnostic.code,
                    diagnostic.severity,
                    diagnostic.message.as_str(),
                )
            })
            .collect::<Vec<_>>();
        actual.sort();
        let expected = if issue == "5071" {
            vec![
                ("withScript.vue", 0, 18, Some(1005), 1, "';' expected."),
                ("withoutScript.vue", 1, 25, Some(1005), 1, "';' expected."),
            ]
        } else {
            vec![
                (
                    "main.vue",
                    1,
                    8,
                    Some(6133),
                    2,
                    "'info' is declared but its value is never read.",
                ),
                ("main.vue", 5, 12, Some(1109), 1, "Expression expected."),
            ]
        };
        assert_eq!(actual, expected, "{packet}");
        assert!(
            !result.success,
            "the original authored syntax errors remain errors"
        );
        assert_eq!(uris.len(), if issue == "5071" { 2 } else { 1 });
    }
}
