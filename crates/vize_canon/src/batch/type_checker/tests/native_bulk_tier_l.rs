//! Whole original Tier-L generations; separate from the integration timings.

use super::super::{BatchTypeChecker, BatchTypeCheckerOptions, TypeChecker};
use serde_json::{Value, json};
use std::{fs, path::Path};

#[path = "../../../../tests/support/tier_l_fixture.rs"]
mod fixture;
use fixture::{
    BROKEN_SOURCE, CLEAN_SOURCE, FIXTURE_ID, INJECTED_FILE, InjectedFixtureFile, TIER_L_VUE_FILES,
    collect_vue_paths, env_path, git_revision,
};

#[test]
#[ignore = "full original 500 SFC native law runs after Tier-L in Vue parity"]
fn native_bulk_original_500_sfc_whole_generations_match_original_lsp() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let registry: Value = serde_json::from_slice(
        &fs::read(repo.join("tests/_fixtures/vue-ecosystem-fixtures.json")).unwrap(),
    )
    .unwrap();
    let fixture = registry["projects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|project| project["id"] == FIXTURE_ID)
        .unwrap();
    let root = env_path("VIZE_TIER_L_FIXTURE", &repo)
        .unwrap_or_else(|| repo.join("tests/_fixtures/_git/vue-vben-admin"))
        .canonicalize()
        .unwrap();
    assert_eq!(
        git_revision(&root).as_str(),
        fixture["revision"].as_str().unwrap()
    );
    let native = env_path("VIZE_TIER_L_CORSA_BIN", &repo).unwrap();
    let path = root.join(INJECTED_FILE);
    let injected = InjectedFixtureFile::create(path.clone());
    let options = BatchTypeCheckerOptions {
        tsconfig_path: Some(root.join("playground/tsconfig.json")),
        ..Default::default()
    };
    let mut checker =
        BatchTypeChecker::with_options_and_corsa_path(&root, options, Some(&native)).unwrap();
    let paths = collect_vue_paths(&root);
    checker.scan_paths(&paths).unwrap();
    assert_eq!(paths.len(), TIER_L_VUE_FILES);
    assert_eq!(
        checker.file_count(),
        fixture["batchIncrementalBudget"]["maxRequestedFiles"]
            .as_u64()
            .unwrap() as usize
    );
    let capture = env_path("VIZE_TIER_L_BULK_CAPTURE_DIR", &repo).unwrap();
    let mut captured = Vec::new();
    for (phase, source) in [
        ("cold", CLEAN_SOURCE),
        ("brokenWarm", BROKEN_SOURCE),
        ("repairedWarm", CLEAN_SOURCE),
    ] {
        injected.write(source);
        let result = checker.check_incremental(std::slice::from_ref(&path));
        let mut packet = checker
            .executor
            .qualify_native_bulk_for_test(&checker.project);
        let metrics = checker.incremental_metrics();
        packet["metrics"] = json!({"sessionStarts":metrics.session_starts,"sessionToCliFallbacks":metrics.session_to_cli_fallbacks});
        fs::create_dir_all(&capture).unwrap();
        fs::write(
            capture.join(vize_l0::cstr!("{phase}.json").as_str()),
            serde_json::to_vec_pretty(&json!({
                "fixture":fixture,"injectedSource":source,"vuePaths":paths,
                "mappedResult":result.as_ref().ok().map(|result| vize_l0::cstr!("{result:#?}")),
                "mappedError":result.as_ref().err().map(|error| vize_l0::cstr!("{error:#?}")),
                "wholeGeneration":packet,
            }))
            .unwrap(),
        )
        .unwrap();
        captured.push((phase, result, packet));
    }
    for (phase, result, packet) in captured {
        result.unwrap();
        // Retain every side and custody before assertions, including refusal.
        assert_eq!(
            packet["observed"]["mode"], "native-bulk",
            "{phase}: {}",
            packet["observed"]
        );
        assert_eq!(packet["observed"]["bulk"]["release"], "acknowledged");
        assert_eq!(packet["metrics"]["sessionStarts"], 1);
        assert_eq!(packet["metrics"]["sessionToCliFallbacks"], 0);
        assert!(packet["actualError"].is_null());
        assert!(packet["originalError"].is_null());
        assert_eq!(
            packet["requestedUris"].as_array().unwrap().len(),
            checker.file_count()
        );
        assert!(
            packet["completeNative"]
                .as_array()
                .unwrap()
                .iter()
                .all(Value::is_array)
        );
        assert_eq!(
            packet["completeNative"], packet["completeOriginalLsp"],
            "{phase}: entire full diagnostic vectors must match"
        );
        assert_eq!(
            packet["actual"], packet["original"],
            "{phase}: entire public preprojection batches must match"
        );
    }
}
