use super::{CorsaTypeAwareSession, TypeProbe, errors::compact_error};
use corsa::{
    CorsaError,
    api::{DocumentIdentifier, ProjectSession, TypeProbeOptions, TypeResponse},
    fast::{CompactString as CorsaString, ToCompactString as _},
    runtime::block_on,
};
use vize_l0::{FxHashMap, String, profile};

const SNAPSHOT_REGISTRY_NOT_FOUND: &str = "not found in snapshot registry";

impl CorsaTypeAwareSession {
    /// Resolve all type probes for one virtual document with batched position
    /// queries. Output order matches `requests`.
    pub(in crate::linter) fn probe_types_at_offsets(
        &self,
        generated_source: &str,
        requests: &[(u32, bool, bool)],
    ) -> Result<Vec<Option<TypeProbe>>, String> {
        if requests.is_empty() {
            return Ok(Vec::new());
        }
        let generated_source = self.rewritten_source.as_deref().unwrap_or(generated_source);
        let positions = profile!("patina.corsa_session.byte_to_utf16_batch", {
            requests
                .iter()
                .map(|(generated_offset, _, _)| {
                    let generated_offset =
                        self.import_source_map.get_virtual_offset(*generated_offset);
                    byte_offset_to_utf16_offset(generated_source, generated_offset)
                })
                .collect::<Vec<_>>()
        });
        let options = requests
            .iter()
            .map(
                |(_, load_property_types, load_signatures)| TypeProbeOptions {
                    load_property_types: *load_property_types,
                    load_signatures: *load_signatures,
                },
            )
            .collect::<Vec<_>>();
        profile!(
            "patina.corsa_session.probe_type_batch",
            block_on(probe_types_at_positions(
                &self.session,
                self.virtual_file_wire.as_str(),
                positions,
                options,
            ))
        )
        .map_err(|error| {
            compact_error(
                "Failed to query checker type probe",
                error.to_compact_string().as_str(),
            )
        })
    }
}

async fn probe_types_at_positions(
    session: &ProjectSession,
    file: impl Into<DocumentIdentifier>,
    positions: Vec<u32>,
    options: Vec<TypeProbeOptions>,
) -> corsa::Result<Vec<Option<TypeProbe>>> {
    let file = file.into();
    let symbols = match profile!(
        "patina.corsa_session.symbols_at_positions",
        session
            .client()
            .get_symbols_at_positions(
                session.snapshot().handle.clone(),
                session.project_handle(),
                file.clone(),
                positions.clone(),
            )
            .await
    ) {
        Ok(symbols) => symbols,
        Err(error) if is_snapshot_registry_handle_error(&error) => vec![None; positions.len()],
        Err(error) => return Err(error),
    };
    if symbols.len() != positions.len() {
        return Err(CorsaError::Protocol(
            "Incomplete type probe symbol batch".into(),
        ));
    }

    let symbol_slots = symbols
        .iter()
        .enumerate()
        .filter_map(|(index, symbol)| symbol.as_ref().map(|symbol| (index, symbol.id.clone())))
        .collect::<Vec<_>>();
    let symbol_types = if symbol_slots.is_empty() {
        Vec::new()
    } else {
        match profile!(
            "patina.corsa_session.types_of_symbols",
            session
                .get_types_of_symbols(
                    symbol_slots
                        .iter()
                        .map(|(_, symbol)| symbol.clone())
                        .collect(),
                )
                .await
        ) {
            Ok(types) => types,
            Err(error) if is_snapshot_registry_handle_error(&error) => {
                vec![None; symbol_slots.len()]
            }
            Err(error) => return Err(error),
        }
    };
    if symbol_types.len() != symbol_slots.len() {
        return Err(CorsaError::Protocol(
            "Incomplete type probe symbol-type batch".into(),
        ));
    }

    let mut responses = vec![None; positions.len()];
    for ((index, _), response) in symbol_slots.into_iter().zip(symbol_types) {
        let Some(slot) = responses.get_mut(index) else {
            return Err(CorsaError::Protocol(
                "Type probe symbol index escaped its batch".into(),
            ));
        };
        *slot = response;
    }
    let missing = responses
        .iter()
        .enumerate()
        .filter_map(|(index, response)| response.is_none().then_some(index))
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        let mut missing_positions = Vec::with_capacity(missing.len());
        for &index in &missing {
            let Some(&position) = positions.get(index) else {
                return Err(CorsaError::Protocol(
                    "Type probe position index escaped its batch".into(),
                ));
            };
            missing_positions.push(position);
        }
        let positional = session
            .client()
            .get_types_at_positions(
                session.snapshot().handle.clone(),
                session.project_handle(),
                file,
                missing_positions,
            )
            .await?;
        if positional.len() != missing.len() {
            return Err(CorsaError::Protocol(
                "Incomplete type probe position batch".into(),
            ));
        }
        for (index, response) in missing.into_iter().zip(positional) {
            let Some(slot) = responses.get_mut(index) else {
                return Err(CorsaError::Protocol(
                    "Type probe response index escaped its batch".into(),
                ));
            };
            *slot = response;
        }
    }

    let mut cache: FxHashMap<(_, bool, bool), Option<TypeProbe>> = FxHashMap::default();
    let mut probes = Vec::with_capacity(responses.len());
    for (response, options) in responses.into_iter().zip(options) {
        let Some(response) = response else {
            probes.push(None);
            continue;
        };
        let key = (
            response.id.clone(),
            options.load_property_types,
            options.load_signatures,
        );
        let probe = match cache.get(&key) {
            Some(probe) => probe.clone(),
            None => {
                let probe = probe_type_response(session, response, options).await?;
                cache.insert(key, probe.clone());
                probe
            }
        };
        probes.push(probe);
    }
    Ok(probes)
}

async fn probe_type_response(
    session: &ProjectSession,
    type_response: TypeResponse,
    options: TypeProbeOptions,
) -> corsa::Result<Option<TypeProbe>> {
    let Some(type_texts) = render_type_texts(session, &type_response).await? else {
        return Ok(None);
    };
    let mut probe = TypeProbe {
        type_texts,
        property_names: Vec::new(),
        property_types: Vec::new(),
        call_signatures: Vec::new(),
        return_types: Vec::new(),
    };

    let properties = match session
        .get_properties_of_type(type_response.id.clone())
        .await
    {
        Ok(properties) => properties,
        Err(error) if is_snapshot_registry_handle_error(&error) => Vec::new(),
        Err(error) => return Err(error),
    };
    probe.property_names.reserve(properties.len());
    for property in &properties {
        probe
            .property_names
            .push(property.name.as_str().to_compact_string());
    }

    if options.load_property_types && !properties.is_empty() {
        let property_types = match profile!(
            "patina.corsa_session.types_of_symbols",
            session
                .get_types_of_symbols(
                    properties
                        .iter()
                        .map(|property| property.id.clone())
                        .collect(),
                )
                .await
        ) {
            Ok(property_types) => property_types,
            Err(error) if is_snapshot_registry_handle_error(&error) => {
                vec![None; properties.len()]
            }
            Err(error) => return Err(error),
        };

        probe.property_types.reserve(property_types.len());
        for property_type in property_types {
            if let Some(property_type) = property_type {
                probe.property_types.push(
                    render_type_texts(session, &property_type)
                        .await?
                        .unwrap_or_default(),
                );
            } else {
                probe.property_types.push(Vec::new());
            }
        }
    }

    if options.load_signatures {
        let signatures = match session.get_signatures_of_type(type_response.id, 0).await {
            Ok(signatures) => signatures,
            Err(error) if is_snapshot_registry_handle_error(&error) => Vec::new(),
            Err(error) => return Err(error),
        };
        probe.call_signatures.reserve(signatures.len());
        probe.return_types.reserve(signatures.len());

        for signature in signatures {
            if signature.parameters.is_empty() {
                probe.call_signatures.push(Vec::new());
            } else {
                let parameter_count = signature.parameters.len();
                let parameter_types = match profile!(
                    "patina.corsa_session.types_of_symbols",
                    session.get_types_of_symbols(signature.parameters).await
                ) {
                    Ok(parameter_types) => parameter_types,
                    Err(error) if is_snapshot_registry_handle_error(&error) => {
                        vec![None; parameter_count]
                    }
                    Err(error) => return Err(error),
                };
                let mut rendered_parameters = Vec::with_capacity(parameter_types.len());

                for parameter_type in parameter_types {
                    if let Some(parameter_type) = parameter_type {
                        rendered_parameters.push(
                            render_type_texts(session, &parameter_type)
                                .await?
                                .unwrap_or_default(),
                        );
                    } else {
                        rendered_parameters.push(Vec::new());
                    }
                }

                probe.call_signatures.push(rendered_parameters);
            }

            let return_type = match session.get_return_type_of_signature(signature.id).await {
                Ok(return_type) => return_type,
                Err(error) if is_snapshot_registry_handle_error(&error) => None,
                Err(error) => return Err(error),
            };
            if let Some(return_type) = return_type {
                probe.return_types.push(
                    render_type_texts(session, &return_type)
                        .await?
                        .unwrap_or_default(),
                );
            } else {
                probe.return_types.push(Vec::new());
            }
        }
    }

    Ok(Some(probe))
}

async fn render_type_texts(
    session: &ProjectSession,
    type_response: &TypeResponse,
) -> corsa::Result<Option<Vec<CorsaString>>> {
    match session.render_type_texts(type_response).await {
        Ok(type_texts) => Ok(Some(type_texts)),
        Err(error) if is_snapshot_registry_handle_error(&error) => Ok(None),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
fn select_probe_type_response(
    symbol_type_response: Option<TypeResponse>,
    position_type_response: Option<TypeResponse>,
) -> Option<TypeResponse> {
    symbol_type_response.or(position_type_response)
}

fn is_snapshot_registry_handle_error(error: &CorsaError) -> bool {
    match error {
        CorsaError::Rpc(rpc) => is_snapshot_registry_handle_message(rpc.message.as_str()),
        CorsaError::Protocol(message) => is_snapshot_registry_handle_message(message.as_str()),
        _ => false,
    }
}

fn is_snapshot_registry_handle_message(message: &str) -> bool {
    if message == SNAPSHOT_REGISTRY_NOT_FOUND {
        return true;
    }

    let message = message
        .strip_prefix("api: client error: ")
        .unwrap_or(message);
    is_missing_snapshot_handle(message, "type handle ")
        || is_missing_snapshot_handle(message, "symbol handle ")
}

fn is_missing_snapshot_handle(message: &str, prefix: &str) -> bool {
    let Some(handle) = message.strip_prefix(prefix) else {
        return false;
    };
    let Some(handle) = handle.strip_suffix(" not found in snapshot registry") else {
        return false;
    };
    !handle.trim().is_empty()
}

pub(super) fn byte_offset_to_utf16_offset(source: &str, byte_offset: u32) -> u32 {
    let mut clamped = usize::min(byte_offset as usize, source.len());
    while clamped > 0 && !source.is_char_boundary(clamped) {
        clamped -= 1;
    }
    let prefix = source.get(..clamped).unwrap_or_default();
    prefix.encode_utf16().count() as u32
}

#[cfg(test)]
mod tests {
    use super::{is_snapshot_registry_handle_error, select_probe_type_response};
    use corsa::{
        CorsaError,
        api::{TypeHandle, TypeResponse},
        error::RpcResponseError,
        fast::CompactString,
    };
    use serde_json::json;

    fn type_response(id: &str) -> TypeResponse {
        TypeResponse {
            id: TypeHandle::from(id),
            flags: 0,
            object_flags: None,
            value: None,
            target: None,
            type_parameters: Vec::new(),
            outer_type_parameters: Vec::new(),
            local_type_parameters: Vec::new(),
            element_flags: Vec::new(),
            fixed_length: None,
            readonly: None,
            object_type: None,
            index_type: None,
            check_type: None,
            extends_type: None,
            base_type: None,
            subst_constraint: None,
            texts: Vec::new(),
            symbol: None,
        }
    }

    #[test]
    fn symbol_type_response_wins_over_position_type_response() {
        let selected = select_probe_type_response(
            Some(type_response("symbol-type")),
            Some(type_response("position-type")),
        )
        .expect("expected a selected type");

        assert_eq!(selected.id.as_str(), "symbol-type");
    }

    #[test]
    fn position_type_response_is_used_when_symbol_lookup_misses() {
        let selected = select_probe_type_response(None, Some(type_response("position-type")))
            .expect("expected a selected type");

        assert_eq!(selected.id.as_str(), "position-type");
    }

    #[test]
    fn recognizes_structured_snapshot_registry_rpc_errors() {
        assert!(is_snapshot_registry_handle_error(&CorsaError::Rpc(
            RpcResponseError {
                code: -32603,
                message: CompactString::from(
                    "api: client error: type handle \"t0000000000000057\" not found in snapshot registry",
                ),
                data: Some(json!({ "method": "getPropertiesOfType" })),
            },
        )));
        assert!(is_snapshot_registry_handle_error(&CorsaError::Rpc(
            RpcResponseError {
                code: -32603,
                message: CompactString::from("not found in snapshot registry"),
                data: None,
            },
        )));
        assert!(is_snapshot_registry_handle_error(&CorsaError::Rpc(
            RpcResponseError {
                code: -32603,
                message: CompactString::from(
                    "api: client error: symbol handle 162 not found in snapshot registry",
                ),
                data: Some(json!({ "method": "getTypeOfSymbol" })),
            },
        )));
    }

    #[test]
    fn unrelated_errors_are_not_snapshot_registry_handles() {
        assert!(!is_snapshot_registry_handle_error(&CorsaError::Rpc(
            RpcResponseError {
                code: -32601,
                message: CompactString::from(
                    "api: invalid request: unknown API method \"getPropertiesOfType\"",
                ),
                data: Some(json!({ "method": "getPropertiesOfType" })),
            },
        )));
        assert!(!is_snapshot_registry_handle_error(&CorsaError::Protocol(
            CompactString::from("api: client error: snapshot registry handle not found"),
        )));
        assert!(!is_snapshot_registry_handle_error(&CorsaError::Protocol(
            CompactString::from("api: client error: symbol handle not found in snapshot registry"),
        )));
    }
}
