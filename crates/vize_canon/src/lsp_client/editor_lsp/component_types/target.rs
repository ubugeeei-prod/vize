//! Resolve the configured Vue component type on the retained native snapshot.

use super::{ALIAS, MODULE, capture, known_type};
use crate::lsp_client::session::uri_document_identifier;
use corsa::{
    api::{ApiClient, ProjectHandle, SnapshotHandle, TypeResponse},
    runtime::block_on,
};
use serde_json::json;
use std::collections::BTreeSet;
use vize_l0::cstr;

#[derive(serde::Serialize)]
pub(super) struct ComponentTarget {
    pub(super) component: TypeResponse,
    pub(super) option_properties: BTreeSet<String>,
}

pub(super) fn component_target(
    api: &ApiClient,
    snapshot: &SnapshotHandle,
    project: &ProjectHandle,
    uri: &str,
    position: u32,
    capture: &mut capture::Capture,
) -> corsa::Result<Option<ComponentTarget>> {
    let module = block_on(api.get_symbol_at_position(
        snapshot.clone(),
        project.clone(),
        uri_document_identifier(uri),
        position,
    ));
    capture.record("moduleSymbol", || match &module {
        Ok(value) => json!({"result":value}),
        Err(error) => json!({"error":cstr!("{error}")}),
    });
    let Some(module) = module? else {
        return Ok(None);
    };
    if module.flags & MODULE == 0 {
        return Ok(None);
    }
    let exports = block_on(api.get_exports_of_module(snapshot.clone(), project.clone(), module.id));
    capture.record("moduleExports", || match &exports {
        Ok(value) => json!({"result":value}),
        Err(error) => json!({"error":cstr!("{error}")}),
    });
    let exports = exports?;
    let mut option_exports = exports
        .iter()
        .filter(|symbol| symbol.name == "ComponentOptions");
    let Some(mut options) = option_exports.next().cloned() else {
        return Ok(None);
    };
    if option_exports.next().is_some() {
        return Ok(None);
    }
    if options.flags & ALIAS != 0 {
        let Some(resolved) =
            block_on(api.get_aliased_symbol(snapshot.clone(), project.clone(), options.id))?
        else {
            return Ok(None);
        };
        options = resolved;
    }
    capture.record("componentOptionsExport", || json!(options));
    let options_type =
        block_on(api.get_declared_type_of_symbol(snapshot.clone(), project.clone(), options.id));
    capture.record("componentOptionsType", || match &options_type {
        Ok(value) => json!({"result":value}),
        Err(error) => json!({"error":cstr!("{error}")}),
    });
    let Some(options_type) = options_type?.filter(known_type) else {
        return Ok(None);
    };
    let properties =
        block_on(api.get_properties_of_type(snapshot.clone(), project.clone(), options_type.id));
    capture.record("componentOptionProperties", || match &properties {
        Ok(value) => json!({"result":value}),
        Err(error) => json!({"error":cstr!("{error}")}),
    });
    let option_properties = properties?
        .into_iter()
        .map(|symbol| symbol.name)
        .collect::<BTreeSet<_>>();
    if option_properties.is_empty() {
        return Ok(None);
    }
    let mut exports = exports.into_iter().filter(|symbol| symbol.name == "App");
    let Some(mut symbol) = exports.next() else {
        return Ok(None);
    };
    if exports.next().is_some() {
        return Ok(None);
    }
    if symbol.flags & ALIAS != 0 {
        let resolved =
            block_on(api.get_aliased_symbol(snapshot.clone(), project.clone(), symbol.id));
        capture.record("appAlias", || match &resolved {
            Ok(value) => json!({"result":value}),
            Err(error) => json!({"error":cstr!("{error}")}),
        });
        let Some(resolved) = resolved? else {
            return Ok(None);
        };
        symbol = resolved;
    }
    capture.record("appExport", || json!(symbol));
    let declared =
        block_on(api.get_declared_type_of_symbol(snapshot.clone(), project.clone(), symbol.id));
    capture.record("declaredAppType", || match &declared {
        Ok(value) => json!({"result":value}),
        Err(error) => json!({"error":cstr!("{error}")}),
    });
    let Some(declared) = declared?.filter(known_type) else {
        return Ok(None);
    };
    let property = block_on(api.get_property_of_type(
        snapshot.clone(),
        project.clone(),
        declared.id,
        "component",
    ));
    capture.record("componentProperty", || match &property {
        Ok(value) => json!({"result":value}),
        Err(error) => json!({"error":cstr!("{error}")}),
    });
    let Some(property) = property? else {
        return Ok(None);
    };
    let method = block_on(api.get_type_of_symbol(snapshot.clone(), project.clone(), property.id));
    capture.record("componentMethodType", || match &method {
        Ok(value) => json!({"result":value}),
        Err(error) => json!({"error":cstr!("{error}")}),
    });
    let Some(method) = method?.filter(known_type) else {
        return Ok(None);
    };
    let signatures =
        block_on(api.get_signatures_of_type(snapshot.clone(), project.clone(), method.id, 0));
    capture.record("componentSignatures", || match &signatures {
        Ok(value) => json!({"result":value}),
        Err(error) => json!({"error":cstr!("{error}")}),
    });
    // The nongeneric getter references Component with Vue's own defaults.
    // Its generic setter parameter would retain an unresolved type parameter.
    let mut getters = signatures?.into_iter().filter(|signature| {
        signature.parameters.len() == 1 && signature.type_parameters.is_empty()
    });
    let Some(getter) = getters.next() else {
        return Ok(None);
    };
    if getters.next().is_some() {
        return Ok(None);
    }
    let returned =
        block_on(api.get_return_type_of_signature(snapshot.clone(), project.clone(), getter.id));
    capture.record("componentGetterReturnType", || match &returned {
        Ok(value) => json!({"result":value}),
        Err(error) => json!({"error":cstr!("{error}")}),
    });
    let Some(returned) = returned?.filter(known_type) else {
        return Ok(None);
    };
    let component =
        block_on(api.get_non_nullable_type(snapshot.clone(), project.clone(), returned.id));
    capture.record("instantiatedComponentType", || match &component {
        Ok(value) => json!({"result":value}),
        Err(error) => json!({"error":cstr!("{error}")}),
    });
    Ok(component?
        .filter(known_type)
        .map(|component| ComponentTarget {
            component,
            option_properties,
        }))
}
