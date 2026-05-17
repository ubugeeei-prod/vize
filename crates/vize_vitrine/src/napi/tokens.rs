//! NAPI bindings for Musea design token helpers.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use napi::bindgen_prelude::{Error, Result, Status};
use napi_derive::napi;
use serde::Serialize;
use vize_carton::ToCompactString;

#[napi(js_name = "parseDesignTokensFromPath")]
pub fn parse_design_tokens_from_path(tokens_path: String) -> Result<String> {
    to_json(vize_musea::parse_tokens_from_path(tokens_path).map_err(to_napi_error)?)
}

#[napi(js_name = "parseDesignTokensFromJson")]
pub fn parse_design_tokens_from_json(source: String) -> Result<String> {
    to_json(vize_musea::parse_tokens_from_json(&source).map_err(to_napi_error)?)
}

#[napi(js_name = "buildDesignTokenMap")]
pub fn build_design_token_map(categories: String) -> Result<String> {
    let categories = categories_from_json(&categories)?;
    to_json(vize_musea::build_token_map(&categories))
}

#[napi(js_name = "resolveDesignTokenReferences")]
pub fn resolve_design_token_references(categories: String) -> Result<String> {
    let categories = categories_from_json(&categories)?;
    to_json(vize_musea::resolve_token_categories(categories))
}

#[napi(js_name = "flattenDesignTokenCategories")]
pub fn flatten_design_token_categories(categories: String) -> Result<String> {
    let categories = categories_from_json(&categories)?;
    to_json(vize_musea::flatten_token_categories(&categories))
}

#[napi(js_name = "generateDesignTokensMarkdown")]
pub fn generate_design_tokens_markdown(
    categories: String,
    generated_at: Option<String>,
) -> Result<String> {
    let categories = categories_from_json(&categories)?;
    Ok(vize_musea::generate_tokens_markdown(&categories, generated_at.as_deref()).into())
}

#[napi(js_name = "validateDesignTokenReference")]
pub fn validate_design_token_reference(
    token_map: String,
    reference: String,
    self_path: Option<String>,
) -> Result<String> {
    let token_map = token_map_from_json(&token_map)?;
    to_json(vize_musea::validate_reference(
        &token_map,
        &reference,
        self_path.as_deref(),
    ))
}

#[napi(js_name = "findDependentDesignTokens")]
pub fn find_dependent_design_tokens(token_map: String, target_path: String) -> Result<Vec<String>> {
    let token_map = token_map_from_json(&token_map)?;
    Ok(vize_musea::find_dependent_tokens(&token_map, &target_path)
        .into_iter()
        .map(Into::into)
        .collect())
}

fn categories_from_json(source: &str) -> Result<Vec<vize_musea::tokens::TokenCategory>> {
    serde_json::from_str(source).map_err(|error| {
        Error::new(
            Status::InvalidArg,
            vize_carton::cstr!("Invalid token categories: {error}"),
        )
    })
}

fn token_map_from_json(
    source: &str,
) -> Result<vize_carton::FxHashMap<vize_carton::String, vize_musea::tokens::DesignToken>> {
    serde_json::from_str(source).map_err(|error| {
        Error::new(
            Status::InvalidArg,
            vize_carton::cstr!("Invalid token map: {error}"),
        )
    })
}

fn to_json(value: impl Serialize) -> Result<String> {
    serde_json::to_string(&value).map_err(|error| {
        Error::new(
            Status::GenericFailure,
            vize_carton::cstr!("Failed to serialize token result: {error}"),
        )
    })
}

fn to_napi_error(error: vize_musea::tokens::TokenError) -> Error {
    Error::new(Status::GenericFailure, error.to_compact_string())
}
