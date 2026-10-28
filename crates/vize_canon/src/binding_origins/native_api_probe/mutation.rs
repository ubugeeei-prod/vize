//! Test-only requested-declaration invalidation on the existing retained session.
use super::{coverage, fixture, hash, write};
use corsa::{
    api::{FileChanges, ProjectSession, SymbolResponse},
    runtime::block_on,
};
use serde_json::{Value, json};
use std::{fs, path::Path};
use vize_carton::String;

fn query(session: &ProjectSession, file: &Path, position: u32) -> SymbolResponse {
    block_on(session.get_symbol_at_position(file.to_string_lossy().as_ref(), position))
        .unwrap()
        .unwrap()
}

fn alias_target(session: &ProjectSession, selected: &SymbolResponse) -> SymbolResponse {
    if selected.flags & (1 << 21) == 0 {
        return selected.clone();
    }
    block_on(session.client().get_aliased_symbol(
        session.snapshot().handle.clone(),
        session.project().id.clone(),
        selected.id.clone(),
    ))
    .unwrap()
    .unwrap()
}

pub(super) fn run(
    session: &mut ProjectSession,
    provider: &Path,
    root: &Path,
    output: &Path,
    version: &str,
) -> Vec<Value> {
    let authored = root.join("custom.ts");
    let original = fs::read_to_string(&authored).unwrap();
    let old_initializer = "= custom;\n";
    assert_eq!(original.matches(old_initializer).count(), 1);
    let baseline: Vec<Value> = serde_json::from_slice(
        &fs::read(output.join("repaired/getSemanticDiagnostics.json")).unwrap(),
    )
    .unwrap();
    let baseline_diagnostics = baseline
        .iter()
        .flat_map(|packet| packet["response"].as_array().unwrap().iter().cloned())
        .collect::<Vec<_>>();
    assert_eq!(baseline_diagnostics.len(), 2);
    let probe = root.join("probe.ts");
    let probe_bytes = fs::read(&probe).unwrap();
    let config_bytes = fs::read(root.join("tsconfig.json")).unwrap();
    let initial_snapshot = session.snapshot().handle.clone();
    let project = serde_json::to_value(session.project()).unwrap();
    let mut position = None;
    let mut rows = Vec::new();
    for (label, initializer) in [
        ("initializer-vue", "actual"),
        ("initializer-custom", "custom"),
        ("initializer-repaired", "actual"),
    ] {
        let current = original.replace(old_initializer, &format!("= {initializer};\n"));
        assert_eq!(current.len(), original.len());
        let marker = format!("= {initializer};\n");
        let byte_position = current.rfind(&marker).unwrap() + "= ".len();
        let utf16_position =
            u32::try_from(current[..byte_position].encode_utf16().count()).unwrap();
        assert_eq!(
            current.get(byte_position..byte_position + initializer.len()),
            Some(initializer),
        );
        assert_eq!(*position.get_or_insert(utf16_position), utf16_position);
        fs::write(&authored, &current).unwrap();
        block_on(session.refresh(Some(FileChanges::InvalidateAll {
            invalidate_all: true,
        })))
        .unwrap();
        let lease = session.snapshot_lease();
        let snapshot = lease.handle.clone();
        assert_ne!(snapshot, initial_snapshot);
        assert_eq!(serde_json::to_value(session.project()).unwrap(), project);
        let directory = output.join(label);
        fs::create_dir(&directory).unwrap();
        fs::write(directory.join("authored-custom.ts"), &current).unwrap();
        let selected = query(session, &probe, fixture::position("annotated", "annotated"));
        assert_ne!(selected.flags & (1 << 21), 0);
        let declaration = alias_target(session, &selected);
        assert_eq!(coverage::declaration_path(&declaration), authored);
        let apparent_type = block_on(session.client().get_type_of_symbol(
            session.snapshot().handle.clone(),
            session.project().id.clone(),
            declaration.id.clone(),
        ))
        .unwrap()
        .unwrap();
        let type_owner = block_on(session.client().get_symbol_of_type_in_project(
            session.snapshot().handle.clone(),
            session.project().id.clone(),
            apparent_type.id.clone(),
        ))
        .unwrap()
        .unwrap();
        let type_provider = coverage::package(&type_owner, provider);
        assert_eq!(type_provider["manifest"]["version"], "3.5.42");
        let initializer_symbol = query(session, &authored, utf16_position);
        let initializer_target = alias_target(session, &initializer_symbol);
        let repeated = query(session, &authored, utf16_position);
        assert_eq!(initializer_symbol, repeated);
        let expected_custom = initializer == "custom";
        if expected_custom {
            assert_eq!(initializer_symbol.flags & (1 << 21), 0);
            assert_eq!(coverage::declaration_path(&initializer_target), authored);
            assert_ne!(initializer_target.id, type_owner.id);
        } else {
            assert_ne!(initializer_symbol.flags & (1 << 21), 0);
            assert_eq!(initializer_target.id, type_owner.id);
            assert_eq!(
                coverage::package(&initializer_target, provider),
                type_provider,
            );
        }
        let names = coverage::sources(session, &directory);
        let syntax = coverage::diagnostics(
            session,
            version,
            "getSyntacticDiagnostics",
            &names,
            &directory,
        );
        assert!(syntax.is_empty());
        let semantic = coverage::diagnostics(
            session,
            version,
            "getSemanticDiagnostics",
            &[
                String::from(probe.to_string_lossy().as_ref()),
                String::from(authored.to_string_lossy().as_ref()),
            ],
            &directory,
        );
        assert_eq!(semantic, baseline_diagnostics);
        assert_eq!(fs::read(&probe).unwrap(), probe_bytes);
        assert_eq!(fs::read(root.join("tsconfig.json")).unwrap(), config_bytes);
        assert_eq!(session.snapshot().handle, snapshot);
        let row = json!({
            "label":label,"snapshot":snapshot,"project":session.project(),
            "authoredPath":authored,"authoredBytes":current.len(),
            "authoredSha256":hash(current.as_bytes()),
            "initializerUtf16Position":utf16_position,
            "probePath":probe,"probeUtf16Position":fixture::position("annotated", "annotated"),
            "selected":selected,"exportedDeclaration":declaration,
            "apparentType":apparent_type,"typeOwner":type_owner,
            "typeProvider":type_provider,"initializer":initializer,
            "typeOrigin":{"evidence":"current-type-symbol","owner":type_owner,"provider":type_provider},
            "valueOrigin":{"evidence":"current-authored-initializer-binding",
                "selected":initializer_symbol,"target":initializer_target,
                "fixtureExpected":if expected_custom {"custom-function"} else {"vue-import"},
                "runtimeProducerAcknowledged":false},
            "initializerSelected":initializer_symbol,"initializerTarget":initializer_target,
            "sameSnapshotRepeatedInitializer":repeated,
            "fixtureValueOrigin":if expected_custom {"custom-function"} else {"vue-import"},
            "typeOwnerIsVue":true,"runtimeProducerAcknowledged":false,
            "sourceInputManifestEntries":names.len(),
            "semanticDiagnostics":semantic,"syntaxDiagnostics":syntax,
            "nativeProgramCount":Value::Null,"nativeBindCount":Value::Null,
            "originClassifierQualified":false,"hostWire":false,"passed":true
        });
        write(&directory.join("mutation-receipt.json"), &row);
        rows.push(row);
        drop(lease);
    }
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0]["authoredSha256"], rows[2]["authoredSha256"]);
    assert_ne!(rows[0]["authoredSha256"], rows[1]["authoredSha256"]);
    rows
}
