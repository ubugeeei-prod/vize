//! Native HTML/SVG/MathML definition helpers.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

#[cfg(feature = "native")]
use std::sync::Arc;

use tower_lsp::lsp_types::GotoDefinitionResponse;
#[cfg(feature = "native")]
use tower_lsp::lsp_types::{Location, Position, Range};

#[cfg(feature = "native")]
use vize_canon::CorsaBridge;

#[cfg(feature = "native")]
use super::helpers;
use super::{DefinitionService, IdeContext, template};
#[cfg(feature = "native")]
use crate::ide::corsa_support;
#[cfg(feature = "native")]
use crate::ide::is_component_tag;

impl DefinitionService {
    pub(super) fn definition_in_template_sync(
        ctx: &IdeContext<'_>,
    ) -> Option<GotoDefinitionResponse> {
        #[cfg(feature = "native")]
        if let Some(definition) = Self::definition_for_native_html_tag(ctx) {
            return Some(definition);
        }

        if let Some(definition) = template::find_component_prop_definition(ctx) {
            return Some(definition);
        }

        if let Some(target) = crate::ide::template_ref::target_at_offset(ctx) {
            return Some(GotoDefinitionResponse::Scalar(target.binding_location(ctx)));
        }

        #[cfg(feature = "native")]
        if let Some(definition) = Self::definition_for_native_html_attribute(ctx) {
            return Some(definition);
        }

        template::definition_in_template(ctx)
    }

    #[cfg(feature = "native")]
    fn definition_for_native_html_tag(ctx: &IdeContext<'_>) -> Option<GotoDefinitionResponse> {
        let tag_name = helpers::get_tag_at_offset(&ctx.content, ctx.offset)?;
        if is_component_tag(&tag_name) {
            return None;
        }

        let info = corsa_support::native_dom_tag_info(&tag_name)?;
        external_uri_response(&info.documentation_url)
    }

    #[cfg(feature = "native")]
    fn definition_for_native_html_attribute(
        ctx: &IdeContext<'_>,
    ) -> Option<GotoDefinitionResponse> {
        let (attr_name, tag_name) = helpers::get_attribute_and_component_at_offset(ctx)?;
        if is_component_tag(&tag_name) {
            return None;
        }

        let info = corsa_support::native_dom_attribute_info(&tag_name, &attr_name)?;
        external_uri_response(&info.documentation_url)
    }

    #[cfg(feature = "native")]
    pub(super) async fn definition_for_html_tag_with_corsa(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<&Arc<CorsaBridge>>,
    ) -> Option<GotoDefinitionResponse> {
        let tag_name = helpers::get_tag_at_offset(&ctx.content, ctx.offset)?;
        if is_component_tag(&tag_name) {
            return None;
        }

        let bridge = corsa_bridge?;
        if !bridge.is_initialized() {
            return None;
        }

        let doc = corsa_support::html_tag_virtual_document(&tag_name)?;
        let request_path = corsa_support::html_tag_request_path(ctx.uri);
        let request_uri = bridge
            .open_or_update_virtual_document(&request_path, &doc.content)
            .await
            .ok()?;
        let (line, character) = crate::ide::offset_to_position(&doc.content, doc.definition_offset);
        let locations = bridge
            .definition(&request_uri, line, character)
            .await
            .ok()?;

        external_lsp_locations_response(locations, &request_uri)
    }

    #[cfg(feature = "native")]
    pub(super) async fn definition_for_html_attribute_with_corsa(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<&Arc<CorsaBridge>>,
    ) -> Option<GotoDefinitionResponse> {
        let (attr_name, tag_name) = helpers::get_attribute_and_component_at_offset(ctx)?;
        if is_component_tag(&tag_name) {
            return None;
        }

        let bridge = corsa_bridge?;
        if !bridge.is_initialized() {
            return None;
        }

        let doc = corsa_support::html_attribute_virtual_document(&tag_name, &attr_name)?;
        let request_path = corsa_support::html_attribute_request_path(ctx.uri);
        let request_uri = bridge
            .open_or_update_virtual_document(&request_path, &doc.content)
            .await
            .ok()?;
        let (line, character) = crate::ide::offset_to_position(&doc.content, doc.definition_offset);
        let locations = bridge
            .definition(&request_uri, line, character)
            .await
            .ok()?;

        external_lsp_locations_response(locations, &request_uri)
    }
}

#[cfg(feature = "native")]
fn external_uri_response(uri: &str) -> Option<GotoDefinitionResponse> {
    Some(GotoDefinitionResponse::Scalar(Location {
        uri: tower_lsp::lsp_types::Url::parse(uri).ok()?,
        range: Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 0,
            },
        },
    }))
}

#[cfg(feature = "native")]
fn external_lsp_locations_response(
    locations: Vec<vize_canon::LspLocation>,
    synthetic_request_uri: &str,
) -> Option<GotoDefinitionResponse> {
    let locations = locations
        .into_iter()
        .filter(|location| location.uri != synthetic_request_uri)
        .filter_map(|location| {
            Some(Location {
                uri: tower_lsp::lsp_types::Url::parse(&location.uri).ok()?,
                range: Range {
                    start: Position {
                        line: location.range.start.line,
                        character: location.range.start.character,
                    },
                    end: Position {
                        line: location.range.end.line,
                        character: location.range.end.character,
                    },
                },
            })
        })
        .collect();

    locations_response(locations)
}

#[cfg(feature = "native")]
fn locations_response(locations: Vec<Location>) -> Option<GotoDefinitionResponse> {
    match locations.as_slice() {
        [] => None,
        [location] => Some(GotoDefinitionResponse::Scalar(location.clone())),
        _ => Some(GotoDefinitionResponse::Array(locations)),
    }
}
