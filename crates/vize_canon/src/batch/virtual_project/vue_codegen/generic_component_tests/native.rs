//! Mandatory native cause and explicitly required pinned-project qualification.
#![expect(clippy::expect_used, reason = "required native fixture authority")]

mod generated_diagnostics;
mod install;
mod minimal;
mod misskey_generated_diagnostics;
mod packets;
mod replay;
mod tree;
mod type_imports;

use super::{CONFIG_PATH, GENERIC, ROOT_PATH, STORY_PATH, root_source, source};
use crate::{BatchTypeChecker, BatchTypeCheckerOptions, BatchTypeCheckerTrait};
use std::{fs, path::Path};
use vize_carton::{String, cstr};

fn native() -> Option<std::path::PathBuf> {
    let required = std::env::var("VIZE_TEST_REQUIRE_TSGO").as_deref() == Ok("1")
        || std::env::var("VIZE_TEST_REQUIRE_PIN_INPUT_FIXTURE").as_deref() == Ok("1");
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        assert!(
            !required,
            "required generic native controls cannot be disabled"
        );
        return None;
    }
    Some(
        vize_carton::corsa_resolver::resolve_corsa_executable(
            vize_carton::corsa_resolver::CorsaResolveRequest {
                explicit_path: std::env::var_os("CORSA_PATH").as_deref().map(Path::new),
                project_root: Some(Path::new(env!("CARGO_MANIFEST_DIR"))),
            },
        )
        .expect("the actual native runtime is required")
        .canonicalize()
        .unwrap(),
    )
}

#[test]
fn generic_native_contract_requires_the_self_contained_cause_and_requested_pinned_fixture() {
    let Some(native) = native() else { return };
    let minimal = tempfile::tempdir().unwrap();
    let (root, custody) = minimal::install(minimal.path());
    packets::retain("minimal-fixture", &custody);
    qualify(
        &root,
        &native,
        "minimal",
        &source("minimal-story.vue.txt"),
        minimal::root_source,
    );
    if std::env::var("VIZE_TEST_REQUIRE_PIN_INPUT_FIXTURE").as_deref() == Ok("1") {
        let pinned = tempfile::tempdir().unwrap();
        let (root, custody) = install::snapshot(pinned.path());
        packets::retain("fixture", &custody);
        qualify(
            &root,
            &native,
            "pinned",
            &source("PinInput.story.vue.txt"),
            root_source,
        );
    }
}

fn qualify(
    root: &Path,
    native: &Path,
    scope: &str,
    original: &str,
    child_source: fn(&str) -> String,
) {
    let mut cases = Vec::new();
    for (name, generic, number_prop, number_value, expected_error) in [
        ("original-text", String::from(GENERIC), false, false, false),
        ("explicit-number", String::from(GENERIC), true, true, false),
        ("wrong-number", String::from(GENERIC), true, false, true),
        ("authored-any", cstr!("{GENERIC} = any"), false, false, true),
        (
            "authored-text",
            cstr!("{GENERIC} = 'text'"),
            false,
            false,
            false,
        ),
        (
            "authored-number",
            cstr!("{GENERIC} = 'number'"),
            false,
            true,
            false,
        ),
    ] {
        let mut story = String::from(original);
        if number_value {
            story = story
                .replace("ref<string[]>([])", "ref<number[]>([])")
                .into();
        }
        if number_prop {
            story = story
                .replace("id=\"otp\"", "id=\"otp\" type=\"number\"")
                .into();
        }
        let child = child_source(&generic);
        fs::write(root.join(ROOT_PATH), child.as_str()).unwrap();
        fs::write(root.join(STORY_PATH), story.as_str()).unwrap();
        let mut checker = BatchTypeChecker::with_options_and_corsa_path(
            root,
            BatchTypeCheckerOptions {
                tsconfig_path: Some(root.join(CONFIG_PATH)),
                ..Default::default()
            },
            Some(native),
        )
        .unwrap();
        checker.set_server_count(Some(1));
        checker.set_checker_count(Some(1));
        checker
            .scan_paths(&[root.join(ROOT_PATH), root.join(STORY_PATH)])
            .unwrap();
        let public = checker.check_project();
        packets::retain(
            &cstr!("{scope}-{name}-public"),
            &serde_json::json!({
                "declaredSourceConfig":root.join(CONFIG_PATH),
                "scope":scope,
                "configScope":"this control explicitly selects its own config; the unchanged full App row separately qualifies the original CLI's unspecified selection",
                "wholePublicResult":cstr!("{public:#?}"),
            }),
        );
        public.expect("the genuine configured public checker must complete");
        let (mirror, config) = checker.native_project_paths_for_test();
        let files = checker.virtual_files();
        let child = files
            .iter()
            .find(|file| file.original_path == root.join(ROOT_PATH))
            .unwrap();
        // Retain the actual produced project before validating the replay's
        // unmapped-tail premise, so a failed premise cannot lose its packet.
        let (after, after_packet) = packets::diagnose(
            root,
            native,
            &files,
            &cstr!("{scope}-{name}"),
            "production-after",
            None,
            (&mirror, &config),
        );
        let before = replay::prior_contract(child, &generic);
        fs::write(&child.virtual_path, before.as_str()).unwrap();
        let (old, old_packet) = packets::diagnose(
            root,
            native,
            &files,
            &cstr!("{scope}-{name}"),
            "prior-contract-replay",
            Some(packets::Replay {
                path: &child.virtual_path,
                code: &before,
                scope: "prior helper-contract declaration replay in the actual unmapped Root export tail; not an unavailable old factory/native execution",
            }),
            (&mirror, &config),
        );
        fs::write(&child.virtual_path, child.content.as_str()).unwrap();
        cases.push((
            name,
            story,
            expected_error,
            old,
            old_packet,
            after,
            after_packet,
        ));
    }
    // Save every original/explicit/default case before asserting any verdict.
    for (name, story, expected_error, before, before_packet, after, after_packet) in cases {
        let offset = story.find("v-model=\"value\"").unwrap();
        let (line, column) =
            crate::batch::source_map::offset_to_line_col(&story, offset as u32).unwrap();
        let model_row = |row: &&packets::Row| {
            row.0 == STORY_PATH
                && row.1 == line
                && row.2 == column
                && row.3 == Some(2322)
                && row
                    .5
                    .contains("Type 'string[]' is not assignable to type 'number[]'.")
        };
        if scope == "minimal" {
            assert_eq!(after.len(), usize::from(expected_error), "{after_packet}");
            assert_eq!(
                before.len(),
                usize::from(expected_error || name == "original-text"),
                "{before_packet}"
            );
        }
        if name == "original-text" {
            assert_eq!(
                before.iter().filter(model_row).count(),
                1,
                "{before_packet}"
            );
            let preserved: Vec<_> = before
                .iter()
                .filter(|row| !model_row(row))
                .cloned()
                .collect();
            assert_eq!(after, preserved, "{after_packet}");
        } else {
            assert_eq!(after, before, "{after_packet}");
        }
        assert_eq!(
            after.iter().filter(model_row).count(),
            usize::from(expected_error),
            "{after_packet}"
        );
    }
}
