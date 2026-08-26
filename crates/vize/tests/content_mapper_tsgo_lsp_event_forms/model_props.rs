//! defineModel-derived prop navigation covered by the standard tsgo LSP oracle.

use corsa_lsp::LspClient;
use serde_json::json;

use super::content_mapper_lsp_support::{
    assert_hover, assert_location_range, assert_no_generated_uri_or_zero_range, definition, hover,
    position_range, references,
};

pub(crate) async fn assert_define_model_prop_navigation(
    client: &LspClient,
    app_uri: &str,
    app_source: &str,
    named_model_uri: &str,
    named_model_source: &str,
) {
    let default_start_offset = app_source.find(":model-value").unwrap() + 1;
    let (default_start, default_end) =
        position_range(app_source, default_start_offset, "model-value".len());
    let default_hover = hover(client, app_uri, &default_start).await;
    assert_hover(
        &default_hover,
        &default_start,
        &default_end,
        json!({ "kind": "plaintext", "value": "(property) \"modelValue\": number" }),
        "default defineModel prop hover",
    );
    let default_definition = definition(client, app_uri, &default_start).await;
    assert_no_generated_uri_or_zero_range(&default_definition);
    assert_location_range(
        &default_definition,
        app_uri,
        &default_start,
        &default_end,
        "default defineModel prop definition",
    );
    let default_references = references(client, app_uri, &default_start).await;
    assert_no_generated_uri_or_zero_range(&default_references);
    assert_location_range(
        &default_references,
        app_uri,
        &default_start,
        &default_end,
        "default defineModel prop references",
    );

    let named_prop_start_offset = app_source.find("title=\"chosen\"").unwrap();
    let (named_prop_start, named_prop_end) =
        position_range(app_source, named_prop_start_offset, "title".len());
    let named_literal_offset = named_model_source.find("\"title\"").unwrap();
    let (named_declaration_start, named_declaration_end) =
        position_range(named_model_source, named_literal_offset, "\"title\"".len());
    let (named_reference_start, named_reference_end) =
        position_range(named_model_source, named_literal_offset + 1, "title".len());

    let named_hover = hover(client, app_uri, &named_prop_start).await;
    assert_hover(
        &named_hover,
        &named_prop_start,
        &named_prop_end,
        json!({ "kind": "plaintext", "value": "(property) \"title\": string" }),
        "named defineModel prop hover",
    );
    let named_definition = definition(client, app_uri, &named_prop_start).await;
    assert_no_generated_uri_or_zero_range(&named_definition);
    assert_location_range(
        &named_definition,
        app_uri,
        &named_prop_start,
        &named_prop_end,
        "named defineModel prop definition",
    );
    assert_location_range(
        &named_definition,
        named_model_uri,
        &named_declaration_start,
        &named_declaration_end,
        "named defineModel prop definition",
    );
    let named_references = references(client, app_uri, &named_prop_start).await;
    assert_no_generated_uri_or_zero_range(&named_references);
    assert_location_range(
        &named_references,
        app_uri,
        &named_prop_start,
        &named_prop_end,
        "named defineModel prop references",
    );
    assert_location_range(
        &named_references,
        named_model_uri,
        &named_reference_start,
        &named_reference_end,
        "named defineModel prop references",
    );
}
