//! All handles and responses in this probe belong to one leased project snapshot.
use corsa::{
    api::{
        ApiClient, DocumentIdentifier, ManagedSnapshot, NodeHandle, ProjectResponse,
        SymbolResponse, TypeResponse,
    },
    runtime::block_on,
};
use serde::Serialize;
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::Path};

fn record<T: Serialize>(
    calls: &mut Vec<Value>,
    label: &str,
    response: corsa::Result<T>,
) -> Result<T, String> {
    calls.push(
        json!({"label":label,"typedResponse":response.as_ref().map_err(|error|error.to_string())}),
    );
    response.map_err(|error| format!("{label}: {error}"))
}

fn required<T>(value: Option<T>, label: &str) -> Result<T, String> {
    value.ok_or_else(|| format!("{label}: no checker response"))
}

fn position(source: &str, needle: &str) -> Result<u32, String> {
    let matches = source.match_indices(needle).collect::<Vec<_>>();
    let [(offset, _)] = matches.as_slice() else {
        return Err(format!(
            "expected exactly one generated position for {needle}"
        ));
    };
    u32::try_from(source[..*offset].encode_utf16().count()).map_err(|error| error.to_string())
}

fn declarations(symbol: &SymbolResponse) -> Result<BTreeSet<NodeHandle>, String> {
    if symbol.declarations.is_empty() {
        return Err(format!("{}: no declaration evidence", symbol.name));
    }
    let values = symbol.declarations.iter().cloned().collect::<BTreeSet<_>>();
    if values.len() != symbol.declarations.len() {
        return Err(format!("{}: duplicate declaration evidence", symbol.name));
    }
    if symbol
        .value_declaration
        .as_ref()
        .is_some_and(|node| !values.contains(node))
    {
        return Err(format!(
            "{}: preferred declaration has another owner",
            symbol.name
        ));
    }
    Ok(values)
}

pub(super) fn observe(
    api: &ApiClient,
    snapshot: &ManagedSnapshot,
    project: &ProjectResponse,
    generated_path: &Path,
    generated: &str,
    calls: &mut Vec<Value>,
) -> Result<Value, String> {
    if snapshot
        .projects
        .iter()
        .filter(|value| *value == project)
        .count()
        != 1
    {
        return Err("project descriptor is not uniquely owned by the snapshot".into());
    }
    calls.push(
        json!({"snapshot":snapshot.handle,"project":project,"snapshotProjects":snapshot.projects}),
    );
    let source_names = record(
        calls,
        "getSourceFileNames",
        block_on(api.raw_json_request(
            "getSourceFileNames",
            json!({"snapshot":snapshot.handle,"project":project.id}),
        )),
    )?;
    let source_names: Vec<String> =
        serde_json::from_value(source_names).map_err(|error| error.to_string())?;
    let source_set = source_names.iter().collect::<BTreeSet<_>>();
    if source_set.len() != source_names.len()
        || source_names
            .iter()
            .any(|name| !Path::new(name).is_absolute())
    {
        return Err("invalid native source membership".into());
    }
    let file = generated_path
        .to_str()
        .ok_or("generated path is not UTF8")?;
    if !source_names.iter().any(|name| name == file) {
        return Err("generated file is absent from this native project".into());
    }
    let actual = source(api, snapshot, project, file, calls)?;
    if actual != generated {
        return Err("native snapshot does not own the exact generated VTS".into());
    }
    let mut hosts = Vec::new();
    for name in [
        "__vize_slot_host_9_instances",
        "__vize_slot_props_host_9_instances",
    ] {
        let offset = position(generated, &format!("const {name} ="))? + 6;
        let pair = required(
            record(
                calls,
                &format!("{name}.pairType"),
                block_on(api.get_type_at_position(
                    snapshot.handle.clone(),
                    project.id.clone(),
                    DocumentIdentifier::from(file),
                    offset,
                )),
            )?,
            "pair type",
        )?;
        let argument = property(api, snapshot, project, &pair, "withArguments", calls)?;
        let instance = required(
            record(
                calls,
                &format!("{name}.actualInstance"),
                block_on(api.get_type_of_symbol(
                    snapshot.handle.clone(),
                    project.id.clone(),
                    argument.id,
                )),
            )?,
            "actual instance",
        )?;
        let props = property(api, snapshot, project, &instance, "$props", calls)?;
        declarations(&props)?;
        hosts.push(props);
    }
    let first = declarations(&hosts[0])?;
    if declarations(&hosts[1])? != first {
        return Err("the two actual instance hosts disagree on props declarations".into());
    }
    // Follow the actual authored import binding, not a rewritten module spelling/path pattern.
    let component_import = record(
        calls,
        "importedComponentBinding",
        block_on(api.get_symbol_at_position(
            snapshot.handle.clone(),
            project.id.clone(),
            DocumentIdentifier::from(file),
            position(generated, "import WitnessChild")? + 7,
        )),
    )?;
    let component_import = required(component_import, "component import symbol")?;
    let component_import = required(
        record(
            calls,
            "importedComponentTarget",
            block_on(api.get_aliased_symbol(
                snapshot.handle.clone(),
                project.id.clone(),
                component_import.id,
            )),
        )?,
        "component import target",
    )?;
    let component_declarations = declarations(&component_import)?;
    let component_nodes = component_declarations.iter().collect::<Vec<_>>();
    let [component_declaration] = component_nodes.as_slice() else {
        return Err("component import target must have exactly one native declaration".into());
    };
    let component_path = component_declaration
        .declaring_path()
        .ok_or("component module lacks a declaring path")?;
    if !source_names
        .iter()
        .any(|name| name == component_path.as_str())
    {
        return Err("component module declaration is outside this native project".into());
    }
    let component = source(api, snapshot, project, component_path.as_str(), calls)?;
    let vue_position = position(&component, "'vue'")? + 1;
    let vue = required(
        record(
            calls,
            "resolvedVueModule",
            block_on(api.get_symbol_at_position(
                snapshot.handle.clone(),
                project.id.clone(),
                DocumentIdentifier::from(component_path.as_str()),
                vue_position,
            )),
        )?,
        "Vue module symbol",
    )?;
    let exports = record(
        calls,
        "vueModuleExports",
        block_on(api.get_exports_of_module(snapshot.handle.clone(), project.id.clone(), vue.id)),
    )?;
    let targets = exports
        .into_iter()
        .filter(|symbol| symbol.name == "ComponentPublicInstance")
        .collect::<Vec<_>>();
    let [target] = targets.as_slice() else {
        return Err("Vue export is absent or ambiguous".into());
    };
    let target_type = required(
        record(
            calls,
            "vuePublicInstanceType",
            block_on(api.get_declared_type_of_symbol(
                snapshot.handle.clone(),
                project.id.clone(),
                target.id.clone(),
            )),
        )?,
        "Vue public instance type",
    )?;
    let vue_props = property(api, snapshot, project, &target_type, "$props", calls)?;
    let vue_declarations = declarations(&vue_props)?;
    let mut declaration_sources = Vec::new();
    for node in first.union(&vue_declarations) {
        let path = node
            .declaring_path()
            .ok_or("props declaration has no source identity")?;
        if !source_names.iter().any(|name| name == path.as_str()) {
            return Err("props declaration is not owned by this native project".into());
        }
        let text = source(api, snapshot, project, path.as_str(), calls)?;
        declaration_sources.push(json!({"node":node,"path":path,"wholeSource":text,
            "parsedFullHandle":node.parse().ok().map(|value|json!({"pos":value.pos,"end":value.end,"kind":value.kind,"path":value.path}))}));
    }
    Ok(
        json!({"hostProps":hosts,"vueProps":vue_props,"bothHostsAgree":true,
        "sameDeclarationIdentity":first == vue_declarations,"declarationSources":declaration_sources,
        "flagInterpretation":"UNPROVED","metadataAvailabilityOnly":true,"propsAuthorityAdmitted":false}),
    )
}

fn property(
    api: &ApiClient,
    snapshot: &ManagedSnapshot,
    project: &ProjectResponse,
    value: &TypeResponse,
    name: &str,
    calls: &mut Vec<Value>,
) -> Result<SymbolResponse, String> {
    required(
        record(
            calls,
            &format!("property:{name}"),
            block_on(api.get_property_of_type(
                snapshot.handle.clone(),
                project.id.clone(),
                value.id.clone(),
                name,
            )),
        )?,
        name,
    )
}

fn source(
    api: &ApiClient,
    snapshot: &ManagedSnapshot,
    project: &ProjectResponse,
    name: &str,
    calls: &mut Vec<Value>,
) -> Result<String, String> {
    let response = block_on(api.get_source_file(snapshot.handle.clone(), project.id.clone(), name));
    calls.push(json!({"label":"getSourceFile","requestName":name,
        "outcome":response.as_ref().map(|value|value.is_some()).map_err(|error|error.to_string())}));
    let encoded = required(
        response.map_err(|error| error.to_string())?,
        "snapshot source",
    )?;
    calls.push(json!({"label":"sourceFilePayload","requestName":name,
        "encodedBytes":encoded.as_bytes()}));
    let decoded = encoded
        .decode_source_file()
        .map_err(|error| error.to_string())?;
    calls.push(json!({"label":"decodedSourceFile","requestName":name,
        "fileName":decoded.file_name,"path":decoded.path,"text":decoded.text,"originalText":decoded.original_text,
        "protocolVersion":decoded.protocol_version,"contentMapping":decoded.content_mapping}));
    if decoded.file_name != name || decoded.path != name {
        return Err("native source-file identity differs from requested member".into());
    }
    Ok(decoded.text)
}

#[test]
fn declaration_identity_refuses_empty_duplicate_or_foreign_preferred_nodes() {
    let mut symbol = SymbolResponse {
        id: "test".into(),
        name: "$props".into(),
        flags: 0,
        check_flags: 0,
        declarations: vec![],
        value_declaration: None,
    };
    assert_eq!(
        declarations(&symbol),
        Err("$props: no declaration evidence".into())
    );
    let node = NodeHandle::from("1.5.1./provider.d.ts");
    symbol.declarations = vec![node.clone(), node.clone()];
    assert_eq!(
        declarations(&symbol),
        Err("$props: duplicate declaration evidence".into())
    );
    symbol.declarations.pop();
    symbol.value_declaration = Some("2.6.1./other.d.ts".into());
    assert_eq!(
        declarations(&symbol),
        Err("$props: preferred declaration has another owner".into())
    );
    symbol.value_declaration = Some(node.clone());
    assert_eq!(declarations(&symbol), Ok(BTreeSet::from([node])));
}
