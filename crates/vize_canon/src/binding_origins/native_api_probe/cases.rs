//! Independent names expose type identity versus declaration/value provenance.
use super::{coverage, fixture};
use super::{hash, write};
use corsa::{
    api::{ProjectSession, SymbolResponse},
    runtime::block_on,
};
use serde_json::{Value, json};
use std::{fs, path::Path};
use vize_carton::{FxHashMap, String};

fn query(session: &ProjectSession, file: &Path, position: u32) -> Option<SymbolResponse> {
    block_on(session.get_symbol_at_position(file.to_string_lossy().as_ref(), position)).unwrap()
}
fn target(session: &ProjectSession, selected: &SymbolResponse) -> SymbolResponse {
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
    session: &ProjectSession,
    provider: &Path,
    root: &Path,
    output: &Path,
    version: &str,
    partial: bool,
) -> Value {
    fs::create_dir(output).unwrap();
    fs::copy(root.join("partial.ts"), output.join("original-partial.ts")).unwrap();
    let file = root.join("probe.ts");
    let position = fixture::actual_position();
    let actual = query(session, &file, position).unwrap();
    assert_ne!(actual.flags & (1 << 21), 0);
    let real = target(session, &actual);
    let repeated = query(session, &file, position).unwrap();
    assert_eq!(actual, repeated);
    let normalized = block_on(session.client().raw_json_request(
        "getSymbolAtPosition",
        json!({"snapshot":session.snapshot().handle,"project":session.project().id,"file":file,"position":position}),
    )).unwrap();
    write(
        &output.join("baseline-normalized.json"),
        &json!({
            "request":{"file":file,"position":position},"selected":actual,"target":real,"repeat":repeated,"normalized":normalized
        }),
    );
    let real_package = coverage::package(&real, provider);
    assert_eq!(
        real_package
            .get("manifest")
            .unwrap()
            .get("version")
            .and_then(Value::as_str),
        Some("3.5.42")
    );
    let mut selected = FxHashMap::default();
    let mut targets = FxHashMap::default();
    let mut rows = Vec::new();
    for (label, token) in fixture::CASES {
        let position = fixture::position(label, token);
        let value = query(session, &file, position);
        if label == "missing" {
            assert!(
                value.is_none(),
                "missing original global must not acquire a binding"
            );
            rows.push(json!({"label":label,"position":position,"selected":Value::Null}));
            continue;
        }
        let value = value.unwrap();
        let resolved = target(session, &value);
        rows.push(json!({"label":label,"position":position,"selected":value,"target":resolved}));
        selected.insert(label, value);
        targets.insert(label, resolved);
    }
    write(
        &output.join("cases-selected.json"),
        &json!({"cases":rows,"real":real}),
    );
    let get = |name| targets.get(name).unwrap();
    assert_eq!(get("namespace").id, real.id);
    assert_eq!(
        get("type-only").id,
        real.id,
        "type target alone must not authorize a runtime value"
    );
    assert_ne!(get("other").id, real.id);
    let other_package = coverage::package(get("other"), provider);
    assert_eq!(
        other_package
            .get("manifest")
            .unwrap()
            .get("version")
            .and_then(Value::as_str),
        Some("3.5.44")
    );
    assert_eq!(
        coverage::declaration_path(get("trapped")),
        root.join("custom-vue.ts")
    );
    assert_eq!(
        coverage::declaration_path(get("annotated")),
        root.join("custom.ts")
    );
    for name in [
        "trapped",
        "annotated",
        "value-alias",
        "ambient-vue",
        "ambient-shape",
        "merged",
        "class",
        "enum",
        "namespace-global",
        "destructure",
        "shadow",
    ] {
        assert_ne!(
            get(name).id,
            real.id,
            "{name} is a distinct declaration/value binding"
        );
    }
    assert_ne!(
        selected.get("shadow").unwrap().id,
        actual.id,
        "same spelling cannot inherit the import's identity"
    );
    let merged = get("merged");
    assert_eq!(
        merged.declarations.len(),
        3,
        "all two overloads and namespace declarations must survive"
    );
    let mut paths = merged
        .declarations
        .iter()
        .map(|handle| {
            Path::new(handle.declaring_path().unwrap().as_str())
                .canonicalize()
                .unwrap()
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    assert_eq!(
        paths,
        vec![root.join("globals.d.ts"), root.join("merged.d.ts")]
    );
    let annotated_type = block_on(session.client().get_type_of_symbol(
        session.snapshot().handle.clone(),
        session.project().id.clone(),
        get("annotated").id.clone(),
    ))
    .unwrap()
    .unwrap();
    let annotated_type_owner = block_on(session.client().get_symbol_of_type_in_project(
        session.snapshot().handle.clone(),
        session.project().id.clone(),
        annotated_type.id.clone(),
    ))
    .unwrap()
    .unwrap();
    assert_eq!(
        annotated_type_owner.id, real.id,
        "a custom runtime initializer may still have Vue's annotated type owner"
    );
    write(
        &output.join("cases-normalized.json"),
        &json!({
            "cases":rows,"real":real,"realPackage":real_package,"otherPackage":other_package,
            "annotatedType":annotated_type,"annotatedTypeOwner":annotated_type_owner
        }),
    );
    let names = coverage::sources(session, output);
    let syntax = coverage::diagnostics(session, version, "getSyntacticDiagnostics", &names, output);
    assert_eq!(
        syntax.is_empty(),
        !partial,
        "partial-parse admission must be observed, never guessed"
    );
    if partial {
        assert!(syntax.iter().any(|row| {
            row.get("fileName")
                .and_then(Value::as_str)
                .is_some_and(|name| Path::new(name) == root.join("partial.ts"))
        }));
    }
    let semantic = coverage::diagnostics(
        session,
        version,
        "getSemanticDiagnostics",
        &[String::from(file.to_string_lossy().as_ref())],
        output,
    );
    let mut codes = semantic
        .iter()
        .map(|row| row.get("code").and_then(Value::as_i64).unwrap())
        .collect::<Vec<_>>();
    codes.sort();
    assert_eq!(
        codes,
        vec![1361, 2304],
        "missing global and type-only runtime uses stay diagnosed"
    );
    for row in &semantic {
        assert!(
            row.get("fileName")
                .and_then(Value::as_str)
                .is_some_and(|name| Path::new(name) == file)
        );
        let code = row.get("code").and_then(Value::as_i64).unwrap();
        let (label, token, message) = if code == 1361 {
            (
                "type-only",
                "TypeOnly",
                "'TypeOnly' cannot be used as a value because it was imported using 'import type'.",
            )
        } else {
            (
                "missing",
                "MissingRuntimeRef",
                "Cannot find name 'MissingRuntimeRef'.",
            )
        };
        let position = i64::from(fixture::position(label, token));
        assert_eq!(row.get("text").and_then(Value::as_str), Some(message));
        assert_eq!(row.get("pos").and_then(Value::as_i64), Some(position));
        assert_eq!(
            row.get("end").and_then(Value::as_i64),
            Some(position + i64::try_from(token.encode_utf16().count()).unwrap())
        );
    }
    let mut owners = vec![coverage::declaration_path(&real)];
    for value in targets.values() {
        owners.extend(value.declarations.iter().map(|handle| {
            Path::new(handle.declaring_path().unwrap().as_str())
                .canonicalize()
                .unwrap()
        }));
    }
    owners.sort();
    owners.dedup();
    let mut retained = Vec::new();
    for (index, owner) in owners.iter().enumerate() {
        assert!(
            names
                .iter()
                .any(|name| Path::new(name.as_str()).canonicalize().unwrap() == *owner)
        );
        let payload = block_on(session.client().get_source_file(
            session.snapshot().handle.clone(),
            session.project().id.clone(),
            owner.to_string_lossy().as_ref(),
        ))
        .unwrap()
        .unwrap();
        fs::write(
            output.join(format!("declaration-{index}.encoded.bin")),
            payload.as_bytes(),
        )
        .unwrap();
        retained.push(payload);
    }
    let bytes = retained
        .iter()
        .map(|payload| payload.as_bytes().len())
        .sum::<usize>();
    let receipt = json!({
        "snapshot":session.snapshot().handle,"project":session.project(),"cases":fixture::CASES.len(),
        "sourceInputManifestEntries":names.len(),"sourceSyntaxDiagnostics":syntax,
        "syntaxComplete":!partial,"semanticDiagnostics":semantic,
        "mergedDeclarationCount":merged.declarations.len(),"mergedDeclarationOwners":paths,
        "sameSpellingShadowDistinct":true,"typeOnlyStillHasRuntimeDiagnostic":true,
        "customTypedInitializerTypeOwnerEqualsVue":true,"customValueBindingIsDistinct":true,
        "selectedProviderPackages":[real_package,other_package],
        "retainedEncodedSourcePayloadBytes":bytes,
        "encodedPayloadSha256":retained.iter().map(|payload| hash(payload.as_bytes())).collect::<Vec<_>>(),
        "managedSnapshotLeaseBytes":Value::Null,"nativeProgramCount":Value::Null,"nativeBindCount":Value::Null,
        "originClassifierQualified":false,"hostWire":false,"passed":true
    });
    write(&output.join("phase-receipt.json"), &receipt);
    receipt
}
