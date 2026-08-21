use super::{CorsaTypeAwareSession, TypeProbe, errors::compact_error};
use corsa::{
    CorsaError,
    api::{DocumentIdentifier, ProjectSession, TypeProbeOptions, TypeResponse},
    fast::ToCompactString as _,
    runtime::block_on,
};
use vize_carton::{String, profile};

impl CorsaTypeAwareSession {
    pub(in crate::linter) fn probe_type_at_offset(
        &self,
        generated_source: &str,
        generated_offset: u32,
        load_property_types: bool,
        load_signatures: bool,
    ) -> Result<Option<TypeProbe>, String> {
        let utf16_offset = profile!(
            "patina.corsa_session.byte_to_utf16",
            byte_offset_to_utf16_offset(generated_source, generated_offset)
        );
        profile!(
            "patina.corsa_session.probe_type",
            block_on(probe_type_at_position(
                &self.session,
                self.virtual_file_wire.as_str(),
                utf16_offset,
                TypeProbeOptions {
                    load_property_types,
                    load_signatures,
                },
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

async fn probe_type_at_position(
    session: &ProjectSession,
    file: impl Into<DocumentIdentifier>,
    position: u32,
    options: TypeProbeOptions,
) -> corsa::Result<Option<TypeProbe>> {
    let file = file.into();
    let type_response = match session.get_type_at_position(file.clone(), position).await? {
        Some(type_response) => Some(type_response),
        None => type_from_symbol_at_position(session, file.clone(), position).await?,
    };
    let Some(type_response) = type_response else {
        return Ok(None);
    };

    let mut probe = TypeProbe {
        type_texts: session.render_type_texts(&type_response).await?,
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
        let property_types = match session
            .get_types_of_symbols(
                properties
                    .iter()
                    .map(|property| property.id.clone())
                    .collect(),
            )
            .await
        {
            Ok(property_types) => property_types,
            Err(error) if is_snapshot_registry_handle_error(&error) => {
                vec![None; properties.len()]
            }
            Err(error) => return Err(error),
        };

        probe.property_types.reserve(property_types.len());
        for property_type in property_types {
            if let Some(property_type) = property_type {
                probe
                    .property_types
                    .push(session.render_type_texts(&property_type).await?);
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
                let parameter_types = match session.get_types_of_symbols(signature.parameters).await
                {
                    Ok(parameter_types) => parameter_types,
                    Err(error) if is_snapshot_registry_handle_error(&error) => {
                        vec![None; parameter_count]
                    }
                    Err(error) => return Err(error),
                };
                let mut rendered_parameters = Vec::with_capacity(parameter_types.len());

                for parameter_type in parameter_types {
                    if let Some(parameter_type) = parameter_type {
                        rendered_parameters.push(session.render_type_texts(&parameter_type).await?);
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
                probe
                    .return_types
                    .push(session.render_type_texts(&return_type).await?);
            } else {
                probe.return_types.push(Vec::new());
            }
        }
    }

    Ok(Some(probe))
}

async fn type_from_symbol_at_position(
    session: &ProjectSession,
    file: DocumentIdentifier,
    position: u32,
) -> corsa::Result<Option<TypeResponse>> {
    let Some(symbol) = session.get_symbol_at_position(file, position).await? else {
        return Ok(None);
    };
    session.get_type_of_symbol(symbol.id).await
}

fn is_snapshot_registry_handle_error(error: &CorsaError) -> bool {
    let message = error.to_compact_string();
    let message = message.as_str();
    message.contains("snapshot registry")
        && message.contains("handle")
        && message.contains("not found")
}

pub(super) fn byte_offset_to_utf16_offset(source: &str, byte_offset: u32) -> u32 {
    let mut clamped = usize::min(byte_offset as usize, source.len());
    while clamped > 0 && !source.is_char_boundary(clamped) {
        clamped -= 1;
    }
    source[..clamped].encode_utf16().count() as u32
}
