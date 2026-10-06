//! Hover cards for the authored Vue directive syntax.
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "`to_string()` and `format!` build the std `String` values tower_lsp::lsp_types payloads take"
)]

use tower_lsp::lsp_types::Hover;

use super::super::{HoverBuilder, HoverService};
use super::template_attribute;
use crate::ide::IdeContext;

impl HoverService {
    pub(super) fn hover_template_directive_attribute(ctx: &IdeContext<'_>) -> Option<Hover> {
        let attr_name = template_attribute::name_at_offset(&ctx.content, ctx.offset)?;

        if let Some(event_name) = attr_name
            .strip_prefix('@')
            .or_else(|| attr_name.strip_prefix("v-on:"))
        {
            let event_name = event_name
                .split_once('.')
                .map_or(event_name, |(name, _)| name);
            let title = if event_name.is_empty() {
                "v-on".to_string()
            } else {
                format!("@{event_name}")
            };
            let example = if event_name.is_empty() {
                "v-on:event=\"handler\"".to_string()
            } else {
                format!("@{event_name}=\"handler\"")
            };

            return Some(
                HoverBuilder::new()
                    .title(&title)
                    .meta("Vue event listener")
                    .example("vue", &example)
                    .description(
                        "Attaches a DOM or component event listener. The handler expression is evaluated in component scope.",
                    )
                    .bullets(
                        "Template behavior",
                        &[
                            "`$event` is available inside inline handler expressions.",
                            "Event modifiers such as `.stop`, `.prevent`, and key modifiers are compiled by Vue.",
                        ],
                    )
                    .docs(
                        "Vue Event Handling",
                        "https://vuejs.org/guide/essentials/event-handling.html",
                    )
                    .build(),
            );
        }

        if attr_name.starts_with(':') || attr_name.starts_with("v-bind:") || attr_name == "v-bind" {
            return Some(
                HoverBuilder::new()
                    .title("v-bind")
                    .meta("Vue attribute / prop binding")
                    .example("vue", ":prop=\"expression\"")
                    .description(
                        "Binds an attribute or component prop to a JavaScript expression in template scope.",
                    )
                    .bullets(
                        "Template behavior",
                        &[
                            "Native element bindings patch DOM attributes or reflected properties.",
                            "Component bindings resolve to props when the target is a component.",
                        ],
                    )
                    .docs(
                        "Vue v-bind",
                        "https://vuejs.org/api/built-in-directives.html#v-bind",
                    )
                    .build(),
            );
        }

        if attr_name.starts_with('#') || attr_name.starts_with("v-slot:") || attr_name == "v-slot" {
            return Self::hover_directive("v-slot");
        }

        if attr_name.starts_with("v-") {
            let without_argument = attr_name
                .split_once(':')
                .map_or(attr_name, |(name, _)| name);
            let base = without_argument
                .split_once('.')
                .map_or(without_argument, |(name, _)| name);
            return Self::hover_directive(base);
        }

        None
    }
}
