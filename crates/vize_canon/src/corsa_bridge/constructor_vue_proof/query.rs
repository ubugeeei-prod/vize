//! The caller supplies one existing owner, never independent API handles.

use super::*;
use corsa::api::ApiClient;
use serde_json::json;

// Same alias-query gate used by the existing component-types query phase.
const ALIAS: u32 = 1 << 21;

pub(super) async fn observe(api: &ApiClient, data: &mut ObservationData) -> corsa::Result<()> {
    data.stage = "source-membership";
    data.source_names = Some(
        api.raw_json_request(
            "getSourceFileNames",
            json!({"snapshot":data.snapshot,"project":data.project.id}),
        )
        .await?,
    );
    let Some(names) = data.source_names.as_ref().and_then(Value::as_array) else {
        data.unavailable.push(Unavailable::SourceMembership);
        return Ok(());
    };
    if names.iter().any(|name| !name.is_string()) {
        data.unavailable.push(Unavailable::SourceMembership);
        return Ok(());
    }
    for expected in [
        data.request.generated.clone(),
        data.request.component_source.clone(),
    ] {
        data.stage = "usage-source";
        let Some(index) = source(api, data, &expected.file).await? else {
            return Ok(());
        };
        let Some(decoded) = data.sources[index].decoded.as_ref() else {
            data.unavailable
                .push(Unavailable::IncompleteSourceRecord(expected.file));
            return Ok(());
        };
        if let Some(reason) = source_matches(&expected, decoded) {
            data.unavailable.push(reason);
            return Ok(());
        }
    }
    if !anchor_matches(
        &data.request.generated.text,
        &data.request.component_reference,
    ) || data
        .request
        .hosts
        .iter()
        .any(|anchor| !anchor_matches(&data.request.generated.text, anchor))
        || !anchor_matches(
            &data.request.component_source.text,
            &data.request.vue_module,
        )
    {
        data.unavailable
            .push(Unavailable::Anchor(data.request.usage_id.clone()));
        return Ok(());
    }

    data.stage = "component-type";
    data.constructor_type = api
        .get_type_at_position(
            data.snapshot.clone(),
            data.project.id.clone(),
            data.request.generated.file.as_str(),
            data.request.component_reference.position,
        )
        .await?;
    if let Some(component) = &data.constructor_type {
        data.stage = "declared-construct-signatures";
        // Same existing endpoint/parameters as the typed SDK method. The raw
        // route retains DerivedSnapshots ID adoption before local DTO decode.
        let raw = api
            .raw_json_request(
                "getSignaturesOfType",
                json!({
                    "snapshot":data.snapshot,"project":data.project.id,
                    "type":component.id,"kind":1
                }),
            )
            .await?;
        data.stage = "decode-construct-signatures";
        let signatures = retain_signatures(data, raw)?;
        for signature in signatures {
            let index = data.declared_returns.len();
            data.declared_returns.push(SignatureReturn {
                signature,
                returned: None,
                symbol: None,
                type_parameters_raw: SignatureTypeParametersRaw::NotRequested,
                type_parameters: None,
            });
            let record = &mut data.declared_returns[index];
            data.stage = "declared-signature-return";
            record.returned = api
                .get_return_type_of_signature(
                    data.snapshot.clone(),
                    data.project.id.clone(),
                    record.signature.id.clone(),
                )
                .await?;
            if let Some(returned) = &record.returned {
                data.stage = "declared-return-symbol";
                record.symbol = api
                    .get_symbol_of_type_in_project(
                        data.snapshot.clone(),
                        data.project.id.clone(),
                        returned.id.clone(),
                    )
                    .await?;
            }
        }
    } else {
        missing(data, "component-type");
    }

    for anchor in data.request.hosts.clone() {
        let index = data.hosts.len();
        data.hosts.push(HostObservation {
            anchor,
            pair: None,
            with_arguments: None,
            instance: None,
            instance_symbol: None,
            props: None,
            props_type: None,
        });
        let host = &mut data.hosts[index];
        data.stage = "actual-host-type";
        host.pair = api
            .get_type_at_position(
                data.snapshot.clone(),
                data.project.id.clone(),
                data.request.generated.file.as_str(),
                host.anchor.position,
            )
            .await?;
        let Some(pair) = &host.pair else {
            missing(data, "actual-host-type");
            continue;
        };
        data.stage = "actual-with-arguments-property";
        host.with_arguments = api
            .get_property_of_type(
                data.snapshot.clone(),
                data.project.id.clone(),
                pair.id.clone(),
                "withArguments",
            )
            .await?;
        let Some(property) = &host.with_arguments else {
            missing(data, "withArguments");
            continue;
        };
        data.stage = "actual-returned-instance";
        host.instance = api
            .get_type_of_symbol(
                data.snapshot.clone(),
                data.project.id.clone(),
                property.id.clone(),
            )
            .await?;
        let Some(instance) = &host.instance else {
            missing(data, "actual-returned-instance");
            continue;
        };
        data.stage = "actual-returned-instance-symbol";
        host.instance_symbol = api
            .get_symbol_of_type_in_project(
                data.snapshot.clone(),
                data.project.id.clone(),
                instance.id.clone(),
            )
            .await?;
        data.stage = "actual-public-props";
        host.props = api
            .get_property_of_type(
                data.snapshot.clone(),
                data.project.id.clone(),
                instance.id.clone(),
                "$props",
            )
            .await?;
        if let Some(props) = &host.props {
            data.stage = "actual-public-props-type";
            host.props_type = api
                .get_type_of_symbol(
                    data.snapshot.clone(),
                    data.project.id.clone(),
                    props.id.clone(),
                )
                .await?;
        } else {
            missing(data, "actual-public-props");
        }
    }

    data.stage = "selected-vue-module";
    data.vue.module = api
        .get_symbol_at_position(
            data.snapshot.clone(),
            data.project.id.clone(),
            data.request.component_source.file.as_str(),
            data.request.vue_module.position,
        )
        .await?;
    if let Some(module) = &data.vue.module {
        let module_id = if module.flags & ALIAS != 0 {
            data.stage = "selected-vue-module-alias";
            data.vue.module_alias = api
                .get_aliased_symbol(
                    data.snapshot.clone(),
                    data.project.id.clone(),
                    module.id.clone(),
                )
                .await?;
            data.vue.module_alias.as_ref().map(|value| value.id.clone())
        } else {
            Some(module.id.clone())
        };
        if let Some(module_id) = module_id {
            data.stage = "selected-vue-exports";
            let exports = api
                .get_exports_of_module(data.snapshot.clone(), data.project.id.clone(), module_id)
                .await?;
            let selected: Vec<_> = exports
                .iter()
                .filter(|symbol| symbol.name == "ComponentPublicInstance")
                .cloned()
                .collect();
            data.vue.exports = Some(exports);
            if selected.len() == 1 {
                data.vue.public_instance = selected.into_iter().next();
            } else {
                data.unavailable
                    .push(Unavailable::ExportCardinality(selected.len()));
            }
        } else {
            missing(data, "selected-vue-module-alias");
        }
    } else {
        missing(data, "selected-vue-module");
    }
    if let Some(instance) = &data.vue.public_instance {
        let id = if instance.flags & ALIAS != 0 {
            data.stage = "selected-vue-public-instance-alias";
            data.vue.public_instance_alias = api
                .get_aliased_symbol(
                    data.snapshot.clone(),
                    data.project.id.clone(),
                    instance.id.clone(),
                )
                .await?;
            data.vue
                .public_instance_alias
                .as_ref()
                .map(|value| value.id.clone())
        } else {
            Some(instance.id.clone())
        };
        if let Some(id) = id {
            data.stage = "selected-vue-declared-instance";
            data.vue.declared_instance = api
                .get_declared_type_of_symbol(data.snapshot.clone(), data.project.id.clone(), id)
                .await?;
            if let Some(instance) = &data.vue.declared_instance {
                data.stage = "selected-vue-public-props";
                data.vue.props = api
                    .get_property_of_type(
                        data.snapshot.clone(),
                        data.project.id.clone(),
                        instance.id.clone(),
                        "$props",
                    )
                    .await?;
                if let Some(props) = &data.vue.props {
                    data.stage = "selected-vue-public-props-type";
                    data.vue.props_type = api
                        .get_type_of_symbol(
                            data.snapshot.clone(),
                            data.project.id.clone(),
                            props.id.clone(),
                        )
                        .await?;
                } else {
                    missing(data, "selected-vue-public-props");
                }
            } else {
                missing(data, "selected-vue-declared-instance");
            }
        } else {
            missing(data, "selected-vue-public-instance-alias");
        }
    }

    let symbols = data
        .hosts
        .iter()
        .flat_map(|host| {
            [
                host.with_arguments.clone(),
                host.instance_symbol.clone(),
                host.props.clone(),
            ]
        })
        .chain([
            data.vue.module.clone(),
            data.vue.module_alias.clone(),
            data.vue.public_instance.clone(),
            data.vue.public_instance_alias.clone(),
            data.vue.props.clone(),
        ])
        .chain(
            data.declared_returns
                .iter()
                .map(|record| record.symbol.clone()),
        )
        .flatten()
        .collect::<Vec<_>>();
    let mut declarations = Vec::new();
    for symbol in symbols {
        if symbol.declarations.is_empty() {
            data.unavailable
                .push(Unavailable::DeclarationMetadata(symbol.name.into()));
        }
        declarations.extend(symbol.declarations);
        declarations.extend(symbol.value_declaration);
    }
    declarations.extend(
        data.declared_returns
            .iter()
            .filter_map(|record| record.signature.declaration.clone()),
    );
    for declaration in declarations {
        let parsed = match declaration.parse() {
            Ok(parsed) => Some(parsed),
            Err(_) => {
                data.unavailable
                    .push(Unavailable::DeclarationRangeUnavailable(
                        declaration.clone(),
                    ));
                None
            }
        };
        // A compact handle can name a source without supplying node offsets.
        // Retain its body on this owner without inventing a native call range.
        let path: Option<String> = parsed
            .as_ref()
            .map(|value| value.path.as_str().into())
            .or_else(|| {
                declaration
                    .declaring_path()
                    .map(|path| path.as_str().into())
            });
        let Some(path) = path else {
            data.unavailable.push(Unavailable::DeclarationMetadata(
                declaration.as_str().into(),
            ));
            continue;
        };
        data.stage = "declaration-source";
        let Some(index) = source(api, data, path.as_str()).await? else {
            continue;
        };
        let Some(decoded) = data.sources[index].decoded.as_ref() else {
            data.unavailable
                .push(Unavailable::IncompleteSourceRecord(path));
            continue;
        };
        if decoded.file_name != path.as_str()
            || decoded.path != path.as_str()
            || parsed.as_ref().is_some_and(|parsed| {
                utf16_byte(&decoded.text, parsed.pos).is_none()
                    || utf16_byte(&decoded.text, parsed.end).is_none()
            })
        {
            data.unavailable.push(Unavailable::SourceIdentity(path));
        }
    }
    // Append to the old observation prefix: no existing query changes order.
    observe_signature_type_parameters(api, data).await?;
    data.stage = "observed-not-admitted";
    Ok(())
}

async fn observe_signature_type_parameters(
    api: &ApiClient,
    data: &mut ObservationData,
) -> corsa::Result<()> {
    for record in &mut data.declared_returns {
        data.stage = "declared-signature-type-parameters";
        record.type_parameters_raw = SignatureTypeParametersRaw::Requested;
        // Exact existing SDK SignatureOnlyRequest route and owner handles.
        // The raw route preserves null before the typed wrapper defaults it.
        let response = api
            .raw_json_request(
                "getTypeParametersOfSignature",
                json!({
                    "snapshot":data.snapshot,"project":data.project.id,
                    "signature":record.signature.id
                }),
            )
            .await;
        if response.is_ok() {
            data.stage = "decode-signature-type-parameters";
        }
        retain_signature_type_parameters(record, response)?;
    }
    Ok(())
}

fn missing(data: &mut ObservationData, label: &str) {
    data.unavailable
        .push(Unavailable::MissingQueryValue(label.into()));
}

async fn source(
    api: &ApiClient,
    data: &mut ObservationData,
    file: &str,
) -> corsa::Result<Option<usize>> {
    if !source_member(data.source_names.as_ref(), file) {
        data.unavailable
            .push(Unavailable::SourceIdentity(file.into()));
        return Ok(None);
    }
    if let Some(index) = data
        .sources
        .iter()
        .position(|source| source.requested == file)
    {
        return Ok(data.sources[index].decoded.as_ref().map(|_| index));
    }
    data.pending_source = Some(file.into());
    let payload = api
        .get_source_file(data.snapshot.clone(), data.project.id.clone(), file)
        .await?;
    data.sources.push(NativeSource {
        requested: file.into(),
        encoded: payload.as_ref().map(|value| value.as_bytes().to_vec()),
        decoded: None,
    });
    let index = data.sources.len() - 1;
    let Some(payload) = payload else {
        data.unavailable
            .push(Unavailable::AbsentSource(file.into()));
        return Ok(None);
    };
    data.sources[index].decoded = Some(payload.decode_source_file()?);
    data.pending_source = None;
    Ok(Some(index))
}
