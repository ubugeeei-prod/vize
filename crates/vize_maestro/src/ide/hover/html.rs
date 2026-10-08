#![expect(
    clippy::disallowed_macros,
    reason = "`format!` builds the std `String` values tower_lsp::lsp_types payloads take"
)]

use std::sync::Arc;

use tower_lsp::lsp_types::{Hover, Position, Range};
use vize_canon::{CorsaBridge, LspHover};

use super::{HoverBuilder, HoverService};
use crate::ide::IdeContext;

impl HoverService {
    pub(super) fn hover_native_dom_attribute(ctx: &IdeContext<'_>) -> Option<Hover> {
        let (attr_name, tag_name) =
            crate::ide::definition::helpers::get_attribute_and_component_at_offset(ctx)?;
        if crate::ide::is_component_tag(&tag_name) {
            return None;
        }

        let info = crate::ide::corsa_support::native_dom_attribute_info(&tag_name, &attr_name)?;
        let signature = format!("{}: {}", info.property_name, info.type_expression);
        let value_kind = if info.is_boolean {
            "Boolean HTML attribute"
        } else {
            "DOM reflected attribute"
        };
        let example = native_attribute_example(&tag_name, &attr_name, info.is_boolean);

        Some(
            HoverBuilder::new()
                .title(&format!("{attr_name} on <{tag_name}>"))
                .meta(info.category)
                .code("typescript", &signature)
                .description(
                    "Native DOM attribute recognized by the Vue template compiler. Vue patches it on the platform element rather than resolving it as a component prop.",
                )
                .section("Runtime surface", value_kind)
                .bullets(
                    "Editor behavior",
                    &[
                        "Hover and go-to-definition use the DOM library type for the reflected property when the type service is available.",
                        "Component prop lookup is skipped because the enclosing tag is a native element.",
                    ],
                )
                .example("vue", &example)
                .docs("MDN reference", &info.documentation_url)
                .build(),
        )
    }

    pub(super) fn hover_native_dom_tag(ctx: &IdeContext<'_>) -> Option<Hover> {
        let tag_name =
            crate::ide::definition::helpers::get_tag_at_offset(&ctx.content, ctx.offset)?;
        if crate::ide::is_component_tag(&tag_name) {
            return None;
        }

        let info = crate::ide::corsa_support::native_dom_tag_info(&tag_name)?;
        let signature = format!("const element: {}", info.type_expression);

        Some(
            HoverBuilder::new()
                .title(&format!("<{tag_name}>"))
                .meta(info.category)
                .code("typescript", &signature)
                .description(
                    "Native DOM element recognized by the Vue template compiler. It is emitted as an element node, not resolved as a component.",
                )
                .bullets(
                    "Editor behavior",
                    &[
                        "Go-to-definition uses TypeScript DOM lib data when the type service is available.",
                        "Component resolution is skipped for this tag because it is part of the platform DOM surface.",
                    ],
                )
                .example("vue", &native_tag_example(&tag_name))
                .docs("MDN reference", &info.documentation_url)
                .build(),
        )
    }

    pub(super) async fn hover_html_attribute_with_corsa(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<&Arc<CorsaBridge>>,
    ) -> Option<Hover> {
        let (attr_name, tag_name, source_span) =
            crate::ide::definition::helpers::get_attribute_with_source_span_at_offset(ctx)?;
        if crate::ide::is_component_tag(&tag_name) {
            return None;
        }

        let bridge = corsa_bridge?;
        if !bridge.is_initialized() {
            return None;
        }

        let doc =
            crate::ide::corsa_support::html_attribute_virtual_document(&tag_name, &attr_name)?;
        let request_path = crate::ide::corsa_support::html_attribute_request_path(ctx.uri);
        let request_uri = bridge
            .open_or_update_virtual_document(&request_path, &doc.content)
            .await
            .ok()?;
        let (line, character) = crate::ide::offset_to_position(&doc.content, doc.hover_offset);
        let hover = bridge.hover(&request_uri, line, character).await.ok()??;

        Some(Self::project_native_attribute_hover(
            ctx,
            source_span,
            hover,
        ))
    }

    fn project_native_attribute_hover(
        ctx: &IdeContext<'_>,
        source_span: Option<(usize, usize)>,
        hover: LspHover,
    ) -> Hover {
        let mut converted = Self::convert_lsp_hover(hover);
        converted.range = source_span.map(|(start, end)| {
            let (line, character) = crate::ide::offset_to_position(&ctx.content, start);
            let (end_line, end_character) = crate::ide::offset_to_position(&ctx.content, end);
            Range::new(
                Position::new(line, character),
                Position::new(end_line, end_character),
            )
        });
        converted
    }

    pub(super) async fn hover_html_tag_with_corsa(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<&Arc<CorsaBridge>>,
    ) -> Option<Hover> {
        let tag_name =
            crate::ide::definition::helpers::get_tag_at_offset(&ctx.content, ctx.offset)?;
        if crate::ide::is_component_tag(&tag_name) {
            return None;
        }

        let bridge = corsa_bridge?;
        if !bridge.is_initialized() {
            return None;
        }

        let doc = crate::ide::corsa_support::html_tag_virtual_document(&tag_name)?;
        let request_path = crate::ide::corsa_support::html_tag_request_path(ctx.uri);
        let request_uri = bridge
            .open_or_update_virtual_document(&request_path, &doc.content)
            .await
            .ok()?;
        let (line, character) = crate::ide::offset_to_position(&doc.content, doc.hover_offset);
        let hover = bridge.hover(&request_uri, line, character).await.ok()??;

        Some(Self::convert_lsp_hover(hover))
    }
}

fn native_attribute_example(tag_name: &str, attr_name: &str, is_boolean: bool) -> String {
    if is_boolean {
        format!("<template>\n  <{tag_name} {attr_name}>...</{tag_name}>\n</template>")
    } else {
        format!("<template>\n  <{tag_name} {attr_name}=\"...\">...</{tag_name}>\n</template>")
    }
}

fn native_tag_example(tag_name: &str) -> String {
    format!("<template>\n  <{tag_name}>...</{tag_name}>\n</template>")
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod attribute_range_tests;
