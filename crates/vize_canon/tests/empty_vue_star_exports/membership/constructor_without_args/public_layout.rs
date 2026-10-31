//! Authenticate the current physical package before inverting its finite layout.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "malformed-membership/public-declaration-layout-update-20261031.json";
const FROZEN: &str = "d46c4c1fc777809b9a82c3917020ac14f823f4ad894777043c54a2d55c342a02";
const HISTORICAL: &str = "96772117773d2c9dc53ddd553d2544956bba5ca0c97e96090157657b3159fbde";
const PATHS: [&str; 9] = [
    "npm/cli/virtual-ts.d.ts",
    "npm/cli/virtual-ts.d.mts",
    "npm/cli/virtual-ts.d.cts",
    "npm/cli/virtual-ts-globals.d.ts",
    "npm/cli/virtual-ts-patterns.d.ts",
    "npm/cli/content-mapper-helpers.d.ts",
    "npm/cli/content-mapper-helpers.d.mts",
    "npm/cli/content-mapper-helpers.d.cts",
    "npm/cli/generated/content-mapper-types.d.ts",
];
const WIRING: [&str; 2] = [
    "import type {} from \"./virtual-ts-globals.js\" with { \"resolution-mode\": \"import\" };",
    "import type {} from \"./virtual-ts-patterns.js\" with { \"resolution-mode\": \"import\" };",
];

fn metadata(update: &Value) {
    assert_eq!(update["version"], 1);
    assert_eq!(update["kind"], "exact-public-declaration-layout-inverse");
    assert_eq!(
        update["sourceRevision"],
        "89b3bd1594ba2012b1c3fabd9cd0535d943a2b71"
    );
    assert_eq!(
        update["beforeSourceRevision"],
        "700f41c6171ae0eb6eb5fd7407eeb295b5eea64a"
    );
    assert_eq!(update["nativeAcceptanceCredit"], false);
    assert_eq!(update["physicalFilesRole"], "actual-current-public-package");
    assert_eq!(update["reconstructedEntriesArePhysical"], false);
    assert_eq!(update["frozenConstructorRecordSha256"], FROZEN);
    assert_eq!(update["wiring"], json!([[0, WIRING[0]], [1, WIRING[1]]]));
    let files = update["files"].as_array().unwrap();
    assert_eq!(files.len(), PATHS.len());
    for (file, path) in files.iter().zip(PATHS) {
        assert_eq!(file["path"], path);
        let body = file["wholeUtf8"].as_str().unwrap();
        assert_eq!(file["bytes"], json!(body.len()));
        assert_eq!(file["sha256"], support::hash(body.as_bytes()));
    }
    let entries = update["reconstruction"].as_array().unwrap();
    assert_eq!(entries.len(), 3);
    for (entry, path) in entries.iter().zip(&PATHS[..3]) {
        assert_eq!(entry["path"], *path);
        assert_eq!(entry["beforeBytes"], 47_907);
        assert_eq!(entry["beforeSha256"], HISTORICAL);
        let chunks = entry["chunks"].as_array().unwrap();
        assert_eq!(chunks.len(), 4);
        for (chunk, source) in chunks.iter().zip([*path, PATHS[3], PATHS[4], *path]) {
            assert_eq!(chunk["path"], source);
        }
    }
}

fn load() -> Value {
    let bytes = fs::read(support::corpus().join(RECORD)).unwrap();
    assert_eq!(
        support::hash(&bytes),
        "fa2f6b2369276268c1c4be5ccdc68e5053a732166e8ad42198e77a2daccd7cdc"
    );
    let update = serde_json::from_slice(&bytes).unwrap();
    metadata(&update);
    update
}

fn physical() -> Vec<String> {
    PATHS
        .iter()
        .map(|path| fs::read_to_string(support::repository().join(path)).unwrap())
        .collect()
}

fn authenticate(bodies: &[String], update: &Value) {
    metadata(update);
    assert_eq!(bodies.len(), PATHS.len());
    for (body, file) in bodies.iter().zip(update["files"].as_array().unwrap()) {
        assert_eq!(json!(body.len()), file["bytes"]);
        assert_eq!(
            support::hash(body.as_bytes()),
            file["sha256"].as_str().unwrap()
        );
        assert_eq!(body, file["wholeUtf8"].as_str().unwrap());
    }
    for entry in &bodies[..3] {
        let wiring = entry
            .lines()
            .enumerate()
            .filter(|(_, line)| {
                let line = line.trim_start();
                line.starts_with("import ") || line.starts_with("/// <reference ")
            })
            .collect::<Vec<_>>();
        assert_eq!(wiring, [(0, WIRING[0]), (1, WIRING[1])]);
    }
}

fn reconstruct(path: &str, bodies: &[String], update: &Value, frozen: &Value) -> String {
    authenticate(bodies, update);
    let index = PATHS[..3]
        .iter()
        .position(|candidate| *candidate == path)
        .unwrap();
    assert_eq!(frozen["path"], path);
    assert_eq!(frozen["afterBytes"], 47_907);
    assert_eq!(frozen["afterSha256"], HISTORICAL);
    let entry = &update["reconstruction"][index];
    let mut historical = String::new();
    for chunk in entry["chunks"].as_array().unwrap() {
        let source = PATHS
            .iter()
            .position(|path| chunk["path"] == *path)
            .unwrap();
        let start = usize::try_from(chunk["start"].as_u64().unwrap()).unwrap();
        let bytes = usize::try_from(chunk["bytes"].as_u64().unwrap()).unwrap();
        let part = bodies[source]
            .get(start..start.checked_add(bytes).unwrap())
            .unwrap();
        assert_eq!(
            support::hash(part.as_bytes()),
            chunk["sha256"].as_str().unwrap()
        );
        historical.push_str(part);
    }
    assert_eq!(json!(historical.len()), entry["beforeBytes"]);
    assert_eq!(support::hash(historical.as_bytes()), HISTORICAL);
    assert_eq!(historical, frozen["afterWholeUtf8"].as_str().unwrap());
    historical
}

pub(super) fn read(path: &str, frozen: &Value) -> String {
    reconstruct(path, &physical(), &load(), frozen)
}

pub(super) fn retain_provenance(manifest: &mut Value) {
    let update = load();
    authenticate(&physical(), &update);
    manifest["provenance"]["currentPublicDeclarationLayoutUpdate"] = update;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frozen() -> Value {
        let bytes = fs::read(
            support::corpus()
                .join("malformed-membership/constructor-without-args-update-20261031.json"),
        )
        .unwrap();
        assert_eq!(support::hash(&bytes), FROZEN);
        serde_json::from_slice(&bytes).unwrap()
    }

    #[test]
    fn physical_layout_reconstructs_frozen_entries_before_alias_history() {
        let update = load();
        let bodies = physical();
        let frozen = frozen();
        for (index, path) in PATHS[..3].iter().enumerate() {
            let record = &frozen["declarations"][index + 2];
            let historical = reconstruct(path, &bodies, &update, record);
            assert_eq!(historical, record["afterWholeUtf8"].as_str().unwrap());
            assert_eq!(
                super::super::previous(&historical, record),
                record["beforeWholeUtf8"].as_str().unwrap()
            );
        }
    }

    #[test]
    fn every_physical_body_and_entry_reference_is_authenticated() {
        let update = load();
        let bodies = physical();
        let frozen = frozen();
        let record = &frozen["declarations"][2];
        for (index, body) in bodies.iter().enumerate() {
            for changed in [
                format!("{body}\n"),
                body.replacen('\n', "", 1),
                format!("{body}{body}"),
                body.replacen("type ", "interface ", 1),
            ] {
                assert_ne!(&changed, body);
                let mut mutated = bodies.clone();
                mutated[index] = changed;
                assert!(
                    std::panic::catch_unwind(|| reconstruct(PATHS[0], &mutated, &update, record))
                        .is_err()
                );
            }
        }
        let ordered = WIRING.join("\n");
        let reversed = WIRING.into_iter().rev().collect::<Vec<_>>().join("\n");
        for index in 0..3 {
            for changed in [
                bodies[index].replacen(ordered.as_str(), reversed.as_str(), 1),
                bodies[index].replacen(WIRING[0], "", 1),
                format!("{}\n{}", WIRING[0], bodies[index]),
                bodies[index].replacen("virtual-ts-globals.js", "unreviewed.js", 1),
            ] {
                let mut mutated = bodies.clone();
                mutated[index] = changed;
                assert!(
                    std::panic::catch_unwind(|| reconstruct(PATHS[0], &mutated, &update, record))
                        .is_err()
                );
            }
        }
    }

    #[test]
    fn layout_metadata_chunk_order_and_unknown_inputs_are_refused() {
        let update = load();
        let bodies = physical();
        let frozen = frozen();
        let record = &frozen["declarations"][2];
        for field in [
            "sourceRevision",
            "beforeSourceRevision",
            "frozenConstructorRecordSha256",
            "physicalFilesRole",
        ] {
            let mut changed = update.clone();
            changed[field] = json!("unknown");
            assert!(
                std::panic::catch_unwind(|| reconstruct(PATHS[0], &bodies, &changed, record))
                    .is_err()
            );
        }
        for field in ["nativeAcceptanceCredit", "reconstructedEntriesArePhysical"] {
            let mut changed = update.clone();
            changed[field] = json!(true);
            assert!(
                std::panic::catch_unwind(|| reconstruct(PATHS[0], &bodies, &changed, record))
                    .is_err()
            );
        }
        let mut changed = update.clone();
        changed["files"].as_array_mut().unwrap().swap(0, 1);
        assert!(
            std::panic::catch_unwind(|| reconstruct(PATHS[0], &bodies, &changed, record)).is_err()
        );
        for mutation in 0..6 {
            let mut changed = update.clone();
            let chunks = changed["reconstruction"][0]["chunks"]
                .as_array_mut()
                .unwrap();
            match mutation {
                0 => chunks.swap(0, 1),
                1 => {
                    chunks.pop();
                }
                2 => chunks.push(chunks[0].clone()),
                3 => chunks[0]["start"] = json!(104),
                4 => chunks[0]["sha256"] = json!("0".repeat(64)),
                _ => chunks[0]["path"] = json!("npm/cli/unknown.d.ts"),
            }
            assert!(
                std::panic::catch_unwind(|| reconstruct(PATHS[0], &bodies, &changed, record))
                    .is_err()
            );
        }
        assert!(
            std::panic::catch_unwind(|| reconstruct(
                "npm/cli/unknown.d.ts",
                &bodies,
                &update,
                record
            ))
            .is_err()
        );
    }
}
