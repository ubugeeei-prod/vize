//! Independent complete literal projections and authenticated export identities.

use super::support;
use oxc_syntax::module_record::ExportExportName;
use serde_json::{Value, json};

pub(super) fn plain(case: &Value, generated: &Value) {
    for (file, expected) in case["projectedEquals"].as_object().unwrap() {
        assert_eq!(
            generated[file].as_str().unwrap(),
            support::body(expected),
            "{}: whole independent projected source for {file}",
            case["id"]
        );
    }
}

pub(super) fn exports(case: &Value, generated: &Value) -> Value {
    let mut observations = serde_json::Map::new();
    for (file, expected) in case["exportedNamesEquals"].as_object().unwrap() {
        let source = generated[file].as_str().unwrap();
        let arena = oxc_allocator::Allocator::default();
        let parsed = oxc_parser::Parser::new(&arena, source, oxc_span::SourceType::ts()).parse();
        assert!(!parsed.panicked, "the whole generated module must parse");
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let mut names = parsed
            .module_record
            .local_export_entries
            .iter()
            .chain(parsed.module_record.indirect_export_entries.iter())
            .map(|entry| match &entry.export_name {
                ExportExportName::Name(name) => name.name.to_string(),
                ExportExportName::Default(_) => "default".to_owned(),
                unknown => panic!("unproved exported identity: {unknown:?}"),
            })
            .collect::<Vec<_>>();
        names.sort();
        assert_eq!(
            json!(names),
            *expected,
            "complete exported identities for {file}"
        );
        assert!(
            parsed.module_record.star_export_entries.is_empty(),
            "exact public contracts cannot hide an unproved indirect star"
        );
        observations.insert(
            file.clone(),
            json!({"wholeSourceSHA256":support::hash(source.as_bytes()),
            "completeExportedNames":names,"wholeModuleRecord":format!("{:#?}",parsed.module_record),
            "wholeParserDiagnostics":format!("{:#?}",parsed.diagnostics)}),
        );
    }
    Value::Object(observations)
}

/// A historical raw oracle is valid only for byte-identical diagnostic files.
/// No generated position is converted into an invented authored position.
pub(super) fn raw_identity(case: &Value, generated: &Value, packet: &Value) -> Value {
    let rows = support::raw_rows(packet["wholeStdoutUtf8"].as_str().unwrap());
    let mut proof = Vec::new();
    for row in rows.as_array().unwrap() {
        let file = row["file"].as_str().unwrap();
        let native_input = support::body(&packet["inputs"][file]);
        assert_eq!(
            generated[file].as_str().unwrap(),
            native_input,
            "raw native coordinates require the complete historical projected body: {file}"
        );
        let input = case["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|input| input["file"] == file)
            .unwrap();
        let authored = support::body(input);
        let line = usize::try_from(row["line"].as_u64().unwrap() - 1).unwrap();
        assert_eq!(
            authored.lines().nth(line),
            native_input.lines().nth(line),
            "authored counterfactual coordinates require an identical diagnostic source line"
        );
        proof.push(json!({"file":file,"authoredSourceSHA256":input["sha256"],
            "wholeProjectedBodySHA256":packet["inputs"][file]["sha256"],
            "identicalDiagnosticLine":row["line"]}));
    }
    json!(proof)
}
