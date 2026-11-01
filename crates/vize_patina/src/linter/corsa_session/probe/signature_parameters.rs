//! A compact signature mention does not establish its symbol's connection owner.

use super::is_snapshot_registry_handle_error;
use corsa::{
    CorsaError,
    api::{ProjectSession, SignatureResponse, SymbolHandle, SymbolResponse, TypeResponse},
};
use vize_l0::profile;

pub(super) async fn load(
    session: &ProjectSession,
    signature: &SignatureResponse,
) -> corsa::Result<Vec<Option<TypeResponse>>> {
    // Full symbols teach this exact connection their owner. Keep the signature's
    // project and snapshot; neither names nor a second session can supply it.
    // In particular, authentic getter failures must reach the caller unchanged.
    let parameters = session
        .client()
        .get_parameters_of_signature(
            session.snapshot().handle.clone(),
            session.project_handle(),
            signature.id.clone(),
        )
        .await?;
    validate_parameters(&signature.parameters, &parameters)?;
    let parameter_count = parameters.len();
    let types = match profile!(
        "patina.corsa_session.types_of_symbols",
        session
            .get_types_of_symbols(
                parameters
                    .into_iter()
                    .map(|parameter| parameter.id)
                    .collect()
            )
            .await
    ) {
        Ok(types) => types,
        Err(error) if is_snapshot_registry_handle_error(&error) => vec![None; parameter_count],
        Err(error) => return Err(error),
    };
    if types.len() != parameter_count {
        return Err(CorsaError::Protocol(
            "Incomplete signature parameter-type batch".into(),
        ));
    }
    Ok(types)
}

fn validate_parameters(
    expected: &[SymbolHandle],
    parameters: &[SymbolResponse],
) -> corsa::Result<()> {
    if !parameters
        .iter()
        .map(|parameter| &parameter.id)
        .eq(expected.iter())
    {
        return Err(CorsaError::Protocol(
            "Signature parameter symbols do not match their ordered mentions".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{SymbolHandle, SymbolResponse, validate_parameters};
    use corsa::fast::ToCompactString as _;
    use serde_json::json;

    fn parameter(id: &str) -> SymbolResponse {
        serde_json::from_value(json!({
            "id":id,"name":"value","flags":1,"checkFlags":0,
            "declarations":[],"valueDeclaration":null,
        }))
        .unwrap()
    }

    #[test]
    fn ordered_mentions_require_every_exact_full_symbol() {
        let ids = ["{\"file\":\"160\",\"id\":5}", "{\"file\":\"160\",\"id\":6}"];
        let expected = ids
            .iter()
            .map(|id| serde_json::from_value(json!(id)).unwrap())
            .collect::<Vec<SymbolHandle>>();
        assert!(validate_parameters(&expected, &[parameter(ids[0]), parameter(ids[1])]).is_ok());
        for invalid in [
            vec![],
            vec![parameter(ids[0])],
            vec![parameter(ids[1]), parameter(ids[0])],
            vec![parameter(ids[0]), parameter(ids[0])],
            vec![
                parameter(ids[0]),
                parameter("{\"file\":\"other\",\"id\":6}"),
            ],
            vec![parameter(ids[0]), parameter(ids[1]), parameter("7")],
        ] {
            assert_eq!(
                validate_parameters(&expected, &invalid)
                    .unwrap_err()
                    .to_compact_string(),
                "protocol error: Signature parameter symbols do not match their ordered mentions",
            );
        }
    }
}
